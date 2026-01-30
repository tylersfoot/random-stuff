# random-stuff

This repository is filled with scripts and projects that are:

- Too small for their own repository
- Unfinished
- Something I just wanted to play around with or test

This README gives a brief summary of each project, sorted by programming language/engine (following the folder structure).

## Rust

#### color-reduction-clustering

A program that takes an image and reduces its color pallete (color quantization) using a multithreaded K-means clustering algorithm in 3D space with RGB as the axes.

#### fft

A terminal audio visualizer that uses Fast Fourier Transform (FFT) to display a waveform and frequency analyzer.

#### fluXis-bad-apple

A script that converts the Bad Apple video into a FluXis map. It uses FFmpeg to extract and downscale the frames, and turns the pixel data into notes to render each frame. It uses scroll velocity to animate the frames.

#### frame-merge

TODO

#### letter-counter

A script that counts the number of each letter in a text file. I kept rewriting/improving it to see how fast I could make it. Originally a school project.

#### mandelbrot-rust

An attempt to rewrite my Python mandelbrot fractal generator in Rust.

#### mania-bot

A bot that plays osu!mania really good. It constantly captures the screen, detects if there are notes near the receptors, and simulates the key presses accordingly.

#### minesweeper

TODO

#### minesweeper-solver

An automated minesweeper solver. It reads the game board by capturing the screen, and uses colors to determine the cell states. It calculates which tiles are safe or have mines, falling back on random guesses when stuck. It automatically moves the mouse and clicks on cells to progess. Features a live ASCII representation of the board in the terminal while it runs.

#### rat-slime

A music player for the terminal, built with `ratatui` and `rodio`. It handles mp3 metadata, and features a playback queue, volume/speed controls, and playlist looping logic. No, I don't know why I named it that.

#### rust-anim

A terminal animation of the sea built with `ratatui`. Uses Perlin noise to animate waves, color gradients, and the sky.

## Python

#### ascii-video-player

A CLI and GUI tool that processes video files into ASCII art animations, rendered out to mp4 files. It uses custom font analysis that maps pixel brightness to character density.

#### fish-sim

My attempt to rewrite my friend's fish simulation from Matlab to Python. It's a Boids flocking simulation that models fish behavior using zones of repulsion, orientation, and attraction. Uses `OpenCV` to render a video of the simulation.

#### GDI-PY

A joke "malware" program. It uses the Windows GDI API to cause the screen to glitch and artifact, such as melting, tunneling, and inverting colors. Also has ByteBeat audio that goes along with it. Inspired by old joke malware, such as the MEMZ virus.

#### mandelbrot-python

A script that generates high-resolution images of the Mandelbrot set fractal. Uses `numpy` for vectorized complex number calculations, and supports custom color gradients.

#### sorting-alg

A simple sorting algorithm (selection sort).

## Godot

#### astral-eternity

A demo that simulates metaball projectiles, uses shaders to show a different background inside the metaballs, and a pixel outline shader.

#### non-euclidean

A project experimenting with making seamless portals in Godot with shaders, with a first-person controller.

#### shader-test

A playground project for testing or experimenting with various shaders.

## Misc

#### GDI-CS

A C# implementation of the joke malware, translated from the Python version.

#### bootloader

A project for learning x86 bootloaders written in ASM, inspired by the bootloader featured in the MEMZ virus.

#### end-portal-wallpaper

A WebGB animated wallpaper background, simulating the end portal effect from Minecraft. Can be found on [Wallpaper Engine](https://steamcommunity.com/sharedfiles/filedetails/?id=3185732456)!

#### image-corruption

A tool to corrupt images for glitch art by directly prodding at the raw binary data.

#### nes-asm

A project for taking notes and learning NES programming in 6502 assembly.
