extern crate rand;
extern crate hidapi;
use hidapi::HidDevice;
use std::io::Read;

const EP_NUM: u8 = 1;
const BOOTLOADER_INFO_CMD: u8 = 0;
// const ERASE_BLOCK_CMD: u8 = 1;
const ERASE_APP_CMD: u8 = 2;
const START_PROGRAMMING_CMD: u8 = 3;
const PROGRAM_DATA_CMD: u8 = 4;
const ECHO_BACK_CMD: u8 = 9;
const REBOOT_CMD: u8 = 10;

pub struct Bootloader<'a> {
    device: HidDevice<'a>,
    app_offset: u32,
}

#[derive(Debug)]
pub enum BootloaderReason {
    Unknown = -1,
    NotEnteringBootloader = 0,
    BootTokenPresent = 1,
    BootFailedTooManyTimes = 2,
    NoProgramPresent = 3,
    ButtonHeldDown = 4,
}

#[derive(Debug)]
pub enum BootloaderError {
    Unknown = -1,
    NoError = 0,
    UnhandledCommand = 1,
    AddressOutOfRange = 2,
    NoAddressSet = 3,
    SubsystemError = 4,
    AddressNotValid = 5,
    SizeNotValid = 6,
    KeyNotValid = 7,
    FlashNotErased = 8,
}

#[derive(Debug)]
pub struct BootloaderInfo {
    response_code: u32,
    flash_size: u32,
    bootloader_version: u32,
    bootloader_reason: BootloaderReason,
    app_offset: u32,
}

impl From<u8> for BootloaderReason {
    fn from(kind: u8) -> Self {
        match kind {
            0 => BootloaderReason::NotEnteringBootloader,
            1 => BootloaderReason::BootTokenPresent,
            2 => BootloaderReason::BootFailedTooManyTimes,
            3 => BootloaderReason::NoProgramPresent,
            4 => BootloaderReason::ButtonHeldDown,
            _ => BootloaderReason::Unknown,
        }
    }
}

fn dumb_from(kind: u8) -> BootloaderReason {
    match kind {
        0 => BootloaderReason::NotEnteringBootloader,
        1 => BootloaderReason::BootTokenPresent,
        2 => BootloaderReason::BootFailedTooManyTimes,
        3 => BootloaderReason::NoProgramPresent,
        4 => BootloaderReason::ButtonHeldDown,
        _ => BootloaderReason::Unknown,
    }
}

fn bootloader_error_from_u8(kind: u8) -> BootloaderError {
    match kind {
        0 => BootloaderError::NoError,
        1 => BootloaderError::UnhandledCommand,
        2 => BootloaderError::AddressOutOfRange,
        3 => BootloaderError::NoAddressSet,
        4 => BootloaderError::SubsystemError,
        5 => BootloaderError::AddressNotValid,
        6 => BootloaderError::SizeNotValid,
        7 => BootloaderError::KeyNotValid,
        8 => BootloaderError::FlashNotErased,
        _ => BootloaderError::Unknown,
    }
}

impl<'a> Bootloader<'a> {
    pub fn new(dev: hidapi::HidDevice) -> Bootloader {

        println!("Product: {}",
                 dev.get_product_string().expect("Unable to get product string"));

        Bootloader {
            device: dev,
            app_offset: 0,
        }
    }

    pub fn print_info(&mut self) {
        let mut bl_cmd = Self::make_info_cmd(EP_NUM, 0);
        let mut in_buf = [0u8; 8];

        // Dummy read to prime the HID buffer
        self.device.read_timeout(&mut in_buf[..], 1).expect("BL_INFO returned no data");

        self.device.write(&mut bl_cmd[..]).expect("Unable to write BL_INFO command");
        let result =
            self.device.read_timeout(&mut in_buf[..], 100).expect("BL_INFO returned no data");
        if result == 0 {
            self.device.write(&mut bl_cmd[..]).expect("Unable to write BL_INFO command");
            let result =
                self.device.read_timeout(&mut in_buf[..], 100).expect("BL_INFO returned no data");
            if result == 0 {
                panic!("Other end returned no data");
            }
            println!("Note: Got it on the second try.");
        }

        let bl_info = Self::decode_bootloader_info(in_buf);
        self.app_offset = bl_info.app_offset;
        println!("Decoded bootloader data: {:?}", bl_info);
    }

    pub fn echo_test(&self) {
        let mut in_buf = [0u8; 8];

        let mut echo_cmd = Self::make_echo_cmd(EP_NUM, 9);
        self.device.write(&mut echo_cmd[..]).expect("Unable to write echo command");
        let result = self.device.read(&mut in_buf[..]).expect("Unable to read echo command back");
        assert!(result != 0);
        // Sanity: Check that the data coming back is the data we sent.
        assert!(in_buf[1] == 0); // Check that the return code is "no error"
        assert!(in_buf[0] & 0xf0 == echo_cmd[1] & 0xf0); // Check that the sequence number is the same
        assert!(in_buf[0] & 0x0f == 15); // Check that the return type is "Result"
        for i in 2..8 {
            assert!(in_buf[i] == echo_cmd[i + 1]);
        }
    }

