#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod sdl;
#[cfg(not(target_arch = "wasm32"))]
pub use sdl::*;
