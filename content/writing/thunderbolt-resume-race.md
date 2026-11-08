+++
title: A Thunderbolt 5 resume hang: pciehp and the thunderbolt driver racing in the noirq phase
date: 2026-11-07
description: My laptop sometimes woke to a black screen. The cause was a hotplug removal running in the middle of system resume, unserialized against the driver it removed.
+++

My ThinkPad P16 Gen 3 has a discrete Thunderbolt 5 host controller, an Intel JHL9580 ("Barlow Ridge"), below PCIe Root Port `0000:80:1b.4`. Every so often, resume from suspend-to-idle ended in a black screen. The oops below killed a resume worker, so resume never finished, and the only way out was a forced reboot. The kernel writes oopses to persistent RAM (ramoops), so the log survived the reboot in pstore (`/sys/fs/pstore/dmesg-ramoops-*`):

```
pcieport 0000:80:1b.4: Data Link Layer Link Active not set in 100 msec
pcieport 0000:80:1b.4: pciehp: Slot(25): Card not present
thunderbolt 0000:8a:00.0: RX ring 0 already stopped
...
Oops: general protection fault, kernel NULL pointer dereference 0x32b
Workqueue: async async_run_entry_fn
RIP: 0010:tb_resume_noirq+0x3f/0x2e0 [thunderbolt]
Call Trace:
 tb_domain_resume_noirq+0x42/0x60 [thunderbolt]
 dpm_run_callback+0x4b/0x140
 device_resume_noirq+0x16d/0x2c0
 async_resume_noirq+0x21/0x30
 async_run_entry_fn+0x36/0x160
```

The first two lines are the whole story, once you follow them.

## What happens

1. The Root Port leaves D3cold, but the link to Barlow Ridge is not up within 100 ms. The PCI core marks the devices below it as disconnected.
2. `pciehp_resume_noirq()` concludes that the device in the slot was **replaced** and synthesizes a Presence Detect Changed event with `pciehp_request()`, which wakes the hotplug IRQ thread through `irq_wake_thread()`.
3. `suspend_device_irqs()` holds back real interrupts during this phase, but it does not stop `irq_wake_thread()`. So the IRQ thread starts tearing the slot down immediately, in the middle of the noirq phase.
4. In the noirq and early phases the PM core resumes the devices below the port asynchronously and holds no `device_lock`. Nothing serializes the removal against their `->resume_noirq()` callbacks.
5. On one CPU, the removal runs `nhi_pci_remove()` → `tb_domain_remove()` → `tb_stop()`, which sets `tb->root_switch` to `NULL`. On another, `nhi_resume_noirq()` → `tb_resume_noirq()` dereferences `tb->root_switch`.

Before commit `e56249d8a68e` ("thunderbolt: Set tb->root_switch to NULL when domain is stopped") the pointer was left dangling instead, so older kernels hit the same race as a use-after-free rather than a clean NULL dereference. And nothing here is specific to Thunderbolt: any driver below a hotplug port is exposed to the same race.

## The fixes

**pciehp: defer the replacement until `->resume()`.** `pciehp_resume_noirq()` now only records the synthesized event, and `pciehp_system_resume()` wakes the IRQ thread. By then every noirq and early callback has finished, and `device_resume()` holds the `device_lock` across each driver's `->resume()`, the same lock the removal path takes in `device_release_driver()`. It is also where `pciehp_check_presence()` already brings down slots that were vacated during sleep. The old devices are still marked disconnected in the noirq phase, so their drivers and any nested hotplug ports keep seeing them as gone. Runtime resume is unchanged.

**thunderbolt: don't resume a connection manager whose host is gone.** When the controller has vanished, `nhi_resume_noirq()` sets `nhi->going_away` and `tb_ring_start()` refuses to start the control rings. The software connection manager resumed anyway, so every control request failed with `-ESHUTDOWN` and `tb_cfg_read()` / `tb_cfg_write()` printed a WARN for each one:

```
tb_cfg_write: -108
WARNING: drivers/thunderbolt/ctl.c:1157 at tb_cfg_write+0x94/0xf0 [thunderbolt]
...
thunderbolt 0000:8a:00.0: 0: failed to restore TMU configuration
```

The firmware connection manager already checks the flag in `icm_complete()`. The software one now does the same and skips `tb_resume_noirq()`.

**USB: don't disable a disabled controller.** The same scenario surfaced a third bug. `suspend_common()` disables a USB host controller and `resume_common()` re-enables it. If the controller is hot-removed during resume before its `->resume()` runs, `usb_hcd_pci_remove()` disables it a second time and the PCI core warns, `disabling already-disabled device`. The fix only disables it if it is still enabled.

## Status

I posted all three on 2026-10-18, against v7.3-rc7:

- [PATCH 0/2] PCI: pciehp / thunderbolt: Fix resume with a slow Thunderbolt host link ([lore](https://lore.kernel.org/all/20261018163750.130470-1-cole@unwrap.rs/))
- [PATCH] USB: hcd-pci: Do not disable an already disabled device on remove ([lore](https://lore.kernel.org/all/20261018164134.134106-1-cole@unwrap.rs/)), acked by Alan Stern

They are still under review, not merged. I run them on this laptop every day.

## What I took from it

- **The noirq phase is not serialized against hotplug.** A removal that starts there races every driver below the port.
- **`irq_wake_thread()` is not an interrupt.** Code that relies on `suspend_device_irqs()` to keep threaded handlers quiet during resume has to treat it separately.
- **A slow link looks like an unplug.** "Link not up within 100 ms" became "the card is gone", then "the card was replaced", then a removal. Each step is reasonable on its own.
