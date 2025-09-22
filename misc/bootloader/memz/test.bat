@echo off
nasm -o disk.img kernel.asm
qemu-system-x86_64 -drive format=raw,file=disk.img
pause