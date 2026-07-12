# karna on the web

This crate is a small demo that runs the engine in the browser via
WebAssembly + WebGPU.

## One-time setup

1. **Rust wasm target**

   ```sh
   rustup target add wasm32-unknown-unknown
   ```

2. **trunk** (builds, bundles and serves the wasm app)

   ```sh
   cargo install trunk --locked
   ```

3. **wasi-sdk** — dear imgui is C++, so it has to be cross-compiled to
   wasm. Download the latest release from
   <https://github.com/WebAssembly/wasi-sdk/releases> (the
   `x86_64-windows` tarball) and extract it to `%USERPROFILE%\.wasi-sdk`
   so that `~\.wasi-sdk\bin\clang++.exe` exists.

   `.cargo/config.toml` points the wasm build at this toolchain (compiler
   env vars for `cc`, plus linking wasi-libc for imgui's `vsnprintf`/math
   calls). If you install it elsewhere, update the paths there.

## Run it

```sh
trunk serve --config web/Trunk.toml
```

then open <http://127.0.0.1:8721/>. Requires a WebGPU-capable browser
(Chrome/Edge stable, Firefox stable, Safari 26+).

## How it works (web specifics)

- `App::run` cannot block on the web: gpu setup is awaited inside a
  `wasm_bindgen_futures::spawn_local` future, then the winit event loop is
  started with `EventLoopExtWebSys::spawn_app`.
- There are no threads: each window's `WindowState` lives on the main
  thread and `WindowState::frame_once` is driven by `RedrawRequested`
  (requestAnimationFrame), re-requesting a redraw each frame. Native keeps
  the thread-per-window blocking loop.
- The "window" is a `<canvas>` appended to `<body>`. The browser reports
  the canvas size asynchronously, so the engine falls back to the
  `WindowBuilder` size until the first real `Resized` event arrives.
- `wasi-shim.js` satisfies the `wasi_snapshot_preview1` imports that
  musl's exit-time stdio machinery drags in via imgui's `__cxa_atexit`;
  none of it can run in a browser.
- Audio is stubbed out on the web for now (`Mixer::play` warns).
