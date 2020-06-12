
use super::super::Platform;

pub fn get_platform() -> Platform {
    Platform::NATIVE
}

pub fn to_console(message : &str)
{
    println!("{}",message);
}