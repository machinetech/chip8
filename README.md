# chip8
A chip8 emulator written in the Rust programming language. A few sample game screenshots below.

Brix
--- 
![brix.png](http://wm9.github.io/chip8/images/brix.png "Brix")

Space Invaders
---
![space_invaders.png](http://wm9.github.io/chip8/images/space_invaders.png "Space Invaders")

Pong
---
![pong.png](http://wm9.github.io/chip8/images/pong.png "Pong")

Tetris
---
![tetris.png](http://wm9.github.io/chip8/images/tetris.png "Tetris")

## Requirements
The windowing system was built with SDL2. Windows and Mac OSX binaries are available for [download](https://www.libsdl.org/download-2.0.php) from the SDL website. 

**Ubuntu**:  
sudo apt-get install libsdl2-dev

**MacPorts**:  
sudo port install libsdl2  
export LIBRARY\_PATH="${LIBRARY\_PATH}:/opt/local/lib"

**HomeBrew**:  
brew install sdl2  
export LIBRARY\_PATH="${LIBRARY\_PATH}:/usr/local/lib"