use std::fs;


pub fn load_shader(filename: &str) -> String 
{
    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");

    contents
}