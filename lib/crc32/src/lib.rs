#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(not(feature = "std"))]
use core::convert::TryInto;
#[cfg(not(feature = "std"))]
use core::hash::Hasher;
#[cfg(feature = "std")]
use std::convert::TryInto;
#[cfg(feature = "std")]
use std::hash::Hasher;

mod tables;

const DEFAULT_CRC32: u32 = 0xffffffff;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Crc32 {
    state: u32,
}

impl Crc32 {
    pub fn new() -> Self {
        Self::from(DEFAULT_CRC32)
    }

    pub fn as_u32(&self) -> u32 {
        self.state
    }

    pub fn reset(&mut self) {
        self.state = DEFAULT_CRC32;
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.state = update_fast(self.state, bytes);
    }
}

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

impl From<u32> for Crc32 {
    fn from(state: u32) -> Self {
        Self { state }
    }
}

impl Hasher for Crc32 {
    fn finish(&self) -> u64 {
        self.as_u32() as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

impl PartialEq<u32> for Crc32 {
    fn eq(&self, &other: &u32) -> bool {
        self.as_u32() == other
    }
}

impl PartialEq<Crc32> for u32 {
    fn eq(&self, other: &Crc32) -> bool {
        *self == other.as_u32()
    }
}

// TODO: Big-endian implementation.
#[inline(always)]
pub(crate) fn update_fast(mut crc: u32, bytes: &[u8]) -> u32 {
    use tables::WORD_TABLE;
    crc = u32::swap_bytes(crc);
    let chunks = bytes.chunks_exact(8);
    let remainder = chunks.remainder();
    crc = chunks.fold(crc, |crc, word| {
        let word = u64::from(crc) ^ u64::from_le_bytes(word.try_into().unwrap());
        WORD_TABLE[7][(word & 0xff) as usize]
            ^ WORD_TABLE[6][((word >> 8) & 0xff) as usize]
            ^ WORD_TABLE[5][((word >> 16) & 0xff) as usize]
            ^ WORD_TABLE[4][((word >> 24) & 0xff) as usize]
            ^ WORD_TABLE[3][((word >> 32) & 0xff) as usize]
            ^ WORD_TABLE[2][((word >> 40) & 0xff) as usize]
            ^ WORD_TABLE[1][((word >> 48) & 0xff) as usize]
            ^ WORD_TABLE[0][(word >> 56) as usize]
    });
    crc = u32::swap_bytes(crc);
    update_slow(crc, remainder)
}

#[inline(always)]
pub(crate) fn update_slow(crc: u32, bytes: &[u8]) -> u32 {
    bytes.iter().fold(crc, |crc, &byte| {
        (crc << 8) ^ tables::BYTE_TABLE[(crc >> 24) as usize ^ byte as usize]
    })
}

#[cfg(test)]
mod tests {
    use quickcheck::quickcheck;

    #[test]
    fn check() {
        let mut crc = super::Crc32::new();
        crc.update(b"123456789");
        assert_eq!(0xa5fd3138u32, crc);
    }

    fn golden(crc: u32, bytes: &[u8]) -> u32 {
        bytes.iter().fold(crc, |mut crc, &byte| {
            crc ^= u32::from(byte) << 24;
            for _ in 0..8 {
                crc = if crc & 0x80000000 != 0 {
                    (crc << 1) ^ 0xaf
                } else {
                    crc << 1
                };
            }
            crc
        })
    }

    #[test]
    fn golden_is_valid() {
        assert_eq!(0xa5fd3138, golden(crate::DEFAULT_CRC32, b"123456789"));
    }

    quickcheck! {
        fn update_slow_matches_golden(crc: u32, bytes: Vec<u8>) -> bool {
            super::update_slow(crc, &bytes) == golden(crc, &bytes)
        }

        fn update_fast_matches_golden(crc: u32, bytes: Vec<u8>) -> bool {
            super::update_fast(crc, &bytes) == golden(crc, &bytes)
        }
    }
}
