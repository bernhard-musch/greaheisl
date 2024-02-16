# Greaheisl

Timer control on the Arduino UNO R4 Wifi.

## How to test in the terminal

The `lib_rs/greaheisl_emu/` directory conains a Rust terminal application that can be used to test the Rust part of the software interactively on your PC. You do not need to have the Arduino hardware to try that out. 

Unfortunately,  standard terminals do not provide raw keyboard events. Therefore, you need to use a terminal that supports the [kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/), for example [kitty](https://sw.kovidgoyal.net/kitty/).

Open a terminal supporting the kitty keyboard protocol, change to the `lib_rs/` directory and enter

```
cargo run
```

The keyboard mapping is as follows

device button | PC key 
--------------|-----------------
Escape        | left arrow key
Previous      | up arrow key
Next          | down arrow key
Enter         | right arrow key
(exit test)   | Ctrl-C

The first line of the terminal application shows
* The states of the four buttons (0=released, 1=pressed)
* The states of the four relays (as booleans)

## pin configuration on Arduino UNO R4 Wifi

The software in this repository was designed with the following connections of electrical signals in mind.

pin | configured as  | connected to
----|----------------|----------------------------
D2  | `INPUT_PULLUP` | button `Escape`
D3  | `INPUT_PULLUP` | button `Previous`
D4  | `INPUT_PULLUP` | button `Next`
D5  | `INPUT_PULLUP` | button `Enter`
D6  | `OUTPUT`       | relay board input signal 1
D7  | `OUTPUT`       | relay board input signal 2
D8  | `OUTPUT`       | relay board input signal 3
D9  | `OUTPUT`       | relay board input signal 4

The four push buttons facilitate input from the user.  Electrically, the buttons can be simple switches. As a safety measure, the current through the switches can be limited by a resistor connected in series.

The software configures 4 pins of the Arduino Uno R4 Wifi board as outputs that can be used to switch appliances on and off. There are modules available on the market that are compatible with the Arduino Uno R4 Wifi and offer multiple relays on a single board for that purpose.

### Safety notice

> **Electricity can cause serious injuries and damage!**
> 
> If you plan to design and build your own circuit 
> so you can try out the software on the Arduino board,
> make sure you are competent to do so. 
> This includes knowledge about safety hazards 
> and skills to take appropriate measures to prevent 
> them.

Here is just a selection of important aspects you need to have under control:
* voltages dangerous to the human body
* proper insulation
* safe wire connections
* adequate dimensioning and placement of fuses
* required conductor cross sections
* compatibility of the connected devices 
* humidity and temperature, cooling / ventilation
* flammeable materials near electric components
* protection of other people, potentially children 
* exposure to unhealthy substances

## How to build the software for the real device

Install [Rust](https://www.rust-lang.org/) according to their web site.
Make sure you have a suitable target for the RA4M1 microcontroller from Renesas in your toolchain:

```
rustup target add thumbv7em-none-eabihf
``` 

We need `cbindgen` to generate the header files:

```
cargo install cbindgen
```

To cross compile the library for use on the Arduino Rev4 Wifi use the following script

```
./make_arduino_lib.sh
```

This will build a static library and store the results in `prog/libraries/greaheisl_lib`.

Now install and open the [Arduino IDE 2.x](https://www.arduino.cc/en/software). Go to `File` -> `Preferences...` -> `Settings` -> `Sketchbook location:` and specify the full path to `prog/`.

Now open `prog/greaheisl/greaheisl.ino`. You should be able to compile it and upload it to your device.

Notes:

* At the heart of the build script is the command
`cargo build --release --target thumbv7em-none-eabihf`
* The thumbv7em-none-eabihf target is for ARM Cortex-M4F and Cortex-M7F (with FPU support).

## Documentation

To browse documentation for the Rust code, change to directory `lib_rs` and run

```
cargo doc --no-deps --document-private-items --open
```

## License


Licensed under either of [Apache License, Version
2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

## Note

The source code for `yield_now()` has been copied from
[the async version of the Rust standard library](https://github.com/async-rs/async-std) published under the same license.