#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate log;

pub mod oss;
pub mod prelude;
#[cfg(not(target_arch = "wasm32"))]
pub mod object;
#[cfg(not(target_arch = "wasm32"))]
pub mod service;
pub mod errors;

mod utils;
mod auth;

