# CHIP-8 [![Build Status](https://travis-ci.org/machinetech/chip8.svg)](https://travis-ci.org/machinetech/chip8)
A [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) and Super CHIP-8 (SCHIP) emulator written in the [Rust](https://www.rust-lang.org/) programming language. A few sample game screenshots below. The project uses the [MIT](https://github.com/machinetech/chip8/blob/master/LICENSE) license.

Brix
--- 

[![Brix](https://machinetech.github.io/chip8/images/brix.png)](https://www.youtube.com/watch?v=V3jA3SWWKOg)

There is also a short video clip of the game on [YouTube](https://www.youtube.com/watch?v=V3jA3SWWKOg).

Blinky
---
![blinky.png](https://machinetech.github.io/chip8/images/blinky.png "Blinky Title")

Space Invaders
---
![space_invaders_title.png](https://machinetech.github.io/chip8/images/space_invaders_title.png "Space Invaders Title")

![space_invaders.png](https://machinetech.github.io/chip8/images/space_invaders.png "Space Invaders")

Pong
---
![pong.png](https://machinetech.github.io/chip8/images/pong.png "Pong")

Tetris
---
![tetris.png](https://machinetech.github.io/chip8/images/tetris.png "Tetris")

H.Piper
---
![hpiper.png](https://machinetech.github.io/chip8/images/hpiper.png "HPiper")

Car
---
![car.png](https://machinetech.github.io/chip8/images/car.png "Car")

Super Trip
---
![super_trip.png](https://machinetech.github.io/chip8/images/super_trip.png "Super Trip")

Super Worm
---
![super_worm.png](https://machinetech.github.io/chip8/images/super_worm.png "Super Worm")

Ant
---
![ant.png](https://machinetech.github.io/chip8/images/ant.png "Ant")

## Requirements

### RUST
The emulator compiles against the master branch of Rust. See the Rust documentation for installation of the Rust binaries, including the Rust package manager Cargo.  

### SDL2
The emulator uses the cross platform media library [SDL2](https://www.libsdl.org/) for access to audio, keyboard and graphics hardware. Windows and Mac OSX binaries are available for [download](https://www.libsdl.org/download-2.0.php) from the SDL website. 

**Ubuntu**:  

```
sudo apt-get install libsdl2-dev
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"
```

**HomeBrew**:  

```
brew install sdl2  
export LIBRARY_PATH="${LIBRARY_PATH}:/opt/homebrew/lib"
```

## Running games

A few games are included in the roms folder. Many more are available on the internet.

```
cargo run roms/brix.ch8
```

## Keys
The original CHIP-8 specification had a 16 key hexadecimal keypad with the following layout:

| 1 | 2  | 3 | c |
| --- |---| ---| --- |
| 4 | 5  | 6 | d |
| 7 | 8  | 9 | e |
| a | 0  | b | f |

However, for the sake of convenience, the layout has been remapped onto a standard keyboard. Bear in mind that the documentation for ROMS found on the internet most likely will specify action keys according to the original mapping.

| 1 | 2  | 3 | 4 |
| --- |---| ---| --- |
| q | w  | e | r |
| a | s  | d | f |
| z | x  | c | v |

Below are some additional keypresses that are also not in the official specification:

| Enter or Return | Pause |
| :--------------- | ----- |
| Backspace or Delete | Reset |
| Esc | Exit |

## Code diagram
![pong.png](https://machinetech.github.io/chip8/images/code_diagram.jpeg "Code diagram")

## Reporting problems
If anything should go wrong, please report the issue [here](https://github.com/machinetech/chip8/issues) and I will look into it. Thanks!

## References
* [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)   
* [MASTERING CHIP-8 by Matthew Mikolay](http://mattmik.com/chip8.html)
* [SUPER-CHIP v1.1 specifications by Erik Bryntse] (http://devernay.free.fr/hacks/chip8/schip.txt)
* [CHIP8.DOC by David Winter] (http://devernay.free.fr/hacks/chip8/CHIP8.DOC)



