# Chipst8

A simple Chip8 emulator written by Rust and Tauri.

### Attention
In this project, I use Mutex in parking_lot library to get high performance. Otherwise, when user press a key, Chipst8 will freeze for a while.

### How to run
Init:
```
pnpm install
```

Debug:
```bash
pnpm tauri dev 
```

Bulid:
```bash
pnpm tauri build
```

### Input
|Keyboard||Chip8 Keypad|
|-:|:-:|:-|
|1 2 3 4| |1 2 3 C|
|Q W E R| --> |4 5 6 D|
|A S D F| |7 8 9 E|
|Z X C V| |A 0 B F|

'[': speed down the emulate speed.

']': speed up the emulate speed.

### Where to find Chip8 ROM
https://johnearnest.github.io/chip8Archive/