# Third-party notices

## Bootstrap Icons 1.11.3

The icons inlined in `frontend/index.html` and `sitegen/src/main.rs`
(journal-text, person-vcard, laptop, github, linkedin, envelope-fill, key-fill, rss-fill) come from
https://icons.getbootstrap.com.

```
The MIT License (MIT)

Copyright (c) 2019-2024 The Bootstrap Authors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
```

## JetBrains Mono

`frontend/fonts/JetBrainsMono-latin.woff2` is a Latin subset of JetBrains Mono,
under the SIL Open Font License 1.1 in `frontend/fonts/OFL.txt`.

## Rust syntax definition (sublimehq/Packages)

`sitegen/syntaxes/Rust.sublime-syntax` is `Rust/Rust.sublime-syntax` from
https://github.com/sublimehq/Packages at commit
7f74b54967118d74d7e2d439b19c526c025ea6b5 (2026-02-11), unmodified.
It is used only at build time to highlight Rust code in posts.

```
Permission to copy, use, modify, sell and distribute this
software is granted. This software is provided "as is" without
express or implied warranty, and with no claim as to its
suitability for any purpose.
```

## LLVM Xtensa target (llvm-project)

`xtensa-dis/src/table.rs` is generated from the TableGen records of LLVM's
Xtensa target (`llvm/lib/Target/Xtensa/*.td`, llvm-project llvmorg-22.1.8),
and the operand decoding and printing in `xtensa-dis/src/lib.rs` follow
`XtensaDisassembler.cpp` and `XtensaInstPrinter.cpp` from the same release.
LLVM is licensed under the Apache License v2.0 with LLVM Exceptions
(https://llvm.org/LICENSE.txt).
