/****************************************************************************
    Copyright (c) 2015 Osspial All Rights Reserved.
    This file is part of hidapi-rs, based on hidapi_rust by Roland Ruckerbauer.
    hidapi-rs is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.
    hidapi-rs is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.
    You should have received a copy of the GNU General Public License
    along with hidapi-rs.  If not, see <http://www.gnu.org/licenses/>.
****************************************************************************/

//! Opens a Thrustmaster T-Flight HOTAS X HID and reads data from it. This 
//! example will not work unless such an HID is plugged in to your system. 
//! Will update in the future to support all HIDs. 

extern crate hidapi;
extern crate clap;
use clap::{Arg, App};

use hidapi::HidApi;
pub mod bootloader;

use std::fs::File;

fn main() {
    let matches = App::new("Joyboot: Palawan bootloader API")
        .version("1.2")
        .author("Sean Cross <sean@xobs.io>")
        .about("Communicates with the Palawan USB HID bootloader")
        .arg(Arg::with_name("firmware")
            .short("f")
            .long("firmware")
            .value_name("FIRMWARE")
            .help("Firmware file")
            .takes_value(true))
        .get_matches();

    let firmware_filename = matches.value_of("firmware").expect("Unable to get firmware path");
    let firmware_file = File::open(firmware_filename).expect("Unable to open firmware file");

    let api = HidApi::new().expect("Failed to create API instance");

    println!("List of devices:");
    for dev in api.devices() {
        println!("    Device: {:?}", dev);
    }

    let joystick = api.open(0x1bcf, 0x05ce).expect("Failed to open device");
    let mut bl = bootloader::Bootloader::new(joystick);

    bl.print_info();
    bl.echo_test();
    bl.echo_test();
    bl.echo_test();
    bl.erase_app();
    bl.program_app(firmware_file);
    bl.reboot();
}
