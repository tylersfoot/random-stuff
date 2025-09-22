use16
[org 0x7C00]

    mov ax, 0xB800
    mov es, ax
    mov al, 182
    out 0x43, al
    in al, 0x61
    or al, 3
    out 0x61, al

loop_forever:
    xor di, di
    mov cx, 2000
fill_screen_loop:
    in al, 0x40
    mov ah, al
    mov al, 219
    stosw
    loop fill_screen_loop

    in al, 0x40
    out 0x42, al
    in al, 0x40
    out 0x42, al
    jmp loop_forever

times 510-($-$$) db 0
dw 0xAA55