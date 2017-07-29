extern crate libusb;
extern crate rustc_serialize;

use std::io::BufRead;
use rustc_serialize::hex::FromHex;


fn read_hex_file(f : &std::fs::File) -> Result<Vec<u8>, String> {

    let filereader = std::io::BufReader::new(f);
    let mut buf : std::vec::Vec<u8> = vec![0;64];
    
    let mut file_end = false;
    let mut line_number = 0;
    
    for line in filereader.lines() {
        line_number += 1;
        let line_number_str = &line_number.to_string();
    
        match line {        
            Err(e) => return Err(format!("{}", e)),
            Ok(line) => {
                if line == "\n" {
                    return Err(format!("Unexpected empty line in line {}", line_number_str));
                }
                if file_end {
                    return Err(format!("Unexpected data after firmware end in line {}", line_number_str));
                }
                let linebytes  = (&line[1 .. ]).from_hex().unwrap();
                let len : usize = linebytes[0] as usize;
                if len+5 == linebytes.len() {
                    let mut sum : u32 = 0;
                    for i in 0..linebytes.len() { 
                        sum += linebytes[i] as u32;
                    }
                    if sum & 0xFF != 0 {
                        return Err(format!("Checksum mismatch in line {}", line_number_str));
                    }
                    
                    let address : u16 = (linebytes[1] as u16) << 8
                                        |  linebytes[2] as u16;
                    let linetype = linebytes[3];
                    let data = &linebytes[4..len+4];
                    
                    match linetype {
                        0x00 => {
                            while address as usize + data.len() as usize > buf.len() {
                                let buflen = buf.len();
                                buf.resize(buflen * 2, 0)
                            }
                            for i in 0..data.len() {
                                buf[address as usize + i] = data[i];
                            }
                        },
                        0x01 => file_end = true, 
                        _ => return Err(format!("Unknow data type in line {}", line_number_str)),
                    }
                } else {
                    return Err(format!("Line length mismatch in line {}", line_number_str));
                }
            }
        }
    }
        
    if file_end {
        Ok(buf)
    } else {
        Err(format!("Unexpected end of file"))
    }
}

fn write_packet(device_handle : &libusb::DeviceHandle, buf : &[u8;64])
        -> Result<usize, libusb::Error> {
    let request_type : u8 = libusb::request_type(libusb::Direction::Out,
                                                libusb::RequestType::Class,
                                                libusb::Recipient::Interface);
    let request : u8  = 9;
    let value : u16 = 0x0300;
    let index : u16 = 0;
    device_handle.write_control(request_type, request, value, index, buf, std::time::Duration::new(1, 0))
}

fn read_packet(device_handle : &libusb::DeviceHandle)
        ->  std::result::Result<Vec<u8>, libusb::Error> {
    // I have no idea how large this buffer should be, or what happens if the buffer is too small
    let mut bufvec : Vec<u8> = vec![0; 4096];

    let request_type : u8 = libusb::request_type(libusb::Direction::In,
                                                libusb::RequestType::Class,
                                                libusb::Recipient::Interface);
    let request : u8  = 1;
    let value : u16 = 0x0300;
    let index : u16 = 0;
    
    match device_handle.read_control(request_type, request, value, index, &mut bufvec, std::time::Duration::new(1, 0)) {
        Err(e) => Err(e),
        Ok(bytes_read) => {bufvec.resize(bytes_read, 0); Ok(bufvec)}
    }
}

