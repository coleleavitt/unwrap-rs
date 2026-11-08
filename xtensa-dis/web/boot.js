// Loads the live Xtensa disassembler (Rust compiled to WebAssembly).
// Posts that contain an `.xtensa-demo` include this module; nothing else does.
import init from "./xtensa_dis.js";

init();
