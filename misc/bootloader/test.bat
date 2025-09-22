@REM @echo off
@REM nasm -o disk.img bootloader.asm
@REM qemu-system-x86_64 -drive format=raw,file=disk.img
@REM pause

@echo off
nasm -o disk.img bootloader.asm
qemu-system-x86_64 -audiodev dsound,id=snd0 -machine pcspk-audiodev=snd0 -drive format=raw,file=disk.img
pause