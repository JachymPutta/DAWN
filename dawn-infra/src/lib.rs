#![warn(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    missing_docs,
    clippy::perf,
    clippy::missing_docs_in_private_items,
    clippy::panic
)]
#![allow(clippy::unused_async, clippy::module_name_repetitions)]

//!general coding infrastructure for implementing a dap debugger

///! communication codec
pub mod codec;
///! adapted dap requests
pub mod dap_requests;
///! dap debugger
pub mod debugger;
