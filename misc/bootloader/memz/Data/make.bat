@REM cd image
@REM python png2bin.py frames\*.png image.bin
@REM cd ../song
@REM python midi2bin.py midi.mid song.bin
@REM cd ..
@REM type image\image.bin song\song.bin message.txt > data.bin
@REM compressor\compress.exe data.bin compressed.bin

py make.py "frames/*.png" midi.mid message.txt compressed.bin
@REM type output.bin message.txt

pause