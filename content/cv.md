+++
title: CV
description: Cole Leavitt's CV: security research, Linux kernel work, open-source Rust tools, and education.
+++

<!-- The CV repeats homepage facts (frontend/index.html). Change both together. -->
<div class="cv">
    <section class="section" aria-labelledby="experience-heading">
      <h2 id="experience-heading" class="section-label">Experience</h2>
      <ul class="entries">
        <li><div class="entry-static"><span class="entry-name">Principal Security Engineer</span><span class="entry-desc">Security engineering as an individual contributor</span><span class="entry-meta">2024–now</span></div></li>
      </ul>
    </section>
    <section class="section" aria-labelledby="research-heading">
      <h2 id="research-heading" class="section-label">Research</h2>
      <ul class="entries">
        <li>
          <a href="/writing/aamva-pdf417/"><span class="entry-name">AAMVA PDF417</span><span class="entry-desc">ID scanners accept spoofed driver's license barcodes</span><span class="entry-meta">2025-07</span></a>
          <ul class="entry-refs">
            <li><a href="https://www.cve.org/CVERecord?id=CVE-2025-31337">CVE-2025-31337</a> IDScan.net</li>
            <li><a href="https://www.cve.org/CVERecord?id=CVE-2025-31336">CVE-2025-31336</a> TokenWorks</li>
          </ul>
        </li>
      </ul>
    </section>
    <section class="section" aria-labelledby="kernel-heading">
      <h2 id="kernel-heading" class="section-label">Linux kernel</h2>
      <ul class="entries">
        <li><a href="https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=92cee08dc4f00e77fd1317e4343c5d458b0abab7"><span class="entry-name">iwlwifi</span><span class="entry-desc">Fix TSO segmentation explosion when A-MSDU is disabled</span><span class="entry-meta">2026-05</span></a></li>
        <li><a href="https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=2ddb69f686ef7a621645e97fc7329c50edf5d0e5"><span class="entry-name">pstore/ram</span><span class="entry-desc">Fix a resource leak when ioremap() fails</span><span class="entry-meta">2026-03</span></a></li>
        <li><a href="https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=2c96956fe764f8224f9ec93b2a9160a578949a7a"><span class="entry-name">soundwire</span><span class="entry-desc">Demote expected UNATTACHED state warnings to debug</span><span class="entry-meta">2026-03</span></a></li>
        <li><a href="https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=122439234f6ae2583a3701d7619f9a8242748547"><span class="entry-name">i915</span><span class="entry-desc">Reported the CX0 PHY turnaround timing bug Intel fixed</span><span class="entry-meta">2026-02</span></a></li>
        <li><a href="https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/commit/?id=390caeed0897fcac75f3c414dbdd85d593183d9c"><span class="entry-name">ASoC cs42l43</span><span class="entry-desc">Enable the headphone pin for the LINEOUT jack type</span><span class="entry-meta">2026-01</span></a></li>
        <li><a href="/writing/thunderbolt-resume-race/"><span class="entry-name">In review</span><span class="entry-desc">PCI hotplug and Thunderbolt resume race; USB hcd-pci (acked by Alan Stern)</span><span class="entry-meta">2026-10</span></a></li>
      </ul>
      <p class="section-more">Released in Linux 6.19 and 7.1. <a href="https://lore.kernel.org/all/?q=f%3Acole%40unwrap.rs">All patches and reviews on lore</a></p>
    </section>
    <section class="section" aria-labelledby="open-source-heading">
      <h2 id="open-source-heading" class="section-label">Open source</h2>
      <ul class="entries">
        <li><a href="https://github.com/coleleavitt/unlace"><span class="entry-name">unlace</span><span class="entry-desc">Open-source decompiler in Rust with Mach-O, Objective-C and Swift support</span><span class="entry-meta">2026-04</span></a></li>
        <li><a href="https://github.com/coleleavitt/ida-mcp-server"><span class="entry-name">ida-mcp-server</span><span class="entry-desc">IDA Pro plugin exposing 170+ analysis tools to coding agents over MCP</span><span class="entry-meta">2026-02</span></a></li>
        <li><a href="https://github.com/Vector35/idb-rs/pull/61"><span class="entry-name">idb-rs</span><span class="entry-desc">IDA 9.5 database support for Vector35's IDB parser (PR #61, in review)</span><span class="entry-meta">2026-10</span></a></li>
        <li><a href="https://github.com/coleleavitt/servant-rs"><span class="entry-name">servant-rs</span><span class="entry-desc">Rust port of Haskell Servant; this site runs on it</span><span class="entry-meta">2026-06</span></a></li>
        <li><a href="https://github.com/coleleavitt/jsbeautify"><span class="entry-name">jsbeautify</span><span class="entry-desc">JavaScript beautifier, deobfuscator and bundle unpacker built on oxc</span><span class="entry-meta">2026-01</span></a></li>
        <li><a href="/writing/keyoxide-gpg-agent/"><span class="entry-name">keyoxide</span><span class="entry-desc">SSH and GPG agent in Rust that serves gpg-agent's protocol</span><span class="entry-meta">2026-11</span></a></li>
        <li><a href="/writing/codeflow-mir/"><span class="entry-name">codeflow</span><span class="entry-desc">Call trees and control-flow graphs for coding agents, incl. rustc MIR</span><span class="entry-meta">2026-10</span></a></li>
        <li><a href="https://github.com/coleleavitt/remarkable-ecosystem"><span class="entry-name">reMarkable</span><span class="entry-desc">Reverse-engineered sync protocols and a self-hosted cloud: 11 tools</span><span class="entry-meta">2026-09</span></a></li>
      </ul>
    </section>
    <section class="section" aria-labelledby="skills-heading">
      <h2 id="skills-heading" class="section-label">Skills</h2>
      <ul class="entries">
        <li><div class="entry-static"><span class="entry-name">Languages</span><span class="entry-desc">Rust, C, C++, Python, TypeScript and JavaScript</span></div></li>
        <li><div class="entry-static"><span class="entry-name">Reverse engineering</span><span class="entry-desc">IDA Pro (IDAPython, IDAlib, plugins), binary and database formats, firmware</span></div></li>
        <li><div class="entry-static"><span class="entry-name">Systems</span><span class="entry-desc">Linux kernel (PCI hotplug, power management, Wi-Fi, audio), rustc MIR, LLVM IR</span></div></li>
        <li><div class="entry-static"><span class="entry-name">Security</span><span class="entry-desc">Vulnerability research, coordinated disclosure, OpenPGP and GnuPG internals</span></div></li>
      </ul>
    </section>
    <section class="section" aria-labelledby="education-heading">
      <h2 id="education-heading" class="section-label">Education</h2>
      <ul class="entries">
        <li><div class="entry-static"><span class="entry-name">University of Arizona</span><span class="entry-desc">Computer Science; left to work full time</span><span class="entry-meta">2023–2024</span></div></li>
        <li><div class="entry-static"><span class="entry-name">Pima JTED</span><span class="entry-desc">Cybersecurity and Artificial Intelligence, with dual enrollment at Pima Community College</span><span class="entry-meta">2021–2023</span></div></li>
      </ul>
    </section>
</div>
