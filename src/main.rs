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
use hidapi::HidApi;
pub mod bootloader;

fn main() {
    let api = HidApi::new().expect("Failed to create API instance");

    /*
    println!("List of devices:");
    for dev in api.devices() {
        println!("    Device: {:?}", dev);
    }
    */
    let joystick = api.open(0x1bcf, 0x05ce).expect("Failed to open device");
    let bl = bootloader::Bootloader::new(joystick);

    bl.print_info();
    bl.echo_test();

}
