use super::super::Platform;

pub fn get_platform() -> Platform {
    Platform::BROWSER
}

macro_rules! log {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }

pub fn to_console(message: &str) {
    log!("{}", message);
}


pub fn get_time_in_micro() -> u64 {
    (web_sys::window().unwrap().performance().unwrap().now() * 1000.0) as u64
}
