use std::io;
use std::fs;

pub async fn load_file_u8(file_name: &str) -> Result<Vec<u8>, io::Error> {
    fs::read(file_name)
}

pub async fn load_file_string(file_name: &str) -> Result<String, io::Error> {
    fs::read_to_string(file_name)
}

pub async fn file_exists(file_name: &str) -> bool {
    std::path::Path::new(file_name).exists()
}

