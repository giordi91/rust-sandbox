# rust-sandbox

#compiling

Install rust:
```
https://www.rust-lang.org/tools/install
```
Install python 3, one possible way is using choco or from installer:
```
https://chocolatey.org/packages/python3
```
Install Ninja build:
```
choco install ninja
```
To target wasm you will also need to add the wasm-bindgen-cli
```
cargo install -f wasm-bindgen-cli
```

wasmpack

```
https://rustwasm.github.io/wasm-pack/installer/
```

Finally you can run:
```
cargo bulid
```
