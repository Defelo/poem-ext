#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug)]
#![warn(missing_docs, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod auth;
#[cfg(feature = "sea-orm")]
pub mod db;
pub mod panic_handler;
pub mod patch_value;
pub mod responses;
#[cfg(feature = "shield")]
pub mod shield_mw;
mod static_string;
