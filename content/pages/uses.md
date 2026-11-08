+++
title: Uses
description: The hardware, system and tools Cole Leavitt works with every day: a ThinkPad on Gentoo with a self-built kernel, niri, and Rust tools he wrote himself.
updated: 2026-11-08
+++

I have used Linux since I was 13. Today almost everything runs on one laptop, and where a tool didn't fit, I wrote my own: those are linked below.

## Machine

- **Lenovo ThinkPad P16 Gen 3**: Intel Core Ultra 9 275HX (24 cores), 128 GB RAM, NVIDIA RTX PRO 4000 Blackwell laptop GPU, 3840×2400 display, 4 TB Samsung NVMe.
- Thunderbolt 5 through an Intel JHL9580 (Barlow Ridge) host, which is where [the resume race](/writing/thunderbolt-resume-race/) came from, and Intel Wi-Fi 7, where [the iwlwifi fix](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=92cee08dc4f00e77fd1317e4343c5d458b0abab7) came from.

## System

- **Gentoo Linux** with OpenRC, everything compiled for this CPU (`-march=arrowlake-s -O2`).
- **Kernel**: mainline release candidates built from source, with my own patches on top while they are in review. pstore/ramoops is set up so a crash leaves its log for the next boot.
- **Packages**: my [Gentoo overlay](https://github.com/coleleavitt/coleleavitt-overlay) plus a local repository for pinned drivers.

## Desktop

- **[niri](https://github.com/YaLTeR/niri)**, a scrollable-tiling Wayland compositor, with Waybar, mako, wofi and swayidle.
- **Login, lock screen and sudo**: greetd with [linux_hello](/writing/linux-hello-face-unlock/), my IR face unlock, through PAM.
- **Terminal**: foot with MonoLisa, bash with starship and zoxide, tmux.
- **Editors**: Neovim, plus JetBrains RustRover and WebStorm.
- **Apps**: Zen Browser, Thunderbird, Vesktop.

## Keys and identity

- **[keyoxide](/writing/keyoxide-gpg-agent/)**, my Rust agent, replaces both gpg-agent and ssh-agent.
- OpenPGP key DEC2 3202 3B77 3F66 98EA 54F1 1DFB 3167 910E 6193, a vanity prefix found with [pgp-vanity](https://github.com/coleleavitt/pgp-vanity), published here through [WKD](/pgp.asc) and verified on [Keyoxide](https://keyoxide.org/DEC232023B773F6698EA54F11DFB3167910E6193).

## Toolchain

- **Rust**: nightly, built from rust-lang/rust master, used for everything here.
- **C and C++**: clang 22 and GCC 15, linked with mold, cached with sccache.
- **Code structure**: [codeflow](/writing/codeflow-mir/) for call trees and control-flow graphs, including rustc MIR.

## Reverse engineering

- **IDA Pro 9.5** with idalib, driven by agents through my [ida-mcp-server](https://github.com/coleleavitt/ida-mcp-server).
- **Frida** for dynamic analysis.
- My own tools: [unlace](https://github.com/coleleavitt/unlace) (decompiler), [jsbeautify](https://github.com/coleleavitt/jsbeautify) (JavaScript deobfuscator), [xtensa-decoder](/writing/xtensa-decoder-apartment-wifi/).

## Hosting

- **This site**: one static Rust binary built on [servant-rs](https://github.com/coleleavitt/servant-rs) behind nginx on a Linode VPS. No JavaScript except the animation, no trackers.
- **Mail**: my own Stalwart mail server for `unwrap.rs`.
