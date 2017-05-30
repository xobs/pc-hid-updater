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

struct BlPkt {
    cmd: u8,
    seq: u8,
    data: [u8; 6],
}

fn main() {
    let api = HidApi::new().expect("Failed to create API instance");

    println!("List of devices:");
    for dev in api.devices() {
        println!("    Device: {:?}", dev);
    }
    let joystick = api.open(0x1bcf, 0x05ce).expect("Failed to open device");
    let mut pkt_num = 0;
    let offset = 0u32;

    loop {
        let mut in_buf = [0u8; 9];
        let mut out_buf = [0u8; 9];

        out_buf[0] = 2;
        out_buf[1] = pkt_num;
        out_buf[2] = 5;

        out_buf[3] = 0;
        out_buf[4] = 1;

        out_buf[5] = (offset & 0xff) as u8;
        out_buf[6] = ((offset >> 8) & 0xff) as u8;
        out_buf[7] = ((offset >> 16) & 0xff) as u8;
        out_buf[8] = ((offset >> 24) & 0xff) as u8;

        match joystick.write(&mut out_buf[..]) {
            Ok(_) => (),//println!("Wrote {} bytes", write_res),
            //Err(msg) => panic!("An error occurred while writing: {}", msg),
            Err(msg) => {println!("An error occurred while writing: {}", msg); continue},
        }
        
        //std::thread::sleep(std::time::Duration::from_millis(20));
        match joystick.read_timeout(&mut in_buf[..], 1000) {
            Ok(res) if res == 0 => {println!("No data read"); continue; },
            Err(_) => {println!("Unknown error occurred when reading"); continue; },
            Ok(res) => {
                println!("Done with read, formatting {} bytes", res);
                let mut data_string = String::new();

                for u in &in_buf[..res] {
                    data_string.push_str(&(u.to_string() + "\t"));
                }

                println!("{}", data_string);
        
                pkt_num = pkt_num.wrapping_add(1);
                //offset += 256;
            },
        }
    }
}