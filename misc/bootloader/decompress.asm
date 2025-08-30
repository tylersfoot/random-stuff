; set ES, DS, and BX to the address of the destination buffer
; needed because lodsb and stosb use DS and ES by default
mov bx, daddr
mov es, bx ;
mov ds, bx ; 

; reads 4 sectors from the disk into the buffer at the address of compressed (defined in bootloader.asm)
mov ax, 0x0204 ; sets AX register to 0x0204 (AH = 0x02, AL = 0x04), 0x02 function reads sectors, 0x04 specifies 4 sectors to read
mov cx, 0x0002 ; CH = 0x00: cylinder number; CL = 0x02: sector number
mov dh, 0 ; the head number
mov bx, compressed ; the address of 'compressed' to read the sectors into
int 13h ; BIOS interrupt call to read sectors

; clears AX BX CX DX registers to 0
xor ax, ax
mov bx, ax
mov cx, ax
mov dx, ax

mov si, compressed ; sets source index SI to the address of compressed data
mov di, image ; sets destination index DI to the address of where decompressed data will be stored 

; the following section implements a custom decompression algorithm
; each command in the compressed data is interpreted as either:
; newdata - to copy directly to the output
; olddata - to copy previously decompressed data from a sliding window (back-referencing)
readcommand:
	lodsb ; loads next byte into AL from SI address, increments SI
	cmp si, compressed+compsize	; compares SI to compressed + compsize (end of compressed data)
	jae startanimation ; if at the end, jump to startanimation
	
    ; compares the command byte (AL) to 128:
    ; if AL >= 128 (highest bit is set), jump to newdata, else jump to olddata
	cmp al, 128
	jae newdata
	jmp olddata
	
newdata:
	and al, 127 ; extracts the lower 7 bits of AL, determines the length of the new data to copy
	mov cl, al ; sets CL register to AL (length of new data)
	
	newnextbyte:
		lodsb ; loads next byte into AL from SI address, increments SI
		stosb ; stores AL into DI address, increments DI

        ; decrements CL once every loop, until cl = -1 (end of new data)
		dec cl
		cmp cl, -1
		jne newnextbyte
		
		jmp readcommand ; jumps back to readcommand loop
		
olddata:
	mov ah, al ; copies AL to AH (command byte)
	lodsb ; loads next byte into AL from SI address, increments SI
	
	mov bx, ax ; copies AL to BX (offset to back-reference previously decompressed data)
	lodsb
	
	mov dx, si
	mov si, bx
	add si, image ; adds BX to image address to get the address of the back-referenced data
	mov cl, al ; ; sets CL register to AL (length of old data)
	
	oldnextbyte:
		lodsb ; loads next byte into AL from SI address, increments SI
		stosb ; stores AL into DI address, increments DI
		
        ; decrements CL once every loop, until cl = 0 (end of old data)
		dec cl
		cmp cl, 0
		jne oldnextbyte
		
		mov si, dx
		jmp readcommand ; jumps back to readcommand loop