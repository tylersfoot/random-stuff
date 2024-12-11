@echo off
nasm -o disk.img bootloader.asm
qemu-system-x86_64 -drive format=raw,file=disk.img
pause