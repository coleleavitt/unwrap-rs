# xtensa-dis

An Xtensa disassembler in Rust, generated from LLVM's TableGen records and
checked against `llvm-mc` on every possible input. It runs as WebAssembly in
the post [My apartment Wi-Fi was corrupting my uploads, so I wrote an Xtensa
decoder](https://unwrap.rs/writing/xtensa-decoder-apartment-wifi/).

- `src/table.rs`: 283 instructions (mask, match, operand fields), generated
  by `tools/gen_table.py` from `llvm-tblgen -dump-json` of LLVM 22.1.8's
  Xtensa target (llvm-project `ca7933e`). Do not edit by hand.
- `src/lib.rs`: decoding and printing, following LLVM's
  `XtensaDisassembler.cpp` and `XtensaInstPrinter.cpp`. Special and user
  registers are the ones `llvm-mc -mcpu=esp32` accepts.
- `src/web.rs` (wasm32 only): makes each `.xtensa-demo` on a page live. The site's
  Trunk hook builds it with content-hashed file names and writes the `boot-<hash>.js`
  loader.
- `src/bin/xtensa-dis.rs`: prints one instruction per input line in
  `llvm-mc`'s format, for the check below.

## Check

```sh
cargo build --release -p xtensa-dis
python3 xtensa-dis/tools/check_llvm.py
# 11,529,216 inputs, 0 differences
```

It compares every two-byte word, every three-byte word that does not start
with a 16-bit instruction, and every three-byte word that looks like one but
is not (see the script). Agreement means the same valid or invalid verdict,
the same mnemonic and the same operands. The configuration is LLVM's
`esp32` processor, which enables every Xtensa option LLVM implements.

## Regenerate the table

```sh
git clone --depth 1 --branch llvmorg-22.1.8 --filter=blob:none --sparse \
    https://github.com/llvm/llvm-project.git llvm-project
cd llvm-project && git sparse-checkout set llvm/include llvm/lib/Target/Xtensa
llvm-tblgen -dump-json -I llvm/include -I llvm/lib/Target/Xtensa \
    llvm/lib/Target/Xtensa/Xtensa.td -o xtensa.json
python3 tools/gen_table.py xtensa.json > src/table.rs
```

## License

MIT OR Apache-2.0, like the rest of the repository. The instruction table
is derived from LLVM, which is Apache-2.0 WITH LLVM-exception; see
`THIRD-PARTY-NOTICES.md` at the repository root.