    pub fn erase_app(&self) {
        let mut in_buf = [0u8; 8];
        let mut erase_app = Self::make_erase_app_cmd(EP_NUM, 0);

        println!("Attempting to erase app...");
        self.device.write(&mut erase_app[..]).expect("Unable to write erase_app command");
        //        use std::{time, thread};
        //        thread::sleep(time::Duration::from_millis(2000));
        let result = self.device
            .read_timeout(&mut in_buf[..], 100)
            .expect("Unable to get result of app erasure");
        if result == 0 {
            println!("Read timed out");
            assert!(result != 0);
        }
        println!("Result: {:?}", in_buf);
        if in_buf[1] != 0 {
            panic!("Erase error: {:?}", bootloader_error_from_u8(in_buf[1]));
        }
        println!("Result: {:?}", in_buf);
        while in_buf[0] & 0x0f != 15 {
            let result = self.device
                .read_timeout(&mut in_buf[..], 100)
                .expect("Unable to get result of app erasure");
            if result == 0 {
                println!("Read timed out");
                assert!(result != 0);
            }
            println!("Result: {:?}", in_buf);
            if in_buf[1] != 0 {
                panic!("Erase error: {:?}", bootloader_error_from_u8(in_buf[1]));
            }
        }
    }

    pub fn reboot(&self) {
        let mut in_buf = [0u8; 8];
        let mut reboot = Self::make_reboot_cmd(EP_NUM, 3);

        println!("Attempting to reboot...");
        self.device.write(&mut reboot[..]).expect("Unable to write reboot command");
        let result = self.device
            .read_timeout(&mut in_buf[..], 40)
            .expect("Unable to get result of app erasure");
        if result == 0 {
            println!("Result is 0, likely it disconnected");
            return;
        }
        if in_buf[1] != 0 {
            println!("{:?}", in_buf);
            panic!("Reboot error: {:?}", bootloader_error_from_u8(in_buf[1]));
        }
    }

    pub fn program_app<T>(&self, mut firmware: T)
        where T: Read
    {
        let mut in_buf = [0u8; 8];

        let mut firmware_data = vec![];
        firmware.read_to_end(&mut firmware_data).expect("Unable to read firmware file");

        let mut start_program =
            Self::make_start_cmd(EP_NUM, 9, firmware_data.len(), self.app_offset);
        println!("Starting programming...");
        self.device.write(&mut start_program[..]).expect("Unable to start programming");

        let result = self.device
            .read_timeout(&mut in_buf[..], 400)
            .expect("Unable to get result of start_program");
        if result == 0 {
            panic!("Board returned no data when getting result from start_program");
        }
        if in_buf[1] != 0 {
            panic!("Address wasn't valid: {:?}",
                   bootloader_error_from_u8(in_buf[1]));
        }

        let wind = firmware_data.chunks(7);
        for pkt in wind {
            let mut program_data = Self::make_program_data(EP_NUM, 1 & 0xf, pkt);
            self.device.write(&mut program_data).expect("Unable to program data");
            let result =
                self.device.read_timeout(&mut in_buf[..], 40).expect("Unable to program data");
            if result == 0 {
                panic!("Unable to write data");
            }
        }
    }

    fn make_program_data(ep_num: u8, seq_num: u8, data: &[u8]) -> Vec<u8> {
        let mut data_thing = vec![];
        for dat in data {
            data_thing.push(*dat);
        }
        data_thing.resize(7, 0xff);
        vec![ep_num,
             PROGRAM_DATA_CMD | ((seq_num & 0xf) << 4),
             data_thing[0],
             data_thing[1],
             data_thing[2],
             data_thing[3],
             data_thing[4],
             data_thing[5],
             data_thing[6]]
    }

    fn make_start_cmd(ep_num: u8, seq_num: u8, count: usize, offset: u32) -> Vec<u8> {
        println!("Count: {}  Offset: {}", count, offset);
        vec![ep_num,
             START_PROGRAMMING_CMD | ((seq_num & 0xf) << 4),
             0,
             ((count >> 0) & 0xff) as u8,
             ((count >> 8) & 0xff) as u8,
             ((offset >> 0) & 0xff) as u8,
             ((offset >> 8) & 0xff) as u8,
             ((offset >> 16) & 0xff) as u8,
             ((offset >> 24) & 0xff) as u8]
    }

    fn make_info_cmd(ep_num: u8, seq_num: u8) -> Vec<u8> {
        vec![ep_num, BOOTLOADER_INFO_CMD | ((seq_num & 0xf) << 4), 0, 0, 0, 0, 0, 0, 0]
    }

    fn make_reboot_cmd(ep_num: u8, seq_num: u8) -> Vec<u8> {
        vec![ep_num, REBOOT_CMD | ((seq_num & 0xf) << 4), 0x91, 0x82, 0x73, 0x64, 0xad, 0xef, 0xba]
    }

    fn decode_bootloader_info(data: [u8; 8]) -> BootloaderInfo {
        let flash_size = 2u32.pow((data[3] as u32) << 8 | (data[2] as u32));
        BootloaderInfo {
            response_code: data[1] as u32,
            bootloader_version: data[5] as u32,
            bootloader_reason: dumb_from(data[4]), // Because automatic casting isn't working
            flash_size: flash_size,
            app_offset: (((data[7] as u32) << 8) | ((data[6] as u32) << 0)) * flash_size,
        }
    }

    pub fn make_echo_cmd(ep_num: u8, seq_num: u8) -> Vec<u8> {
        vec![ep_num,
             ECHO_BACK_CMD | ((seq_num & 0xf) << 4),
             0xff,
             0xff,
             0xff,
             0xff,
             0xff,
             0xff,
             0xff]
        // use self::rand::Rng;
        // let mut rng = self::rand::thread_rng();
        // vec![ep_num,
        // ECHO_BACK_CMD | ((seq_num & 0xf) << 4),
        // rng.gen(),
        // rng.gen(),
        // rng.gen(),
        // rng.gen(),
        // rng.gen(),
        // rng.gen(),
        // rng.gen()]
        //
    }

    fn make_erase_app_cmd(ep_num: u8, seq_num: u8) -> Vec<u8> {
        vec![ep_num, ERASE_APP_CMD | ((seq_num & 0xf) << 4), 0, 0, 0, 0, 0, 0, 0]
    }
}
