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

LoadSprites:
  LDX #$00            ; start at 0
LoadSpritesLoop:
  ; load sprites into RAM
  LDA sprites, x      ; load data from address (sprites +  x)
  STA $0200, x        ; store into RAM address ($0200 + x)
  INX                 ; X = X + 1
  CPX #$20            ; Compare X to hex $20 (32)
  BNE LoadSpritesLoop ; Branch to LoadSpritesLoop if compare was Not Equal to zero
                      ; if compare was equal to 32, keep going down


  LDA #%10000000   ; enable NMI, sprites from Pattern Table 0
  STA $2000

  LDA #%00010000   ; enable sprite rendering
  STA $2001

Forever: ; runs an infinite loop
  JMP Forever


;;;;;;;;;;;;;; VBlank Period


NMI: ; Non-Maskable Interrupt - beginning of VBlank period, pause code (saved in stack) and run:
  ; transfer sprites using DMA (Direct Memory Transfer)
  ; by copying a block of RAM from CPU memory to PPU sprite memory
  LDA #$00
  STA $2003    ; set the low byte (00) of the RAM address
  LDA #$02     ; $02 = page
  STA $4014    ; set the high byte (02) of the RAM address
               ; writing to $4014 triggers the DMA

LatchController:
  LDA #$01
  STA $4016
  LDA #$00
  STA $4016    ; tell both controllers to latch buttons


ReadA: ; A button on controller 1
  LDA $4016
  AND #%00000001
  BEQ ReadADone

ReadADone:

ReadB: ; B button on controller 1
  LDA $4016
  AND #%00000001
  BEQ ReadBDone
ReadBDone:

ReadSelect: ; Select button on controller 1
  LDA $4016
  AND #%00000001
  BEQ ReadSelectDone
ReadSelectDone:

ReadStart: ; Start button on controller 1
  LDA $4016
  AND #%00000001
  BEQ ReadStartDone
ReadStartDone:

; Sprite Movement - for each sprite: 
; 1. load position LDA (X = $0203, Y = $0200)
; 2. set carry flag (add = CLC, sub = SEC)
; 3. add/sub (ADC or SBC)
; 4. save position STA (X = $0203, Y = $0200)

ReadUp: ; Up button on controller 1
  LDA $4016
  AND #%00000001
  BEQ ReadUpDone
  LDA $0200
  SEC
  SBC #$01
  STA $0200
  LDA $0204
  SEC
  SBC #$01
  STA $0204
  LDA $0208
  SEC
  SBC #$01
  STA $0208
  LDA $020C
  SEC
  SBC #$01
  STA $020C
ReadUpDone:

ReadDown: ; Down button on controller 1
  LDA $4016
  AND #%00000001
  BEQ ReadDownDone
  LDA $0200
  CLC
  ADC #$01
  STA $0200
  LDA $0204
  CLC
  ADC #$01
  STA $0204
  LDA $0208
  CLC
  ADC #$01
  STA $0208
  LDA $020C
  CLC
  ADC #$01
  STA $020C
ReadDownDone:

ReadLeft: ; Left button on controller 1
  LDA $4016
  AND #%00000001
  BEQ ReadLeftDone
  LDA $0203
  SEC
  SBC #$01
  STA $0203
  LDA $0207
  SEC
  SBC #$01
  STA $0207
  LDA $020B
  SEC
  SBC #$01
  STA $020B
  LDA $020F
  SEC
  SBC #$01
  STA $020F
ReadLeftDone:

ReadRight: ; Right button on controller 1
  LDA $4016
  AND #%00000001    ; only look at bit 0
  BEQ ReadRightDone ; branch to ReadRightDone if button is NOT pressed (0)
                    ; add instructions here to do something when button IS pressed (1)
  LDA $0203         ; load sprite X position
  CLC               ; make sure the carry flag is clear
  ADC #$01          ; A = A + 1
  STA $0203         ; save sprite X position
  LDA $0207
  CLC
  ADC #$01
  STA $0207
  LDA $020B
  CLC
  ADC #$01
  STA $020B
  LDA $020F
  CLC
  ADC #$01
  STA $020F
ReadRightDone:      ; handling this button is done


  RTI ; Return From Interrupt - return state from stack and resume code


;;;;;;;;;;;;;;  

  
  .bank 1
  .org $E000
palette: ; sets the color palette for the background and sprites
  .db $0F,$31,$32,$33,$0F,$35,$36,$37,$0F,$39,$3A,$3B,$0F,$3D,$3E,$0F
  .db $0F,$1C,$15,$14,$0F,$02,$38,$3C,$0F,$1C,$15,$14,$0F,$02,$38,$3C

sprites: ; sprites 0 to 3
    ; vert tile attr horiz
  .db $80, $32, $00, $80 ; sprite 0
  .db $80, $33, $00, $88 ; sprite 1
  .db $88, $34, $00, $80 ; sprite 2
  .db $88, $35, $00, $88 ; sprite 3
  
  .org $FFFA   ; first of the three vectors starts here
  .dw NMI      ; when an NMI happens (once per frame if enabled) the processor will jump to the label NMI:
  .dw RESET    ; when the processor first turns on or is reset, it will jumpto the label RESET:
  .dw 0        ; external interrupt IRQ is not used in this code


;;;;;;;;;;;;;;  


  .bank 2
  .org $0000
  .incbin "mario.chr"   ; includes 8KB graphics file from SMB1