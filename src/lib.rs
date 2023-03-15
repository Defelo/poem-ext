//! Extension crate for [poem]/[poem_openapi]

#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug)]
#![warn(missing_docs, missing_debug_implementations)]

mod auth;
pub mod panic_handler;
pub mod patch_value;
pub mod responses;
mod static_string;