fn flash_firmware(device : &libusb::Device, firmware : &[u8]) -> Result<(), libusb::Error> {
    println!("Open device …");
    let mut device_handle = try!(device.open());    
    print!("Detach kernel … ");
    match device_handle.detach_kernel_driver(0) {
        Err(e) => println!("already detached. ({})", e),
        _ => println!("done."),
    }
    println!("Claim interface …");
    try!(device_handle.claim_interface(0));
    
    let mut buf: [u8; 64] = [0; 64]; 
    
    buf[0] = 0x33;
    buf[5] = ((firmware.len() & 0xFF00) >> 8) as u8;
    buf[6] = (firmware.len() & 0x00FF) as u8;
    
    print!("[");
    let written_bytes = try!(write_packet(&device_handle, &buf));
    if written_bytes != 64 { 
        println!("Not all bytes have been written when requesting write access! \nYou should flash the keyboard again, to make sure the image is properly written. If the problem persists, your keyboard's flash chip might be broken.");
    }
    
    let mut i = 0;
    while i < firmware.len() {
        let mut buf: [u8; 64] = [0; 64]; 
        
        let to_write = &firmware[i..];
        let bytes_remaining = to_write.len();
        let bytes_to_copy = std::cmp::min(64, bytes_remaining);
        
        buf[..bytes_to_copy].clone_from_slice(&to_write[..bytes_to_copy]);
        
        print!(".");
        let written_bytes = try!(write_packet(&device_handle, &buf));
        
        if written_bytes != 64 { 
            println!("Not all bytes have been written at offset {}! \nYou should flash the keyboard again, to make sure the image is properly written. If the problem persists, your keyboard's flash chip might be broken.", i);
        }
        i += 64;
    }
    
    let mut buf: [u8; 64] = [0; 64]; 
    
    buf[0] = 0x22;
    buf[6] = 0x02;
    
    print!("?");
    let written_bytes = try!(write_packet(&device_handle, &buf));
    if written_bytes != 64 { 
        println!("Not all bytes have been written when requesting checksum! \nYou should flash the keyboard again, to make sure the image is properly written. If the problem persists, your keyboard's flash chip might be broken.");
    }
    println!("]");
    
    let result = try!(read_packet(&device_handle));
    
    if result.len() < 2 {
        println!("Keyboard gave incomplete result when checking for the Checksum. Cannot verify checksum!");
        return Err(libusb::Error::Other);
    }
    
    let mut checksum : u32 = 0;
    for byte in firmware { checksum += *byte as u32; checksum = checksum & 0xFFFF; }
    
    let result_checksum : u32 = (result[0] as u32) << 8
                              |  result[1] as u32;
    
    if checksum == result_checksum { 
        println!("Checksum OK!"); Ok(())
    } else {
        println!("Programming failed: Checksum mismatch!\nYou should flash the keyboard again, to make sure the image is properly written. If the problem persists, your keyboard's flash chip might be broken.");  Err(libusb::Error::Other)
    }
}

fn switch_mode(device : &libusb::Device) -> Result<(), libusb::Error> {
    println!("Open device …");
    let mut device_handle = try!(device.open());    
    print!("Detach kernel … ");
    match device_handle.detach_kernel_driver(0) {
        Err(e) => println!("already detached. ({})", e),
        _ => println!("done."),
    }
    println!("Claim interface …");
    try!(device_handle.claim_interface(0));
    
    let mut buf: [u8; 64] = [0; 64]; 
    buf[0] = 0x44;

    println!("Send switch command!");
    if try!(write_packet(&device_handle, &buf)) != 64 {
        println!("Not all bytes were written switching the mode!");
        Err(libusb::Error::Other)
    } else {
        Ok(())
    }
}

#[derive(PartialEq)]
enum Mode {
    Program,
    Normal,
    Either,
}

fn is_tek(device : &libusb::Device, mode : Mode) -> bool {
    // 0x<vendor><product>
    let normal_mode_id = 0x0E6A030C;
    let program_mode_id = normal_mode_id - 1;

    let device_desc = device.device_descriptor().unwrap();
    
    let device_id : u32 = (device_desc.vendor_id()  as u32) << 16 
                        |  device_desc.product_id() as u32;

    if device_id == normal_mode_id && mode != Mode::Program ||
        device_id == program_mode_id && mode != Mode::Normal {
        true
    } else {
        false
    }
}

fn find_keyboard<'a>(context : &'a libusb::Context) -> Result<libusb::Device<'a>, libusb::Error> {    
    let devices = try!(context.devices());
    let mut tek_devices = devices.iter().filter(|d| {is_tek(d, Mode::Either)});
    
    match tek_devices.next() {
        Some(device) => {
            match tek_devices.next() {
                None => {
                    return Ok(device)
                },
                _ => {
                    println!("More then one TEK device found!"); 
                    Err(libusb::Error::Other)
                }
            }
        },
        None => {
            println!("No TEK device found!");
            Err(libusb::Error::Other)
        }
    }
}

