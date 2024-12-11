use16
[org 0x7C00]

; %include "decompress.asm"


%macro sleep 2 ; sleep for cx:dx microseconds
    push dx
    mov ah, 0x86 ; sets AH register = 0x86, function to sleep
    mov cx, %1
    mov dx, %2
	; BIOS interrupt 0x15 (miscellaneous system services) calls the function in AH (0x86), which literally "wait"s for the time specified in CX:DX
	; https://en.wikipedia.org/wiki/BIOS_interrupt_call#Interrupt_table
    int 0x15
    pop dx
%endmacro

%macro beepfreq 0 ; sets the frequency of the speaker (the note) based on AX register
	; port 0x42 is the data register for PIT (Programmable Interval Timer) channel 2, which controls the frequency of the speaker
	; generates a square wave sound at the specified frequency (freq = 1193182 / divisor) where divisor = AH and AL
	out 0x42, al ; write low byte
	mov al, ah ; move high byte to al
	out 0x42, al ; write high byte
%endmacro

%macro beepon 0 ; turns on the speaker
	in al, 0x61 ; read the value from port 0x61 (speaker control register)
	or al, 00000011b ; enables bits 0 (speaker enable) and 1 (gate 2 enable)
	out 0x61, al ; write the modified value back to the port
%endmacro

%macro beepoff 0 ; turns off the speaker
	in al, 0x61 ; read the value from port 0x61 (speaker control register)
	and al, 11111100b ; disables bits 0 (speaker enable) and 1 (gate 2 enable)
	out 0x61, al ; write the modified value back to the port
%endmacro

startanimation:
	; writes 182 to port 43h, which configures the frequency of the system timer (prepares the speaker to play sound)
	; port 43h: this is the Programmable Interval Timer (PIT) Mode/Command Register, which controls the frequency of the system timer
	; value 182: Configures the PIT to set the divisor latch for channel 2, which controls the speaker.
	mov al, 182
	out 0x43, al
	
	; disables cursor blinking with BIOS interrupt 0x10, function 0x1003
	mov ax, 0x1003 ; sets AX register to 0x1003 (AH = 0x10, AL = 0x03)
	mov bl, 0
	int 0x10

	mov di, 0 ; clears destination index, which will be used to iterate over video memory

	mov dx, image + (2000 * frames) ; loads the memory address of part of the image data into DX
	
	mov cx, 0xB800 ; loads the base address of VGA text mode video memory (0xB800) into CX
	mov es, cx ; sets the extra segment register ES to the base address of video memory
	
	; clears screen
	mov ax, 0 ; sets AX register to 0 (blank character)
	mov cx, 2000 ; sets the loop counter to 2000, representing all 2000 character cells in VGA text mode (80x25 characters)
	rep stosw ; repeats stosw instruction (store word) CX times:
	; writes the value in AX (blank character) to video memory
    ; increments DI to point to the next character cell
	
	mov si, image + (2000 * frames) + 476 ; sets SI to point to the start of the message in the image data
	mov di, 0 ; resets the destination index (DI) for video memory operations
	
	beepon ; calls beepon macro, which turns on the speaker
	mov bl, 1 ; sets BL register to 1 (BL will used for note timing)
	
	startmsg:
		sleep 0x0, 0x6000 ; sleep before each loop, 0x6000 microseconds (24.576 ms)
		
		cmp si, image + (2000 * frames) + 476 + msglen ; compares SI to the end of the message
		jge note ; if SI is greater or equal to the end of the message, jump to note
		
		lodsb ; loads the next byte from SI into AL, increments SI
		mov ah, 0xf0 ; sets AH register to 0xf0 (attribute for message text)
		stosw ; stores the word (AL and AH) at DI, increments DI by 2 (word size)
		
		note:
			; decrements BL, and loops until BL is 0
			; this is used for timing notes (larger BL = longer delay)
			dec bl
			cmp bl, 0
			jne startmsg
			
			push si ; saves SI to the stack
			mov si, dx
			
			; loads the next word from SI into AX, and splits it into CX and AH
			lodsw
			mov cx, ax
			and ah, 0b00011111 ; bitwise AND to AH masking off all but the lower 5 bits
			
			beepfreq ; sets frequency based on AX register
			
			; shifts CH 5 bits to the right and then shifts 2 bits to the left, then writes to BL
			; i think this rounds BL (timing)
			shr ch, 5
			shl ch, 2
			mov bl, ch
			
			mov dx, si
			
			pop si ; restores SI from the stack
			
			cmp dx, image + (2000 * frames) + 26 * 2 ; compares DX to the end of the song
			jne startmsg
		
	; Set image address
	mov si, image
	mov di, 0
	
	mov ax, daddr
	mov ds, ax
	
	mov ax, 0xb800
	mov es, ax
	
	dec bl
	jmp transition
	
	wrimg:
		; Write character
		mov al, 220
		stosb
		
		; Write attributes
		lodsb
		stosb
		
		; Check if animation is done
		cmp si, image + (2000 * frames)
		je repeat
	
		; Check if the next frame is reached
		cmp di, 4000
		je nextframe
	
		; Repeat the loop
		jmp wrimg
	
	nextframe:
		sleep 0x1, 0x6000 ; Sleep some time
		
		transition:
		mov di, 0         ; Reset video memory address
		
		cmp dx, image+ (2000 * frames) + 476
		jne nextnote
		
		mov dx, image + (2000 * frames) + 26 * 2 ; Loop song
		
		nextnote:
			dec bl
			cmp bl, 0
			jne wrimg
			
			push si
			mov si, dx
			
			lodsw
			mov cx, ax
			and ah, 0b00011111
			
			beepfreq
			
			shr ch, 5
			mov bl, ch
			
			mov dx, si
			
			pop si
			jmp wrimg         ; Go back
	
	repeat:
		mov si, image
		jmp nextframe

daddr: equ 0x07e0 ; destination address
compressed: equ 0x0000 ; address of compressed data
image: equ 0x4000 ; address to store decompressed data
msglen: equ 76 ; length of message
frames: equ 3 ; number of frames in the animation

; pad rest of sector with 0s
; ($-$$) calculates size of program
times 510 - ($ - $$) db 0 ; up to 510 to save 2 bytes for signature
dw 0xAA55 ; boot signature - makes BIOS recognize as bootable

; while the bootloader is limited to 512 bytes, the rest of the sector can be used for data storage
comp: incbin "Data/compressed.bin" ; include compressed data file
compsize: equ $-comp ; calculate size of compressed data

times 8*1024 - ($ - $$) db 0 ; pad to 4KB