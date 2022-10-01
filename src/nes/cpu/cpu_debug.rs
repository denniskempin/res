use bincode::{Decode, Encode};

use crate::util::RingBuffer;

use super::Operation;

#[derive(Default, Encode, Decode, Clone)]
pub struct CpuDebug {
    pub last_ops: RingBuffer<u16, 1024>,
}

impl CpuDebug {
    pub fn before_op(&mut self, op: &Operation) {
        self.last_ops.push(op.addr);
    }
}