fn prepare_keyboard() -> Result<(), libusb::Error> {
    let context = try!(libusb::Context::new());
    let device = try!(find_keyboard(&context));

    if is_tek(&device, Mode::Program) {
        println!("TEK is already in program mode. That is strange. I won't continue now …");
        Err(libusb::Error::Other)
    }
    else if is_tek(&device, Mode::Normal) {
        println!("TEK is in normal mode.\nSwitch to program mode …");
        switch_mode(&device)
    } else {Err(libusb::Error::Other)}
}

fn finish_keyboard() -> Result<(), libusb::Error> {
    let context = try!(libusb::Context::new());
    let device = try!(find_keyboard(&context));

    if is_tek(&device, Mode::Normal) {
        println!("TEK is already in normal mode. That is strange.");
        Err(libusb::Error::Other)
    }
    else if is_tek(&device, Mode::Program) {
        println!("TEK is still in program mode.\nSwitch to normal mode …");
        switch_mode(&device)
    } else {Err(libusb::Error::Other)}
}


fn program_keyboard(firmware : &[u8]) -> Result<(), libusb::Error> {
    let context = try!(libusb::Context::new());
    let device = try!(find_keyboard(&context));


    if is_tek(&device, Mode::Normal) {
        println!("TEK is still in normal mode. Mode switching did not work! ");
        println!("Did you set DIP #5 to 'programmable'?");
        Err(libusb::Error::Other)
    }
    else if is_tek(&device, Mode::Program) {
        println!("TEK is in program mode.\nFlash firmware …");
        flash_firmware(&device, firmware)
    } else {Err(libusb::Error::Other)}
}


fn release_keyboard() -> Result<(), libusb::Error> {
    let context = try!(libusb::Context::new());
    let device = try!(find_keyboard(&context));
    
    println!("Reattach kernel driver");
    let mut device_handle = try!(device.open());
    device_handle.attach_kernel_driver(0)
}


fn open_file(filename : &str) -> Result<std::fs::File, String> {
    match std::fs::File::open(filename) {
        Ok(file) => Ok(file),
        Err(e) => Err(format!("{}", e)),
    }
}

fn show_devices() {
    let context = libusb::Context::new().unwrap();

    println!("Seeing the following devices: ");
    for device in context.devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id());
    }
    println!("");
}

fn flash_sequence(filename : &str) -> Result<(), String> {
    let file = try!(open_file(filename));
    let firmware = try!(read_hex_file(&file));

    show_devices();
    
    match prepare_keyboard() {
        Err(error) => println!("Some error occured during switching back to programming mode: {}. I will not program now but try to switch back.", error),
        _ => {
            std::thread::sleep(std::time::Duration::new(2, 0));
            match program_keyboard(&firmware) {
                Err(error) => println!("Some error occured during programming the keyboard: {}. Now trying to switch back to normal mode.", error),
                _ => (),
            }
        }
    }
    std::thread::sleep(std::time::Duration::new(2, 0));
    match finish_keyboard() {
        Err(error) => println!("Some error occured during switching back to normal mode: {}. Probably you need to reconnect your keyboard.", error),
        _ => (),
    }
    std::thread::sleep(std::time::Duration::new(4, 0));
    match release_keyboard() {
        Err(error) => println!("Some error occured during reattaching the kernel driver: {}. Maybe the kernel driver is already attached. Otherwise, you probably need to reconnect your keyboard.", error),
        _ => (),
    }
    
    println!("\nDone.");
    Ok(())
}

fn main() {  
    let mut argv = std::env::args();
    
    match argv.nth(1) {
        Some(filename) => {
            match flash_sequence(&filename) {
                Err(error) => println!("{}", error),
                _ => (),
            }
        }
        None => println!("Error: Filename parameter missing!")
    }
}
