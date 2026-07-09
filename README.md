# Bevy WebGPU

Bevy WebGPU is a template repository for getting started with [Bevy game engine](https://bevyengine.org/) for web game development using WebGPU. Efficient hot reloading of WGSL files with Vite dev server, as well as recompiling Rust code with page refresh.

Demo: https://bevy-webgpu.vercel.app/

Quickstart:

```bash
pnpx degit mate-h/bevy-webgpu my-project
cd my-project
pnpm i
pnpm run build:wasm
pnpm run dev
```

Features:
- WebGPU support with Bevy version 0.19 engine
- TypeScript support with WASM bindings to Rust
- Hot reloading of WGSL files
- Recompiling Rust code with page refresh
- Instant reloading of shaders without refreshing the page
- Support for multiple examples
- Easy to understand and modify template
- Egui support for debugging
- Built-in infinite debug grid via Bevy's `InfiniteGrid`

No dependencies besides:
- Bevy
- Egui
- Vite
- Rust
- Node
- WASM
- WebGPU 

## Initial setup

Use rust-analyzer for Rust development, as well as shader-analyzer for WGSL linting. It is also recommended to use [Mise en place](https://mise.jdx.dev/getting-started.html) for installing the Rust toolchain and other dependencies.

```bash
mise use -g rust
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo install wasm-opt --locked
pnpm i
```

Production builds with the `pnpm run build` command use a `wasm-release` Cargo profile and run `wasm-opt` to shrink the WASM binary. Dev rebuilds keep the faster `release` profile.

VSCode plugins:
- [Rust Analyzer](https://open-vsx.org/extension/rust-lang/rust-analyzer)
- [Shader Validator](https://open-vsx.org/extension/antaalt/shader-validator)
