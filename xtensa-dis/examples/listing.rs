//! Print a listing: `cargo run -p xtensa-dis --example listing -- 40080000 "36 41 00"`
fn main() {
    let mut args = std::env::args().skip(1);
    let base = xtensa_dis::parse_address(&args.next().unwrap_or_default()).expect("base address");
    let code = xtensa_dis::parse_hex(&args.next().unwrap_or_default()).expect("hex bytes");
    println!("{}", xtensa_dis::listing(&code, base));
}
