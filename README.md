# random-stuff

This repository is a collection of scripts, prototypes, and experiments that are:

- Too small for their own repository
- Unfinished or proof-of-concepts
- Things I just wanted to play around with or learn about

This README gives a brief summary of each project, sorted by programming language/engine (following the folder structure).

## Table of Contents

- [Rust](#rust)
- [Python](#python)
- [Godot](#godot)
- [Misc](#misc)

---

## Rust

### [color-reduction-clustering](./rust/color-reduction-clustering/)

A program that performs color quantization on images. It uses a multithreaded K-means clustering algorithm in 3D space (using RGB as axes) to drastically reduce an image's color palette while preserving its visual identity.

### [fft](./rust/fft/)

A terminal audio visualizer. It uses Fast Fourier Transform (FFT) to process audio data and display a real-time waveform and frequency analyzer in the terminal.

### [fluXis-bad-apple](./rust/fluXis-bad-apple/)

A script that converts the Bad Apple video into a map for the rhythm game FluXis. It uses FFmpeg to extract and downscale frames, and translates the pixel brightness into note placement and scroll velocity events to animate the video's frames on the playfield.

### [frame-merge](./rust/frame-merge/)

A WIP utility for blending frames together in a video.

### [letter-counter](./rust/letter-counter/)

A high-performance script that benchmarks counting letter frequencies in a text file. It started of as a school assignment, and I kept optimizing it to see how fast I could make it.

### [mandelbrot-rust](./rust/mandelbrot-rust/)

A port of my Python fractal generator to Rust. It generated high-resolution renders of the Mandelbrot set, using Rust's concurrency for faster calculations.

### [mania-bot](./rust/mania-bot/)

A bot that automaatically plays osu!mania. It captures the screen in real-time, detects notes approaching the receptors, and simulates key presses to play automatically.

### [minesweeper](./rust/minesweeper/)

A WIP terminal-based Minesweeper game.

### [minesweeper-solver](./rust/minesweeper-solver/)

An automated Minesweeper solver. It captures the screen to read the board state, and uses a logic algorithm to flag mines and reveal safe tiles. It falls back to random guessing when logically stuck. Features a cool live ASCII version of the board in the terminal to visualize the bot's "thought process".

### [rat-slime](./rust/rat-slime/)

A music player for the terminal, built with `ratatui` and `rodio`. It handles MP3 metadata, and features a playback queue, volume/speed controls, and playlist looping logic. No, I don't know why I named it that.

### [rust-anim](./rust/rust-anim/)

A terminal-based ambient animation of the sea, built with `ratatui`. It uses Perlin noise to generate dynamic waves and sky gradients that shift based on the time of day.

---

## Python

### [ascii-video-player](./python/ascii-video-player/)

A CLI and GUI tool that processes video files into high-resolution ASCII art animations. It analyzes any font's character density to map pixel brightness to specific characters.

### [fish-sim](./python/fish-sim/)

My attempt to rewrite my friend's fish simulation from Matlab to Python. It's a Boids flocking simulation that models fish behavior using zones of repulsion, orientation, and attraction. Uses `OpenCV` to render a video of the simulation.

### [GDI-PY](./python/fish-sim/)

A "joke malware" program. It uses the Windows GDI API to cause the screen to glitch and artifact, with payloads such as melting, tunneling, and inverting colors. Also has math-based ByteBeat audio to play along with it. Inspired by old joke malware, such as the MEMZ trojan.

### [mandelbrot-python](./python/mandelbrot-python/)

A script that generates high-resolution images of the Mandelbrot set fractal. Uses `numpy` for vectorized complex number calculations, and supports custom color gradients.

### [sorting-alg](./python/sorting-alg/)

A simple manual implementation of the Selection Sort algorithm to learn how it works.

---

## Godot

### [astral-eternity](./godot/astral-eternity/)

A demo that simulates metaball projectiles. Uses custom shaders for masking backgrounds inside the metaballs and rendering pixel perfect outlines.

### [non-euclidean](./godot/non-euclidean/)

An experiment in creating seamless, non-Euclidean portals using shaders and a first-person controller.

### [shader-test](./godot/shader-test/)

A playground project for testing or experimenting with various shaders and effects.

---

## Misc

### [bootloader](./misc/bootloader/)

A project for learning and exploring x86 bootloaders, inspired by the bootloader featured in the MEMZ trojan.

### [end-portal-wallpaper](./misc/end-portal-wallpaper/)

An animated wallpaper simulating the Minecraft End Portal effect, using a shader running on WebGL. [View on Wallpaper Engine](https://steamcommunity.com/sharedfiles/filedetails/?id=3185732456)

### [GDI-CS](./misc/GDI-CS/)

A C# port of the joke malware script.

### [image-corruption](./misc/image-corruption/)

A glitch art tool to corrupt images by prodding at the raw binary data to create visual artifacts.

### [nes-asm](./misc/nes-asm/)

A project for taking notes and learning NES programming in 6502 assembly.
