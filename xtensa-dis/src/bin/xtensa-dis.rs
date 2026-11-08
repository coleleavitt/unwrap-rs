//! `xtensa-dis`: disassemble hex bytes, one instruction per input line, in
//! the format `llvm-mc --disassemble` prints (used to check the decoder).
//! Like `llvm-mc`, it treats plain input as one byte stream, so each line's
//! address is the number of bytes on the lines before it, while a bracketed
//! line (`[0xc0 0x20 0x00]`) is decoded on its own at address 0.
//!
//!     echo "0xc0 0x20 0x00" | xtensa-dis

use std::io::{BufRead, Write};

use xtensa_dis::{Targets, decode};

fn main() {
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let mut address = 0u32;
    for line in std::io::stdin().lock().lines() {
        let line = line.expect("stdin is readable");
        let bytes: Vec<u8> = line
            .split(|c: char| c.is_whitespace() || c == '[' || c == ']')
            .filter(|t| !t.is_empty())
            .filter_map(|t| u8::from_str_radix(t.trim_start_matches("0x"), 16).ok())
            .collect();
        let at = if line.trim_start().starts_with('[') {
            0
        } else {
            address
        };
        address = address.wrapping_add(bytes.len() as u32);
        match decode(&bytes, at, Targets::Relative) {
            Some(i) if i.size == bytes.len() => writeln!(out, "{}\t{}", i.mnemonic, i.operands),
            Some(_) => writeln!(out, "<size mismatch>"),
            None => writeln!(out, "<invalid>"),
        }
        .expect("stdout is writable");
    }
}
