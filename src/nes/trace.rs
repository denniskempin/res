use anyhow::anyhow;
use anyhow::Result;
use regex::Regex;
use std::fmt::Display;
use std::num::ParseIntError;

#[derive(Clone, Debug, Default)]
pub struct Trace {
    pub pc: u16,
    pub opcode_raw: Vec<u8>,
    pub opcode_str: String,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
}

impl PartialEq for Trace {
    fn eq(&self, other: &Self) -> bool {
        self.pc == other.pc
            && self.opcode_raw == other.opcode_raw
            && self.a == other.a
            && self.x == other.x
            && self.y == other.y
    }
}

static TRACE_REGEX: &str = "(.{4})  (.{8})  (.{30})  A:(.{2}) X:(.{2}) Y:(.{2}) P:(.{2})";

impl Display for Trace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcode_raw_str = self
            .opcode_raw
            .iter()
            .map(|c| format!("{c:02X}"))
            .collect::<Vec<String>>()
            .join(" ");

        write!(
            f,
            "{:04X}  {:<8}  {:<30}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X}",
            self.pc, opcode_raw_str, self.opcode_str, self.a, self.x, self.y, self.p,
        )
    }
}

impl Trace {
    pub fn from_log_line(trace_str: &str) -> Result<Trace> {
        let re = Regex::new(TRACE_REGEX).unwrap();

        let captures = re
            .captures(trace_str)
            .ok_or_else(|| anyhow!("Not a valid trace string"))?;

        Ok(Trace {
            pc: u16::from_str_radix(&captures[1], 16)?,
            opcode_raw: captures[2]
                .trim()
                .split(' ')
                .map(|s| u8::from_str_radix(s, 16))
                .collect::<Result<Vec<u8>, ParseIntError>>()?,
            opcode_str: captures[3].trim().to_string(),
            a: u8::from_str_radix(&captures[4], 16)?,
            x: u8::from_str_radix(&captures[5], 16)?,
            y: u8::from_str_radix(&captures[6], 16)?,
            p: u8::from_str_radix(&captures[7], 16)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::Trace;

    #[test]
    pub fn test_parse_fmt_trace() {
        let trace_str = concat!(
            "C000  4C F5 C5  JMP $C5F5                       ",
            "A:00 X:00 Y:00 P:24"
        );
        let trace = Trace {
            pc: 0xC000,
            opcode_raw: vec![0x4C, 0xF5, 0xC5],
            opcode_str: "JMP $C5F5".to_string(),
            a: 0,
            x: 0,
            y: 0,
            p: 0x24,
        };
        assert_eq!(trace_str, format!("{trace}"));
        assert_eq!(trace, Trace::from_log_line(trace_str).unwrap());
    }
}
