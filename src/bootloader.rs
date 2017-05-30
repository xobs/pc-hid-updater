extern crate rand;
extern crate hidapi;
use hidapi::HidDevice;

pub struct Bootloader<'a> {
    device: HidDevice<'a>,
}

const EP_NUM: u8 = 2;
const BOOTLOADER_INFO_CMD: u8 = 0;
// const ERASE_BLOCK_CMD: u8 = 1;
const ERASE_APP_CMD: u8 = 2;
const ECHO_BACK_CMD: u8 = 9;

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
pub struct BootloaderInfo {
    response_code: u32,
    flash_size: u32,
    bootloader_version: u32,
    bootloader_reason: BootloaderReason,
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


pub fn make_erase_app_cmd(ep_num: u8, seq_num: u8) -> Vec<u8> {
    vec![ep_num, ERASE_APP_CMD | ((seq_num & 0xf) << 4), 0, 0, 0, 0, 0, 0, 0]
}

impl<'a> Bootloader<'a> {
    pub fn new(dev: hidapi::HidDevice) -> Bootloader {

        println!("Product: {}",
                 dev.get_product_string().expect("Unable to get product string"));

        Bootloader { device: dev }
    }

    pub fn print_info(&self) {
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
        println!("Raw bootlaoder data: {:?}", in_buf);
        println!("Decoded bootloader data: {:?}",
                 Self::decode_bootloader_info(in_buf));
    }

    pub fn echo_test(&self) {
        let mut in_buf = [0u8; 8];
        let mut echo_cmd = Self::make_echo_cmd(EP_NUM, 9);
        self.device.write(&mut echo_cmd[..]).expect("Unable to write echo command");
        let result = self.device.read(&mut in_buf[..]).expect("Unable to read echo command back");
        assert!(result != 0);
        println!("Orig: {:?},  Readback: {:?}", echo_cmd, in_buf);
        // Sanity: Check that the data coming back is the data we sent.
        assert!(in_buf[1] == 0); // Check that the return code is "no error"
        assert!(in_buf[0] & 0xf0 == echo_cmd[1] & 0xf0); // Check that the sequence number is the same
        assert!(in_buf[0] & 0x0f == 15); // Check that the return type is "Result"
        for i in 2..8 {
            println!("Testing in_buf[{}]", i);
            assert!(in_buf[i] == echo_cmd[i + 1]);
        }
    }

    fn make_info_cmd(ep_num: u8, seq_num: u8) -> Vec<u8> {
        vec![ep_num, BOOTLOADER_INFO_CMD | ((seq_num & 0xf) << 4), 0, 0, 0, 0, 0, 0, 0]
    }

    fn decode_bootloader_info(data: [u8; 8]) -> BootloaderInfo {
        BootloaderInfo {
            response_code: data[1] as u32,
            bootloader_version: data[5] as u32,
            bootloader_reason: dumb_from(data[4]), // Because automatic casting isn't working
            flash_size: 2u32.pow((data[3] as u32) << 8 | (data[2] as u32)),
        }
    }

    pub fn make_echo_cmd(ep_num: u8, seq_num: u8) -> Vec<u8> {
        use self::rand::Rng;
        let mut rng = self::rand::thread_rng();
        vec![ep_num,
             ECHO_BACK_CMD | ((seq_num & 0xf) << 4),
             rng.gen(),
             rng.gen(),
             rng.gen(),
             rng.gen(),
             rng.gen(),
             rng.gen(),
             rng.gen()]
    }
}
