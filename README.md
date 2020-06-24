# rust-sandbox

![Continuous integration](https://github.com/giordi91/rust-sandbox/workflows/Continuous%20integration/badge.svg)


- [rust-sandbox](#rust-sandbox)
- [Compilation](#compilation)
  - [External crate dependencies](#external-crate-dependencies)
  - [WASM compilation](#wasm-compilation)
- [Examples](#examples)
  - [Hello Triangle](#hello-triangle)
  - [GLTF Model](#gltf-model)

This is a learning project for me to learn both Rust and WebGPU. I am aiming to be able to use the same code base to easily target the browser and native, the engine does its best to 
hide the platform differences to the user and hopefully the code will just work on both platforms.

A main difference to my previous projects like SirEngine, this is not an attempt to make a game engine, but I want to have fun with graphics again. As such, control is still fairly in the end of the user there are no big system to automagically deal with lot of the stuff for you. 

An example would be, the engine helps you load shaders, meshes and so on, but then control is returned to you, is up to you to schedule render properly, bind correct binding group and rendering pipelines. This approach will allow me to focus more on the graphics and less on the engine.


<img src="examples/hello-triangle/screenshot.png" alt="triangle" width="256"/>
<img src="examples/gltf-model/suzanne.gif" alt="triangle" width="256"/>

# Compilation

This technology stack is on its early days, as such compiling and running is fiddly and a bit messy, hopefully should improve quickly.

The project has been developed on both windows and mac, on Vulkan and metal backends respectively. If you have issues on any platform let me know.

**NOTE**
The sandbox is the project I use for development, so might not work or be broken or who knows what state it is in. What you want to build and run are the examples. 


Install rust:
```
https://www.rust-lang.org/tools/install
```
## External crate dependencies
The below dependencies are used to compile all the crates, not directly
used by the engine.

Install python 3, one possible way is using choco on windows or follow your platform instructions from the library developer:
```
https://chocolatey.org/packages/python3
```
Install Ninja build:
```
choco install ninja
```

Optional: you might have to fix a dependency of cc, hopefully will be fixed soon.
```
cargo update -p cc --precise 1.0.50
```

Finally you can run:
```
cargo build
```

## WASM compilation

To target wasm you will also need to add the wasm-bindgen-cli
```
cargo install -f wasm-bindgen-cli
```

And the wasm32 target

```
rustup target add wasm32-unknown-unknown
```

To compile you need to use the unstable web_sys API. You do that by setting the environment variable
```
RUSTFLAGS=--cfg=web_sys_unstable_apis
``` 

Next you compile targeting wasm
```
cargo build --target wasm32-unknown-unknown
```

Finally you need to run the wasm-bindgen command to generate the js glue
```
wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/debug/rust-sandbox.wasm
```

I personally run the sequence of commands all in one go:

windows
```
set RUSTFLAGS=--cfg=web_sys_unstable_apis & cargo build --target wasm32-unknown-unknown && wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/debug/rust-sandbox.wasm
```

mac
```
RUSTFLAGS=--cfg=web_sys_unstable_apis  cargo build --target wasm32-unknown-unknown && wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/debug/rust-sandbox.wasm
```

Once you did that  you will find inside ```target/generated``` a bunch of js file and your wasm, to trigger it, you need a basic index.html to load the javascript. Where the javascript you load is the one of the example you have built, or the sandbox itself if you so desire

```html
<html>
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
  </head>
  <body>
    <script type="module">
      import init from "./rust-sandbox.js";
      init();
    </script>
  </body>
</html>
```

This instructions are exactly the same as outlined on [wgpu-rs](https://github.com/gfx-rs/wgpu-rs).

**NOTE**: To run this example you need a browser that supports web GPU, support is sparse and varied. I only tested with Firefox Nightly. I am unsure if will run on any other browser due to the GLSL-SPIRV combo. The language has been standardized to WGSL but no working implementation is available yet. To use Firefox Nightly you also need to enable:

```
dom.webgpu.enabled
```
and

```
gfx.webrender.all
```
To do so type ```about:config``` in the search bar then search for the two options and enable them. You might need to restart the browser.

**What about WASMPACK?**

WASMPACK makes like much easier and streamlines the build, I do want to use it but get it to work for both native and not requires a bit of work I would rather not do now and focus on the graphics. PRs are welcome!
The main problem relies in how WAMSPACK triggers the wasm, I will have to split the project in multiple crates to make it to work.

# Examples

To run the examples compile with:

WASM
```
set RUSTFLAGS=--cfg=web_sys_unstable_apis & cargo build --target wasm32-unknown-unknown --example hello-triangle && wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/debug/examples/hello-triangle.wasm
```

NATIVE
```
cargo run --example hello-triangle
```

Replace ```hello-triangle``` with the example you are trying to build and run.
Examples should come with a basic camera you can move with the keyboard arrows.

## Hello Triangle

The simplest thing I could do with the engine. Here you can see the initial plumbing of the engine, how it loads the different resources for you etc.

<img src="examples/hello-triangle/screenshot.png" alt="triangle"/>


## GLTF Model

In this example we load a GLTF model, just position and normals only. I hope you like
gif compression artefacts!

<img src="examples/gltf-model/suzanne.gif" alt="triangle">
