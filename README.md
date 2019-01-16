# TEK programmer
A programmer for "Truly Ergonomic" keyboards. Basically [m-ou-se/teck-programmer](https://github.com/m-ou-se/teck-programmer) rewritten in Rust.

This tool allows you to update the layout of your Truly Ergonomic keyboard with files generated from the [layout generator]( https://www.trulyergonomic.com/store/layout-designer--configurator--reprogrammable--truly-ergonomic-mechanical-keyboard/) or with a [default layout](https://www.trulyergonomic.com/store/default-layouts--truly-ergonomic-mechanical-keyboard).

WARNING: Usage is at your own risk. This program has only been testet with the model 229 so far. If you bricked your keyboard, you can try to perform a [manual reset](https://web.archive.org/web/20160324205503/https://trulyergonomic.com/store/knowledge-base--truly-ergonomic-mechanical-keyboard#Reset).

## Compile
You need rust and cargo for compilation. In Ubuntu, you can use the packages from your distribution 

    sudo apt install cargo rustc

Compile using

    cargo build --release

## Usage
Run using

    target/release/tek-programmer path/to/your/firmware.hex

If you want to compile and run in one step, you can just invoke

    cargo run --release path/to/your/firmware.hex

## Issues

You have any problems using this software? Open an issue or drop me a message at robert+git@zgtm.de.

If you bricked your keyboard, you can try to perform a [manual reset](https://web.archive.org/web/20160324205503/https://trulyergonomic.com/store/knowledge-base--truly-ergonomic-mechanical-keyboard#Reset).

### Manual reset (summary)
 * Open keyboard (two screws are hidden behind the label)
 * Set DIP switch #5 to ON (Firmware protected!)
 * Plug in the keyboard to your computer
 * Connect pins 1 and 36 on the microcontroller (leftmost of top and bottom row)
 * Upload firmware
