# GilTracker
This is a data visualization tool for ffxiv. It reads currency values from game memory and plots them over time. (Will add pretty pictures and stuff later)

## Platform Note
Currently, I only have builds for the windows version of the game. I plan to add support for OSX once I get the major features finished on windows.

## Build Requirements
- `Rust`
- `npm`

## Building
Run `npm install` to install the required packages. Then run `npm run tauri build`. The binary will be in the `src-tauri/target/release/` directory.