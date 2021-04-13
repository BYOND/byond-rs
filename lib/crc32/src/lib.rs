#![cfg_attr(not(any(feature = "std", test)), no_std)]

#[cfg(not(feature = "std"))]
use core::hash::Hasher;
#[cfg(feature = "std")]
use std::hash::Hasher;

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

    pub fn update(&mut self, _: &[u8]) {
        todo!()
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
