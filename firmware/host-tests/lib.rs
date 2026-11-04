//! Compiles the firmware's hardware-free modules for the host so their
//! `#[cfg(test)]` unit tests run with plain `cargo test`.
#![no_std]

extern crate alloc;
#[cfg(test)]
extern crate std;

#[path = "../src/dns.rs"]
pub mod dns;
#[path = "../src/settings.rs"]
pub mod settings;
#[path = "../src/web.rs"]
pub mod web;
