# open-hardware-controller
Improved asusctl+rogcc

Evolution of asusd+asusctl, written in Rust with async code and the latest zbus,
relying on kernel interfaces instead of raw userspace hardware access.

## Roadmap
- LEDs through the kernel dynamic LED interface (all supported devices)
- Firmware attributes via fwupd, using a manually whitelisted/classified set
- Power management: amd-pstate/intel-pstate EPP, powercap, fan curves, asus-armoury
- Replace ppd, powerd, thermald and PowerStation; expose ppd/tuned D-Bus interfaces
- Named power profiles that can be saved/restored (implemented in `src/lib.rs`),
  falling back to defaults when settings are externally managed
