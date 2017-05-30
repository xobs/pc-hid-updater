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

const EP_NUM: u8 = 2;

fn main() {
    let api = HidApi::new().expect("Failed to create API instance");

    /*
    println!("List of devices:");
    for dev in api.devices() {
        println!("    Device: {:?}", dev);
    }
    */
    let joystick = api.open(0x1bcf, 0x05ce).expect("Failed to open device");
    println!("Product: {}", joystick.get_product_string().expect("Unable to get product string"));

    let mut bl_cmd = bootloader::make_info_cmd(EP_NUM, 0);
    let mut in_buf = [0u8; 8];

    // Dummy read to prime the HID buffer
    joystick.read_timeout(&mut in_buf[..], 1).expect("BL_INFO returned no data");

    joystick.write(&mut bl_cmd[..]).expect("Unable to write BL_INFO command");
    let result = joystick.read_timeout(&mut in_buf[..], 100).expect("BL_INFO returned no data");
    if result == 0 {
        joystick.write(&mut bl_cmd[..]).expect("Unable to write BL_INFO command");
        let result = joystick.read_timeout(&mut in_buf[..], 100).expect("BL_INFO returned no data");
        if result == 0 {
            panic!("Other end returned no data");
        }
        println!("Note: Got it on the second try.");
    }
    println!("Raw bootlaoder data: {:?}", in_buf);
    println!("Decoded bootloader data: {:?}", bootloader::decode_bootloader_info(in_buf));

    let mut echo_cmd = bootloader::make_echo_cmd(EP_NUM, 9);
    joystick.write(&mut echo_cmd[..]).expect("Unable to write echo command");
    let result = joystick.read(&mut in_buf[..]).expect("Unable to read echo command back");
    assert!(result != 0);
    println!("Orig: {:?},  Readback: {:?}", echo_cmd, in_buf);
    // Sanity: Check that the data coming back is the data we sent.
    assert!(in_buf[1] == 0); // Check that the return code is "no error"
    assert!(in_buf[0] & 0xf0 == echo_cmd[1] & 0xf0); // Check that the sequence number is the same
    assert!(in_buf[0] & 0x0f == 15); // Check that the return type is "Result"
    for i in 2 .. 8 {
        println!("Testing in_buf[{}]", i);
        assert!(in_buf[i] == echo_cmd[i + 1]);
    }
}
