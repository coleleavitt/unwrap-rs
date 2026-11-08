#!/usr/bin/env python3
"""Compare xtensa-dis with llvm-mc on every possible input.

    cargo build --release -p xtensa-dis
    python3 xtensa-dis/tools/check_llvm.py [--jobs 20]

Needs `llvm-mc` from LLVM 22 with the Xtensa target. Each input is one
bracketed line (`[0xc0 0x20 0x00]`), which llvm-mc decodes on its own at
address 0. The inputs are:

- all 65,536 two-byte words;
- all three-byte words whose low nibble is not 8-D (no 16-bit
  instruction starts with them): 10,485,760;
- all three-byte words whose first two bytes look like a 16-bit
  instruction (low nibble 8-D) but are not one: 977,920.

Every other three-byte input starts with a valid 16-bit instruction, which
both decoders pick first, so the two-byte words cover it. Prints the totals
and exits non-zero on any difference.
"""
import argparse, os, re, subprocess, sys, tempfile
from concurrent.futures import ProcessPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OURS = os.path.join(ROOT, "target", "release", "xtensa-dis")
LLVM = ["llvm-mc", "--triple=xtensa", "-mcpu=esp32", "--disassemble"]


def line(word, size):
    return "[" + " ".join(f"0x{(word >> (8 * k)) & 0xff:02x}" for k in range(size)) + "]\n"


def check(path):
    """Run both decoders on one input file; return (lines, differences, examples)."""
    with open(path) as f:
        inputs = f.read().splitlines()
    # llvm-mc exits 1 whenever it reports an invalid encoding, which is expected
    llvm = subprocess.run(LLVM + [path], capture_output=True, text=True)
    bad = {int(m) for m in re.findall(r":(\d+):\d+: warning: invalid instruction encoding", llvm.stderr)}
    refs = [l.strip() for l in llvm.stdout.splitlines() if l.startswith("\t")]
    if len(refs) != len(inputs) - len(bad):
        raise SystemExit(f"{path}: llvm-mc output does not line up with its input")
    with open(path) as f:
        ours = subprocess.run([OURS], stdin=f, capture_output=True, text=True, check=True).stdout.splitlines()
    refs_iter = iter(refs)
    diffs, examples = 0, []
    for n, (inp, our) in enumerate(zip(inputs, ours), 1):
        ref = None if n in bad else " ".join(next(refs_iter).split())
        our = None if our == "<invalid>" else " ".join(our.split())
        if ref != our:
            diffs += 1
            if len(examples) < 5:
                examples.append((inp, ref, our))
    return len(inputs), diffs, examples


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--jobs", type=int, default=os.cpu_count())
    args = ap.parse_args()
    with tempfile.TemporaryDirectory() as tmp:
        files = []
        def write(name, words, size):
            path = os.path.join(tmp, name)
            with open(path, "w") as f:
                f.writelines(line(w, size) for w in words)
            files.append(path)
        write("two-byte.txt", range(1 << 16), 2)
        for op0 in [o for o in range(16) if not 8 <= o <= 0xD]:
            for hi in range(0, 256, 64):  # 64 Ki words per file
                write(f"op0-{op0:x}-{hi:02x}.txt",
                      (op0 | rest << 4 for rest in range(hi << 12, (hi + 64) << 12)), 3)
        with ProcessPoolExecutor(args.jobs) as pool:
            first = pool.submit(check, files[0]).result()
        # 16-bit words that look narrow but are not, extended by a third byte
        two = [l for l in open(files[0]).read().splitlines()]
        ours = subprocess.run([OURS], input="\n".join(two) + "\n", capture_output=True, text=True).stdout.splitlines()
        not_narrow = [w for w in range(1 << 16) if 8 <= w & 0xF <= 0xD and ours[w] == "<invalid>"]
        write("narrow-lookalikes.txt", (w | hi << 16 for w in not_narrow for hi in range(256)), 3)
        with ProcessPoolExecutor(args.jobs) as pool:
            results = [first] + list(pool.map(check, files[1:]))
    total = sum(r[0] for r in results)
    diffs = sum(r[1] for r in results)
    print(f"{total:,} inputs, {diffs:,} differences")
    for example in [e for r in results for e in r[2]][:10]:
        print("  input %s\n    llvm-mc: %s\n    ours:    %s" % example)
    sys.exit(1 if diffs else 0)


if __name__ == "__main__":
    main()
