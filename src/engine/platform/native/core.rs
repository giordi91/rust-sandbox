
use super::super::Platform;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_platform() -> Platform {
    Platform::NATIVE
}

pub fn to_console(message : &str)
{
    println!("{}",message);
}

pub fn get_time_in_micro() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64
}