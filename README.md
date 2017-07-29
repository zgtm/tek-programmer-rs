# TEK programmer
A programmer for "Truly Ergonomic" keyboards. Basically https://github.com/m-ou-se/teck-programmer rewritten in Rust.

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
