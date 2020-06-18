mod sandbox;

use rust_sandbox::engine::platform;
use sandbox::Sandbox;


fn main() {
    platform::run_application::<Sandbox>("Rust Sandbox v0.0.2");
}
//set RUSTFLAGS=--cfg=web_sys_unstable_apis & cargo build --target wasm32-unknown-unknown && wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/debug/rust-sandbox.wasm
