//! Flash layout of the STM32F407VE (plan section 23).

/// Start of flash.
pub const FLASH_BASE: u32 = 0x0800_0000;

/// Sectors 0 to 5 hold the firmware.
pub const FIRMWARE_SIZE: u32 = 256 * 1024;

/// Sector 6.
pub const SLOT_A_ADDR: u32 = 0x0804_0000;

/// Sector 7.
pub const SLOT_B_ADDR: u32 = 0x0806_0000;

/// Each slot is one 128 KB sector.
pub const SLOT_SIZE: u32 = 128 * 1024;

/// A slot starts with a little-endian `u32` sequence number, then the compiled file.
pub const SLOT_SEQUENCE_LEN: usize = 4;
