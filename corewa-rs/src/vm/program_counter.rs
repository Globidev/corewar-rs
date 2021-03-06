use super::types::OffsetType;
use crate::spec::{IDX_MOD, MEM_SIZE};

#[derive(Debug, Default, derive_more::From)]
pub struct ProgramCounter(usize);

fn mem_offset(at: usize, offset: isize) -> usize {
    (at as isize + offset + MEM_SIZE as isize) as usize % MEM_SIZE
}

impl ProgramCounter {
    pub fn advance(&mut self, offset: isize) {
        self.0 = mem_offset(self.0, offset);
    }

    pub fn offset(&self, offset: isize, offset_type: OffsetType) -> usize {
        let reach = match offset_type {
            OffsetType::Limited => IDX_MOD,
            OffsetType::Long => MEM_SIZE,
        };
        let offset = offset % reach as isize;
        mem_offset(self.0, offset)
    }

    pub fn addr(&self) -> usize {
        self.0
    }
}
