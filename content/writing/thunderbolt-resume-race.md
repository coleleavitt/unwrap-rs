+++
title: My laptop kept waking up to a black screen, so I went and fixed it in the kernel
date: 2026-11-07
description: A Thunderbolt 5 resume hang on my ThinkPad, how I tracked it down to a race in system resume, and how an unrelated crash finally got me to send the fixes upstream.
+++

My laptop is a ThinkPad P16 Gen 3. It has a Thunderbolt 5 controller on its own PCIe link, an Intel JHL9580 ("Barlow Ridge"), and every so often it would wake from sleep to a black screen. Not every time. Just often enough to be annoying, and rare enough to be hard to catch.

When it did crash, the kernel log had this:

```
pcieport 0000:80:1b.4: Data Link Layer Link Active not set in 100 msec
pcieport 0000:80:1b.4: pciehp: Slot(25): Card not present
thunderbolt 0000:8a:00.0: RX ring 0 already stopped
...
Oops: general protection fault, kernel NULL pointer dereference 0x32b
RIP: 0010:tb_resume_noirq+0x3f/0x2e0 [thunderbolt]
Call Trace:
 tb_domain_resume_noirq+0x42/0x60 [thunderbolt]
 dpm_run_callback+0x4b/0x140
 device_resume_noirq+0x16d/0x2c0
 async_resume_noirq+0x21/0x30
```

The crash is at the bottom, but the story is in the first two lines.

## Digging in

The first line says the PCIe link to the Thunderbolt controller wasn't back up within 100 ms of waking. Fine, slow hardware happens. The second line is where it goes wrong: the hotplug driver, pciehp, decides the card is *not present*.

So I followed it. `pciehp_resume_noirq()` sees the devices under the port marked disconnected and concludes the device in the slot was **replaced**. To deal with that, it fakes a "presence changed" event with `pciehp_request()`, which wakes the hotplug IRQ thread through `irq_wake_thread()`.

That's the catch. During this part of resume, `suspend_device_irqs()` keeps real interrupts quiet, but `irq_wake_thread()` isn't an interrupt, so nothing holds it back. The IRQ thread starts removing the Thunderbolt controller right there, in the middle of the noirq phase.

And the noirq phase isn't serialized. The PM core resumes the devices below the port asynchronously and holds no `device_lock`. So on one CPU the removal path runs `nhi_pci_remove()` → `tb_domain_remove()` → `tb_stop()`, which sets `tb->root_switch` to `NULL`. On another CPU the Thunderbolt driver's own resume, `tb_resume_noirq()`, reaches for `tb->root_switch`. Boom.

Fun detail: before commit `e56249d8a68e` that pointer was left dangling instead of `NULL`, so older kernels had the same race as a use-after-free. And none of this is really Thunderbolt-specific. Any driver below a hotplug port can lose that race.

## What I changed

- **pciehp:** don't act on the faked event during noirq. Just record it, and wake the IRQ thread from `pciehp_system_resume()`, the normal `->resume()` step. By then every noirq callback is done, and `device_resume()` holds the `device_lock` that the removal path also needs, so they can't overlap anymore.
- **thunderbolt:** when the controller is already gone on resume, the driver sets `nhi->going_away`, but the software connection manager still tried to resume through dead rings. Every request failed with `-ESHUTDOWN` and spammed a WARN. Now it skips resume in that case, like the firmware connection manager already did.
- **USB:** the same scenario turned up a third bug. A USB controller that disappears mid-resume got disabled twice, and the PCI core complained. Now it only disables it if it's still enabled.

I wrote these in September and have been running them on my own kernel since.

## Getting them upstream

Here's the honest part: they sat on a local branch for a month.

What changed was an unrelated crash on October 18. The laptop panicked in memory-cgroup accounting and rebooted itself, because I boot with `panic=10`. The log survived in pstore, so the first job was reading that. But while I was in there, I went through all the patches I carry on my kernel, and asked about each one: is this a real bug, and does it belong upstream?

For these three the answer was yes. Before sending, I didn't want to rely on "it works on my machine". I walked the control flow through the kernel source with tree-sitter and libclang to check that every path really behaves the way each commit message says. Then I checked everything against the kernel's *Submitting patches* guide, and sent them the same day:

- [PATCH 0/2] PCI: pciehp / thunderbolt: Fix resume with a slow Thunderbolt host link ([lore](https://lore.kernel.org/all/20261018163750.130470-1-cole@unwrap.rs/))
- [PATCH] USB: hcd-pci: Do not disable an already disabled device on remove ([lore](https://lore.kernel.org/all/20261018164134.134106-1-cole@unwrap.rs/)), acked by Alan Stern

They're still under review, not merged yet. (The memcg crash is a story for another post.)

## What I'd tell myself

- **A slow link looks like an unplug.** "Not up in 100 ms" became "card not present", then "card replaced", then a removal. Every step makes sense on its own, and together they crash.
- **`irq_wake_thread()` isn't an interrupt.** If you count on `suspend_device_irqs()` to keep threaded handlers quiet during resume, that path gets around it.
- **Send the patch.** A fix on my branch helps one laptop. The same fix upstream helps everyone with this hardware.
