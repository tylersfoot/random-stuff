; iNES Header
  .inesprg 1   ; confirms the ROM has 1x 16KB bank of PRG (program) code
  .ineschr 1   ; confirms the ROM has 1x 8KB bank of CHR (graphics) data
  .inesmap 0   ; specifies Mapper 0 (NROM) - no bank swapping
  .inesmir 1   ; sets background mirroring to vertical, for horizontal scrolling


;;;;;;;;;;;;;;;


  .bank 0
  .org $C000
RESET:
  ; when the processor starts up, it will begin code execution here
  SEI          ; disable IRQs
  CLD          ; disable decimal mode
  LDX #$40
  STX $4017    ; disable APU frame IRQ
  LDX #$FF
  TXS          ; set up stack - sets stack pointer to $FF,
               ; the max value, which tells the CPU to grow downwards
  INX          ; now X = 0
  STX $2000    ; disable NMI
  STX $2001    ; disable rendering
  STX $4010    ; disable DMC IRQs

vblankwait1:   ; wait for vblank to make sure PPU is ready
  BIT $2002    ; checks bit 7 of the PPU status register
               ; the PPU sets it to 1 when a vblank period begins
  BPL vblankwait1  ; continue loop if vblank hasn't started (N flag from BIT = 0)

clrmem:
  ; cleans the NES's RAM, setting it to a known state
  ; stores #$00 to memory pages $0000 - $0700 (except for $0300)
  ; every loop increments the offset by 1, eventually
  ; reaching offset +#$0100, filling each page completely
  LDA #$00     
  STA $0000, x 
  STA $0100, x
  STA $0200, x
  STA $0400, x
  STA $0500, x
  STA $0600, x
  STA $0700, x
  LDA #$FE     ; sets sprite ram ($0300) to #$FE, moving all sprites off-screen 
  STA $0300, x
  INX          ; increment X - when X overflows from $FF to $00, it sets the zero flag (Z)
  BNE clrmem   ; continues loop until Z flag is set - this loops 256 times total

vblankwait2:   
  ; wait for vblank again - clrmem takes longer than a vblank period
  ; so this waits for the next vblank to ensure the PPU is ready
  BIT $2002
  BPL vblankwait2

LoadPalettes:
  ; write $3F00 address (the start of palette memory) to the
  ; PPU Address Register, so data sent starts at $3F00
  LDA $2002    ; read PPU status to reset the high/low latch
               ; the latch toggles setting first/last byte
  LDA #$3F
  STA $2006    ; write the high byte of $3F00 address
  LDA #$00
  STA $2006    ; write the low byte of $3F00 address
  LDX #$00     ; set X to 0 for LoadPalettesLoop

LoadPalettesLoop:
  ; copy the palette data (32 bytes) to the PPU
  ; palette memory holds 16 bytes for background ($3F00-$3F0F)
  ; and 16 bytes for sprites ($3F10-$3F1F)
  LDA palette, x   ; load byte from palette db based on index
  STA $2007        ; write to PPU
  INX              ; set index to next byte
  CPX #$20
  BNE LoadPalettesLoop  ; if x = $20 (32), all 32 bytes are copied, continue

  ; set attributes for sprite 0
  LDA #$80
  STA $0200        ; put sprite 0 in center ($80) of screen vertically
  STA $0203        ; put sprite 0 in center ($80) of screen horizontally
  LDA #$00
  STA $0201        ; tile number = 0 (use first tile in the pattern table)
  STA $0202        ; use first color palette, don't flip sprite

  LDA #%10000000   ; enable NMI, sprites from Pattern Table 0
  STA $2000

  LDA #%00010000   ; enable sprite rendering
  STA $2001

Forever:
  ; runs an infinite loop
  JMP Forever 
  
NMI: ; Non-Maskable Interrupt - beginning of VBlank period, pause code (saved in stack) and run:
  ; transfer sprites using DMA (Direct Memory Transfer)
  ; by copying a block of RAM from CPU memory to PPU sprite memory
  LDA #$00
  STA $2003    ; set the low byte (00) of the RAM address
  LDA #$02     ; $02 = page
  STA $4014    ; set the high byte (02) of the RAM address
               ; writing to $4014 triggers the DMA
  
  RTI          ; Return From Interrupt - return state from stack and resume code


;;;;;;;;;;;;;;  

  
  .bank 1
  .org $E000
palette: ; sets the color palette for the background and sprites
  .db $0F,$31,$32,$33,$0F,$35,$36,$37,$0F,$39,$3A,$3B,$0F,$3D,$3E,$0F
  .db $0F,$1C,$15,$14,$0F,$02,$38,$3C,$0F,$1C,$15,$14,$0F,$02,$38,$3C
  
  .org $FFFA   ; first of the three vectors starts here
  .dw NMI      ; when an NMI happens (once per frame if enabled) the processor will jump to the label NMI:
  .dw RESET    ; when the processor first turns on or is reset, it will jumpto the label RESET:
  .dw 0        ; external interrupt IRQ is not used in this code


;;;;;;;;;;;;;;  


  .bank 2
  .org $0000
  .incbin "mario.chr"   ; includes 8KB graphics file from SMB1