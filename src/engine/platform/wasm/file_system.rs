use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use super::core;

pub async fn load_file_u8(url : &String) -> Result<Vec<u8>, JsValue> {
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
    //next we use the array buffer to initialize a uin8Array
    let typebuf: js_sys::Uint8Array = js_sys::Uint8Array::new(&bt);
    //finally we copy this array into the final vector
    let mut body = vec![0 as u8; typebuf.length() as usize];
    typebuf.copy_to(&mut body);
    Ok(body)
}

pub async fn load_file_string(url : &String) -> Result<String, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    let text = JsFuture::from(resp.text()?).await?.as_string().unwrap();
    //lovely I know!!
    Ok(JsValue::from_serde(&text).unwrap().as_string().unwrap())
}

pub async fn file_exists(file_name: &str) -> bool {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request_opt = Request::new_with_str_and_init(&file_name, &opts);
    let request = match request_opt {
        Ok(req) => req,
        Err(e) => {
            let message = format!("[Error]: Could not make HTTP request for file {} with error {:?}",file_name,e);
            core::to_console(&message[..]);
            return false
        }
    };

    let window = web_sys::window().unwrap();
    let resp_value_future = JsFuture::from(window.fetch_with_request(&request)).await;
    let resp_value  = match resp_value_future 
    {
        Ok(resp) => resp,
        Err(e) => {
            let message = format!("[Error]: Could not make fetch request for file {} with error {:?}",file_name,e);
            core::to_console(&message[..]);
            return false 
        }
    };

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();
    resp.status() == 200
}