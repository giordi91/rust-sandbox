use std::io;
use std::fs;

pub async fn load_file_u8(file_name: &String) -> Result<Vec<u8>, io::Error> {
    fs::read(file_name)
}

pub async fn load_file_string(file_name: &String) -> Result<String, io::Error> {
    fs::read_to_string(file_name)
}

pub async fn file_exists(file_name: &str) -> bool {
    return std::path::Path::new(file_name).exists();
}

