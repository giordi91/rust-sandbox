#[cfg(target_arch = "wasm32")]
macro_rules! log {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }

pub fn greet() {
    println!("HELLLOOOOOOO FROM WASM");
}
