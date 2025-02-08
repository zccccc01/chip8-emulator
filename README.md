# Rust implement CHIP-8 Emulator

Following in the footsteps of predecessors, I implemented a CHIP-8 emulator by learning from existing projects.

## CHIP-8 Emulator Core

- [learn from here](https://github.com/jerryshell/rsc8)
- [bilibili](https://www.bilibili.com/video/BV1HKzNYQEjM/?share_source=copy_web&vd_source=5f1982e0be55875e72626a13b28d317d)

## WASM

- [learn from here](https://github.com/aquova/chip8-book)

```bash
cd ./rsc8_wasm
cargo install wasm-pack
rustup target add wasm32-unknown-unknown
wasm-pack build --target web
mv ./pkg/rsc8_wasm_bg.wasm ../web
mv ./pkg/rsc8_wasm.js ../web
```

## Start server by Python

```bash
cd ./web
python -m http.server
```

## Keymap

```text
Keyboard     Keypad
--------------------
1 2 3 4      1 2 3 C
Q W E R  =>  4 5 6 D
A S D F      7 8 9 E
Z X C V      A 0 B F
```

Press `Esc` to exit

## Source

- [CHIP-8 8XY6](https://www.reddit.com/r/EmuDev/comments/72dunw/chip8_8xy6_help/)
- [CHIP-8 BCD](https://velvetcache.org/2024/01/31/chip-8-bcd/)
- [.ch file](https://github.com/loktar00/chip8)
- [chip8-roms](https://github.com/kripod/chip8-roms)
- [wikipedia CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
- [Cowgod's Chip-8](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#Dxyn)
- [CHIP-8](https://multigesture.net/articles/how-to-write-an-emulator-chip-8-interpreter/)
