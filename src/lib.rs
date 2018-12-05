extern crate reqwest;
extern crate base64;
extern crate chrono;
extern crate crypto;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate xml;
extern crate futures;

pub mod oss;
pub mod prelude;
pub mod object;
pub mod service;
mod utils;
mod auth;
mod error;