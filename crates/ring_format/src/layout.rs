//! Flash layout of the STM32F103C8T6 (plan section 23).
//!
//! The chip has 64 KB of flash in 1 KB pages. Many Blue Pill boards carry chips with 128 KB,
//! but only 64 KB is guaranteed, so the layout stays inside it.

/// Start of flash.
pub const FLASH_BASE: u32 = 0x0800_0000;

pub const FLASH_SIZE: u32 = 64 * 1024;

/// Smallest erasable unit.
pub const PAGE_SIZE: u32 = 1024;

/// Pages 0 to 47 hold the firmware.
pub const FIRMWARE_SIZE: u32 = 48 * 1024;

/// Each slot is eight pages.
pub const SLOT_SIZE: u32 = 8 * 1024;

/// Pages 48 to 55.
pub const SLOT_A_ADDR: u32 = FLASH_BASE + FIRMWARE_SIZE;

/// Pages 56 to 63.
pub const SLOT_B_ADDR: u32 = SLOT_A_ADDR + SLOT_SIZE;

/// A slot starts with a little-endian `u32` sequence number, then the compiled file.
pub const SLOT_SEQUENCE_LEN: usize = 4;

/// Largest compiled file a slot can hold.
pub const MAX_FILE_LEN: usize = SLOT_SIZE as usize - SLOT_SEQUENCE_LEN;

const _: () = assert!(SLOT_B_ADDR + SLOT_SIZE == FLASH_BASE + FLASH_SIZE);
const _: () = assert!(FIRMWARE_SIZE % PAGE_SIZE == 0 && SLOT_SIZE % PAGE_SIZE == 0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses() {
        assert_eq!(SLOT_A_ADDR, 0x0800_C000);
        assert_eq!(SLOT_B_ADDR, 0x0800_E000);
        assert_eq!(MAX_FILE_LEN, 8188);
    }
}
