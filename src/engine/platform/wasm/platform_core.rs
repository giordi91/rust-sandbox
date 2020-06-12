
macro_rules! log {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }


pub fn to_console(message : &str)
{
    log!("{}",message);
}