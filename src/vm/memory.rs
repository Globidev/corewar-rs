use crate::spec::*;
use super::types::*;
use super::wrapping_array::WrappingArray;

pub struct Memory {
    values: WrappingArray<u8>,
    ages: WrappingArray<u16>,
    owners: WrappingArray<PlayerId>
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            values: WrappingArray::with_size(MEM_SIZE),
            ages: WrappingArray::repeat(MEM_SIZE, 1024),
            owners: WrappingArray::with_size(MEM_SIZE)
        }
    }
}

impl Memory {
    pub fn size(&self) -> usize {
        MEM_SIZE
    }

    pub fn values_ptr(&self) -> *const u8 {
        self.values.as_ptr()
    }

    pub fn ages_ptr(&self) -> *const u16 {
        self.ages.as_ptr()
    }

    pub fn owners_ptr(&self) -> *const PlayerId {
        self.owners.as_ptr()
    }

    pub fn tick(&mut self) {
        for age in self.ages.iter_mut() {
            *age = age.saturating_sub(1)
        }
    }

    pub fn write(&mut self, at: usize, bytes: &[u8], owner: PlayerId) {
        for (i, byte) in bytes.iter().enumerate() {
            self.values[at + i] = *byte;
            self.ages[at + i] = 1024;
            self.owners[at + i] = owner
        }
    }

    pub fn read_i32(&self, addr: usize) -> i32 {
          (i32::from(self[addr    ]) << 24)
        + (i32::from(self[addr + 1]) << 16)
        + (i32::from(self[addr + 2]) << 8 )
        + (i32::from(self[addr + 3])      )
    }

    pub fn read_i16(&self, addr: usize) -> i16 {
          (i16::from(self[addr    ]) << 8)
        + (i16::from(self[addr + 1])     )
    }

    pub fn write_i32(&mut self, value: i32, owner: PlayerId, at: usize) {
        let value_as_bytes: [u8; 4] = unsafe { std::mem::transmute(value.to_be()) };

        self.write(at, &value_as_bytes, owner)
    }
}

use std::ops::Index;

impl Index<usize> for Memory {
    type Output = u8;

    fn index(&self, index: usize) -> &u8 {
        self.values.index(index)
    }
}

impl super::decoder::Decodable for Memory {
    fn read_i16(&self, at: usize) -> i16 {
        self.read_i16(at)
    }

    fn read_i32(&self, at: usize) -> i32 {
        self.read_i32(at)
    }
}
