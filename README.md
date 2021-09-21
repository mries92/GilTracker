# GilTracker
This is a data visualization tool for ffxiv. It reads currency values from game memory and plots them over time. (Will add pretty pictures and stuff later)

## Platform Note
Currently, I only have builds for the windows version of the game. I plan to look into OSX support once I get the major features finished on windows.
However, I can't say for sure if it is even possible as I haven't worked with memory reading in OSX before.

## Build Requirements
- `Rust`
- `npm`

## Building
Run `npm install` to install the required packages. Then run `npm run tauri build`. The binary will be in the `src-tauri/target/release/` directory.