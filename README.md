# CHIP-8 [![Build Status](https://travis-ci.org/wm9/chip8.svg)](https://travis-ci.org/wm9/chip8)
A [CHIP-8](http://en.wikipedia.org/wiki/CHIP-8) emulator written in the [Rust](http://www.rust-lang.org/) programming language. A few sample game screenshots below. The project uses the [MIT](https://github.com/wm9/chip8/blob/master/LICENSE) license. 

Brix
--- 

[![Brix](http://wm9.github.io/chip8/images/brix.png)](http://www.youtube.com/watch?v=V3jA3SWWKOg)

There is also a short video clip of the game on [YouTube](http://www.youtube.com/watch?v=V3jA3SWWKOg).

Space Invaders
---
![space_invaders_title.png](http://wm9.github.io/chip8/images/space_invaders_title.png "Space Invaders Title")

![space_invaders.png](http://wm9.github.io/chip8/images/space_invaders.png "Space Invaders")

Pong
---
![pong.png](http://wm9.github.io/chip8/images/pong.png "Pong")

Tetris
---
![tetris.png](http://wm9.github.io/chip8/images/tetris.png "Tetris")

## Requirements

### RUST
The emulator compiles against the master branch of Rust. See the section in the official Rust Book for [installing](http://doc.rust-lang.org/nightly/book/installing-rust.html) the Rust binaries, including the Rust package manager Cargo. 

### SDL2
The emulator uses the cross platform media library [SDL2](https://www.libsdl.org/) for access to audio, keyboard and graphics hardware. Windows and Mac OSX binaries are available for [download](https://www.libsdl.org/download-2.0.php) from the SDL website. 

**Ubuntu**:  

```
sudo apt-get install libsdl2-dev
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"
```
**MacPorts**:  

```
sudo port install libsdl2  
export LIBRARY_PATH="${LIBRARY_PATH}:/opt/local/lib"
```

**HomeBrew**:  

```
brew install sdl2  
export LIBRARY_PATH="${LIBRARY_PATH}:/usr/local/lib"
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
![pong.png](http://wm9.github.io/chip8/images/code_diagram.jpeg "Code diagram")

## Reporting problems
If anything should go wrong, please report the issue [here](https://github.com/wm9/chip8/issues) and I will look into it. Thanks!

## References
* [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)   
* [MASTERING CHIP-8 by Matthew Mikolay](http://mattmik.com/chip8.html)



