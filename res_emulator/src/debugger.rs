use std::fmt::Display;
use std::ops::Range;

use bincode::Decode;
use bincode::Encode;

use super::cpu::Operation;
use crate::util::RingBuffer;

pub enum MemoryAccess {
    Read(u16),
    Write(u16, u8),
}

impl MemoryAccess {
    pub fn addr(&self) -> u16 {
        match self {
            MemoryAccess::Read(addr) => *addr,
            MemoryAccess::Write(addr, _) => *addr,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Decode, Encode)]
pub enum Trigger {
    CpuMemoryRead(Range<u16>),
    CpuMemoryWrite(Range<u16>),
    CpuMemoryError(),
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Decode, Encode)]
pub enum BreakReason {
    CpuMemoryRead(u16),
    CpuMemoryWrite(u16),
    CpuMemoryError(u16),
}

impl Display for BreakReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BreakReason::CpuMemoryRead(addr) => {
                write!(f, "CPU memory read at address {:04X}", addr)
            }
            BreakReason::CpuMemoryWrite(addr) => {
                write!(f, "CPU memory write at address {:04X}", addr)
            }
            BreakReason::CpuMemoryError(addr) => {
                write!(f, "Invalid CPU memory access at address {:04X}", addr)
            }
        }
    }
}

#[derive(Default, Clone, Decode, Encode)]
pub struct Debugger {
    pub breakpoints: Vec<Trigger>,
    pub break_reason: Option<BreakReason>,
    pub last_ops: RingBuffer<u16, 1024>,
}

impl Debugger {
    pub fn start_execution(&mut self) {
        self.break_reason = None;
    }

    pub fn should_break(&self) -> bool {
        self.break_reason.is_some()
    }

    pub fn break_message(&self) -> String {
        self.break_reason
            .map(|reason| format!("{:}", reason))
            .unwrap_or_default()
    }

    pub fn on_instruction(&mut self, op: &Operation) {
        self.last_ops.push(op.addr);
    }

    pub fn on_cpu_memory_access(&mut self, access: MemoryAccess) {
        for trigger in self.breakpoints.iter() {
            match trigger {
                Trigger::CpuMemoryRead(range) => {
                    if let MemoryAccess::Read(addr) = access {
                        if range.contains(&addr) {
                            self.break_reason = Some(BreakReason::CpuMemoryRead(addr));
                        }
                    }
                }
                Trigger::CpuMemoryWrite(range) => {
                    if let MemoryAccess::Write(addr, _) = access {
                        if range.contains(&addr) {
                            println!("Write: 0x{:04X}", addr);
                            self.break_reason = Some(BreakReason::CpuMemoryWrite(addr));
                        }
                    }
                }
                _ => (),
            }
        }
    }

    pub fn on_cpu_memory_error(&mut self, access: MemoryAccess) {
        for trigger in self.breakpoints.iter() {
            if let Trigger::CpuMemoryError() = trigger {
                self.break_reason = Some(BreakReason::CpuMemoryError(access.addr()));
            }
        }
    }
}
