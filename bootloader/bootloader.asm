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


start:
    mov si, 0x7C00
    mov ax, [si]          ; Text offset
    mov bx, [si + 4]      ; Text length
    mov cx, [si + 8]      ; MIDI offset
    mov dx, [si + 12]     ; MIDI size
    mov bp, [si + 16]     ; Image (frames) offset
    mov di, [si + 20]     ; Image (frames) size

    ; Set up video mode and clear screen
    mov ax, 0x1003
    mov bl, 0
    int 0x10

    mov di, 0
    mov cx, 0xB800
    mov es, cx
    mov ax, 0
    mov cx, 2000
    rep stosw

    ; Load and display text message
    mov si, ax
    add si, 0x7C00
    mov di, bx
    beepon
    call display_message

    ; Load and play MIDI data
    mov si, cx
    add si, 0x7C00
    mov di, dx
    call play_midi

    ; Load and display image frames
    mov si, bp
    add si, 0x7C00
    mov di, 0
    call render_frames

    ; Infinite loop to keep the bootloader running
    jmp $

display_message:
    mov bl, 1
    startmsg:
        sleep 0x0, 0x6000
        cmp di, 0
        je done_message
        lodsb
        cmp al, 0
        je done_message
        mov ah, 0x0E
        int 0x10
        dec di
        jmp startmsg
    done_message:
        ret

play_midi:
    ; Implement MIDI playback logic here
    ret

render_frames:
    mov cx, 4000
    render_frames_loop:
        lodsb
        stosb
        loop render_frames_loop
        call play_midi
        sub di, 4000
        cmp di, 0
        je render_done
        jmp render_frames_loop
    render_done:
        ret

daddr: equ 0x07e0
compressed: equ 0x0000
image: equ 0x4000
msglen: equ 76
frames: equ 3

times 510 - ($ - $$) db 0
dw 0xAA55

comp: incbin "Data/compressed.bin"
compsize: equ $-comp

times 8*1024 - ($ - $$) db 0