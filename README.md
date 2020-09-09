# RustSBI

RISC-V Supervisor Binary Interface implementation in Rust; runs on M-mode.

## Features

- Functional operating system runtime
- Adapted to RISC-V SBI specification v0.2
- Good support for unix-like operating systems
- Written in Rust
- Alternative to OpenSBI with most of its function
- Supports QEMU emulator (priv. spec v1.11)
- Backward compatible to privileged spec v1.9
- Supports Kendryte K210 with MMU and S-Mode

## Talks and documents

This project is originally a part of rCore Summer of Code 2020 activities, now it is
capable of running rCore-Tutorial and other OS kernels on wide supported RISC-V devices.

Blog article (Chinese): [Here](https://github.com/luojia65/rcore-os-blog/blob/master/source/_posts/os-report-final-luojia65.md)

Slides (Chinese): [Here](https://github.com/luojia65/DailySchedule/blob/master/Rust%E8%AF%AD%E8%A8%80%E4%B8%8ERISC-V%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F.pdf)
