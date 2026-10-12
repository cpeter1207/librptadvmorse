# librptadvmorse

Portable 48 kHz Morse renderer for rpt_advanced. The ABI 1 descriptor in
`include/rptadvmorse/rptadvmorse.h` owns setup allocation, rendering, restart,
and destruction. The renderer keeps Morse alphabet, spacing, and fractional
sample timing; it calls the released `librptadvtone.so.1` oscillator only for
keyed waveform samples and phase resets. It has no Asterisk dependency.

Version `0.1.0-alpha.1` requires the published tone runtime and development
packages at Debian version `0.1.0~alpha1-1` or newer. The native callback
path performs no allocation, locks, logging, or I/O. `make test install-check`
checks the dynamic C descriptor, staged header/pkg-config files, and SONAME.
