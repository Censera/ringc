//! Compiled schedule format shared by the host compiler and the firmware.
//!
//! `no_std`, no allocation.

#![no_std]

pub mod crc;
pub mod header;
pub mod layout;
