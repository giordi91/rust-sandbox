#[cfg(target_arch = "wasm32")]


use super::super::Platform;

pub fn get_platform() -> Platform
{
    Platform::BROWSER
}

    /*
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};


pub async fn load_file_wasm_vec32(url : &String) -> Result<Vec<u32>, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    //TODO investigate, any way to have less jumps around?
    //we extract the array buffer from our response 
    let t : JsValue = JsFuture::from(resp.array_buffer()?).await?;
    //we initialize the array buffer out of the JsValue
    let bt :js_sys::ArrayBuffer = js_sys::ArrayBuffer::from(t);
    //next we use the array buffer to initialize a uin32Array
    let typebuf: js_sys::Uint32Array = js_sys::Uint32Array::new(&bt);
    //finally we copy this array into the final vector
    let mut body = vec![0 as u32; typebuf.length() as usize];
    typebuf.copy_to(&mut body);
    Ok(body)
}

*/