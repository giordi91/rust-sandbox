#[cfg(target_arch = "wasm32")]
macro_rules! log {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }

pub fn greet() {
    log!("HELLLOOOOOOO FROM WASM");
}
