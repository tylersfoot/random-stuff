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
  CPX #$10            ; Compare X to hex $10 (16)
  BNE LoadSpritesLoop ; Branch to LoadSpritesLoop if compare was Not Equal to zero
                      ; if compare was equal to 16, keep going down

LoadBackground:
  LDA $2002             ; read PPU status to reset the high/low latch
  LDA #$20
  STA $2006             ; write the high byte of $2000 address
  LDA #$00
  STA $2006             ; write the low byte of $2000 address
  LDX #$00              ; start out at 0
LoadBackgroundLoop:
  LDA background, x     ; load data from address (background + the value in x)
  STA $2007             ; write to PPU
  INX                   ; X = X + 1
  CPX #$FF              ; Compare X to hex $80, decimal 128 - copying 128 bytes
  BNE LoadBackgroundLoop  ; Branch to LoadBackgroundLoop if compare was Not Equal to zero
                        ; if compare was equal to 128, keep going down
LoadBackgroundLoop2:
  LDA background2, x
  STA $2007
  INX
  CPX #$FF
  BNE LoadBackgroundLoop2
LoadBackgroundLoop3:
  LDA background3, x
  STA $2007
  INX
  CPX #$FF
  BNE LoadBackgroundLoop3
LoadBackgroundLoop4:
  LDA background4, x
  STA $2007
  INX
  CPX #$BF
  BNE LoadBackgroundLoop4

LoadAttribute:
  LDA $2002             ; read PPU status to reset the high/low latch
  LDA #$23
  STA $2006             ; write the high byte of $23C0 address
  LDA #$C0
  STA $2006             ; write the low byte of $23C0 address
  LDX #$00              ; start out at 0
LoadAttributeLoop:
  LDA attribute, x      ; load data from address (attribute + the value in x)
  STA $2007             ; write to PPU
  INX                   ; X = X + 1
  ; CPX #$08              ; Compare X to hex $08, decimal 8 - copying 8 bytes
  CPX #$40
  BNE LoadAttributeLoop


  LDA #%10010000   ; enable NMI, sprites from Pattern Table 0, background from Pattern Table 1
  STA $2000

  LDA #%00011110   ; enable sprites, enable background, no clipping on left side
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


  ; this is the PPU clean up section, so rendering the next frame starts properly
  LDA #%10010000   ; enable NMI, sprites from Pattern Table 0, background from Pattern Table 1
  STA $2000
  LDA #%00011110   ; enable sprites, enable background, no clipping on left side
  STA $2001
  LDA #$00         ; tell the ppu there is no background scrolling
  STA $2005
  STA $2005

  RTI ; Return From Interrupt - return state from stack and resume code


;;;;;;;;;;;;;;  

  
  .bank 1
  .org $E000
palette: ; selects the color palette for the background and sprites
  .db $22,$29,$1A,$0F,  $22,$36,$17,$0F,  $22,$30,$21,$0F,  $22,$27,$17,$0F ; background palette
  .db $22,$1C,$15,$14,  $22,$02,$38,$3C,  $22,$1C,$15,$14,  $22,$02,$38,$3C ; sprite palette

sprites: ; sprites 0 to 3
    ; vert tile attr horiz
  .db $80, $32, $00, $80 ; sprite 0
  .db $80, $33, $00, $88 ; sprite 1
  .db $88, $34, $00, $80 ; sprite 2
  .db $88, $35, $00, $88 ; sprite 3

background: ; nametable for background tiles
  ; .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ; row 1
  ; .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ; all sky ($24 = sky)
  ; .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ; row 2
  ; .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24  ; all sky
  ; .db $24,$24,$24,$24,$45,$45,$24,$24,$45,$45,$45,$45,$45,$45,$24,$24  ; row 3
  ; .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$53,$54,$24,$24  ; some brick tops
  ; .db $24,$24,$24,$24,$47,$47,$24,$24,$47,$47,$47,$47,$47,$47,$24,$24  ; row 4
  ; .db $24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$24,$55,$56,$24,$24  ; brick bottoms
  
  .db $E8,$33,$76,$4F,$89,$6E,$F4,$09,$5B,$48,$32,$26,$84,$C5,$0E,$2B
  .db $4D,$40,$34,$BD,$48,$4F,$AF,$5D,$07,$E9,$1C,$8C,$35,$B0,$CA,$54
  .db $A9,$2C,$14,$F3,$93,$07,$F0,$82,$B7,$71,$56,$CF,$57,$C8,$9E,$90
  .db $4F,$70,$CC,$DC,$F3,$B9,$73,$43,$B5,$E2,$E2,$EB,$6A,$1F,$07,$BD
  .db $58,$E1,$E5,$9C,$8D,$98,$54,$DF,$A1,$8D,$E3,$25,$86,$F6,$2D,$00
  .db $81,$36,$00,$51,$BE,$DE,$E8,$73,$83,$52,$FB,$08,$74,$55,$BB,$61
  .db $57,$38,$7A,$92,$95,$15,$ED,$2D,$A5,$5A,$02,$30,$F3,$35,$36,$D2
  .db $28,$A1,$19,$05,$55,$69,$BD,$C5,$E5,$14,$8D,$B2,$35,$63,$DB,$70

  .db $7F,$6D,$5E,$47,$3A,$1C,$02,$0F,$AC,$09,$CF,$78,$85,$C3,$16,$F9
  .db $56,$5E,$42,$55,$5A,$20,$9F,$3E,$F2,$2C,$E9,$51,$2C,$0D,$15,$02
  .db $54,$89,$1A,$EB,$98,$F5,$D1,$39,$E6,$7C,$1C,$68,$F0,$DF,$42,$E7
  .db $37,$74,$47,$31,$F4,$6E,$49,$F6,$3B,$D8,$F8,$30,$EF,$B0,$7D,$93
  .db $9F,$16,$63,$28,$B4,$01,$42,$67,$D6,$B4,$F8,$8F,$F8,$68,$73,$89
  .db $24,$5B,$3A,$D6,$24,$B6,$77,$A8,$DC,$8A,$E2,$EE,$C8,$AB,$52,$EE
  .db $60,$98,$0A,$5C,$F3,$C2,$E1,$CD,$58,$93,$30,$28,$E5,$AC,$F2,$9F
  .db $A9,$AB,$EF,$E0,$9E,$F5,$1A,$D8,$78,$BC,$3E,$DC,$B8,$BE,$88,$7F

background2:
  .db $A6,$51,$8B,$F0,$16,$68,$2A,$E8,$6C,$73,$B0,$D6,$EC,$24,$ED,$EB
  .db $5F,$48,$D9,$B3,$50,$20,$83,$AC,$AF,$F4,$33,$5C,$18,$C2,$72,$D1
  .db $5F,$50,$FD,$D1,$9D,$46,$25,$0E,$E9,$71,$2A,$74,$C0,$51,$6D,$44
  .db $7C,$F8,$FA,$49,$01,$05,$62,$E8,$C7,$84,$AA,$A7,$65,$26,$8A,$1F
  .db $F3,$DC,$68,$B0,$84,$FC,$4D,$B7,$FE,$0B,$59,$8B,$5F,$C3,$B2,$53
  .db $AF,$C8,$9C,$8A,$CD,$A8,$BF,$36,$75,$8E,$52,$08,$6F,$E1,$56,$59
  .db $02,$22,$3E,$8F,$12,$02,$39,$41,$16,$B6,$3E,$6F,$12,$B5,$E3,$74
  .db $92,$6E,$03,$9B,$BB,$3C,$82,$F5,$D8,$2B,$8D,$DB,$91,$D9,$94,$6E

  .db $7F,$6A,$FC,$F5,$E2,$7F,$06,$A8,$07,$C3,$5A,$64,$80,$22,$A9,$6D
  .db $E3,$C2,$42,$60,$F4,$33,$51,$75,$3D,$D8,$53,$09,$C6,$D5,$B9,$52
  .db $FE,$6F,$7A,$AB,$D3,$16,$EF,$44,$93,$3F,$B3,$86,$11,$EE,$B2,$F4
  .db $D0,$76,$83,$1E,$5B,$8A,$EC,$4C,$85,$B7,$7A,$76,$ED,$8C,$C4,$08
  .db $78,$0C,$73,$AF,$6C,$B0,$E6,$26,$FE,$7C,$B8,$FE,$F5,$F1,$0B,$23
  .db $F3,$9B,$92,$BF,$C9,$2E,$CB,$A3,$6B,$22,$E0,$05,$0E,$C6,$D2,$64
  .db $02,$64,$0D,$B1,$36,$5F,$00,$30,$BF,$8A,$6B,$D6,$96,$1D,$14,$FE
  .db $05,$32,$74,$05,$47,$41,$B1,$1A,$72,$51,$40,$D8,$8E,$A2,$CF,$BF

background3:
  .db $D3,$5D,$FE,$64,$28,$C5,$81,$F5,$B5,$74,$BE,$41,$66,$67,$AF,$3D
  .db $AA,$7D,$C1,$E6,$D1,$2A,$7A,$FA,$0D,$16,$FE,$35,$33,$BA,$65,$99
  .db $36,$90,$6A,$3D,$CA,$09,$96,$25,$A1,$F0,$3F,$D8,$D0,$62,$18,$0F
  .db $A9,$8C,$48,$96,$EF,$39,$A5,$7D,$76,$96,$C5,$D0,$97,$25,$23,$5C
  .db $2B,$F6,$05,$CB,$F2,$82,$06,$77,$1D,$7A,$01,$89,$88,$F8,$1D,$10
  .db $1F,$AE,$EE,$77,$A2,$4C,$AC,$EC,$34,$DA,$2B,$55,$A4,$F5,$4C,$FE
  .db $51,$73,$AB,$9F,$99,$73,$50,$E7,$69,$73,$36,$D3,$B4,$33,$9F,$5A
  .db $84,$44,$27,$8C,$4C,$DA,$C4,$9A,$16,$07,$CC,$91,$DE,$80,$25,$70

  .db $93,$1E,$73,$AA,$C7,$65,$B7,$E3,$C6,$A4,$C3,$1A,$C6,$84,$0E,$92
  .db $F7,$F7,$EB,$61,$F2,$40,$03,$71,$AF,$A0,$77,$66,$31,$E7,$FF,$7E
  .db $58,$31,$5E,$2D,$16,$94,$A7,$A1,$7D,$55,$22,$D8,$9F,$2C,$C8,$58
  .db $00,$BC,$AC,$9E,$B7,$A3,$12,$15,$D6,$8A,$CC,$1E,$33,$B0,$F9,$02
  .db $1F,$B8,$B4,$C8,$41,$1E,$B4,$9A,$55,$99,$E1,$91,$C2,$68,$F4,$86
  .db $69,$DA,$C1,$D9,$C1,$7A,$5A,$D9,$33,$7E,$31,$CF,$86,$C4,$94,$CC
  .db $04,$1E,$07,$DD,$C9,$07,$EF,$8A,$C6,$B0,$FE,$7E,$74,$E2,$B6,$5A
  .db $87,$47,$6F,$53,$A5,$B1,$ED,$13,$A6,$4F,$08,$62,$72,$F2,$79,$CD

background4:
  .db $1B,$61,$90,$48,$C2,$E8,$58,$99,$6D,$0B,$77,$37,$61,$AC,$40,$6D
  .db $85,$9D,$73,$C5,$B8,$CB,$E6,$7F,$E5,$85,$4D,$E9,$EA,$E0,$4B,$87
  .db $29,$8D,$68,$7B,$07,$B2,$05,$91,$67,$B3,$AA,$65,$83,$A5,$04,$BD
  .db $4A,$B0,$1D,$A1,$A9,$68,$8A,$08,$38,$CD,$A3,$7C,$A9,$66,$00,$4A
  .db $7F,$49,$F3,$78,$1D,$C6,$99,$00,$85,$D4,$66,$48,$88,$18,$86,$E7
  .db $17,$D9,$04,$B2,$15,$67,$80,$52,$80,$6D,$38,$B4,$05,$C0,$46,$F4
  .db $07,$84,$2A,$BB,$9E,$AA,$04,$7B,$2A,$E1,$13,$0E,$89,$6F,$C5,$78
  .db $A3,$CF,$46,$95,$7C,$E8,$45,$F1,$FD,$D1,$C4,$20,$82,$6A,$81,$9F

  .db $47,$BA,$E7,$81,$25,$3E,$22,$D1,$A3,$D5,$BB,$A4,$07,$54,$1E,$00
  .db $55,$73,$94,$FE,$90,$FB,$B8,$B8,$E1,$86,$40,$A7,$AF,$70,$CE,$25
  .db $FF,$FC,$89,$C7,$2F,$19,$51,$AB,$AE,$30,$E8,$57,$04,$3A,$C9,$E5
  .db $8C,$5D,$FC,$32,$9C,$F4,$AE,$1D,$94,$0E,$26,$DD,$4F,$F5,$FD,$1E
  ; .db $36,$BB,$6B,$4A,$F9,$1E,$87,$BC,$67,$7B,$BE,$F3,$DE,$54,$8D,$EB
  ; .db $86,$B2,$95,$1A,$97,$62,$08,$4B,$44,$76,$03,$C4,$87,$5A,$69,$A9
  ; .db $23,$EA,$00,$7E,$2C,$F8,$01,$44,$09,$D5,$EC,$FD,$A9,$79,$48,$F5
  ; .db $CF,$55,$38,$89,$8B,$45,$E4,$96,$27,$07,$79,$C0,$FD,$75,$1B,$7E
  ; this is the end of the nametable size (960 bytes, 256+256+256+192, or 64*15)










; 1024 bytes of random AI data
; .db $E8,$33,$76,$4F,$89,$6E,$F4,$09,$5B,$48,$32,$26,$84,$C5,$0E,$2B
; .db $4D,$40,$34,$BD,$48,$4F,$AF,$5D,$07,$E9,$1C,$8C,$35,$B0,$CA,$54
; .db $A9,$2C,$14,$F3,$93,$07,$F0,$82,$B7,$71,$56,$CF,$57,$C8,$9E,$90
; .db $4F,$70,$CC,$DC,$F3,$B9,$73,$43,$B5,$E2,$E2,$EB,$6A,$1F,$07,$BD
; .db $58,$E1,$E5,$9C,$8D,$98,$54,$DF,$A1,$8D,$E3,$25,$86,$F6,$2D,$00
; .db $81,$36,$00,$51,$BE,$DE,$E8,$73,$83,$52,$FB,$08,$74,$55,$BB,$61
; .db $57,$38,$7A,$92,$95,$15,$ED,$2D,$A5,$5A,$02,$30,$F3,$35,$36,$D2
; .db $28,$A1,$19,$05,$55,$69,$BD,$C5,$E5,$14,$8D,$B2,$35,$63,$DB,$70
; .db $7F,$6D,$5E,$47,$3A,$1C,$02,$0F,$AC,$09,$CF,$78,$85,$C3,$16,$F9
; .db $56,$5E,$42,$55,$5A,$20,$9F,$3E,$F2,$2C,$E9,$51,$2C,$0D,$15,$02
; .db $54,$89,$1A,$EB,$98,$F5,$D1,$39,$E6,$7C,$1C,$68,$F0,$DF,$42,$E7
; .db $37,$74,$47,$31,$F4,$6E,$49,$F6,$3B,$D8,$F8,$30,$EF,$B0,$7D,$93
; .db $9F,$16,$63,$28,$B4,$01,$42,$67,$D6,$B4,$F8,$8F,$F8,$68,$73,$89
; .db $24,$5B,$3A,$D6,$24,$B6,$77,$A8,$DC,$8A,$E2,$EE,$C8,$AB,$52,$EE
; .db $60,$98,$0A,$5C,$F3,$C2,$E1,$CD,$58,$93,$30,$28,$E5,$AC,$F2,$9F
; .db $A9,$AB,$EF,$E0,$9E,$F5,$1A,$D8,$78,$BC,$3E,$DC,$B8,$BE,$88,$7F
; .db $A6,$51,$8B,$F0,$16,$68,$2A,$E8,$6C,$73,$B0,$D6,$EC,$24,$ED,$EB
; .db $5F,$48,$D9,$B3,$50,$20,$83,$AC,$AF,$F4,$33,$5C,$18,$C2,$72,$D1
; .db $5F,$50,$FD,$D1,$9D,$46,$25,$0E,$E9,$71,$2A,$74,$C0,$51,$6D,$44
; .db $7C,$F8,$FA,$49,$01,$05,$62,$E8,$C7,$84,$AA,$A7,$65,$26,$8A,$1F
; .db $F3,$DC,$68,$B0,$84,$FC,$4D,$B7,$FE,$0B,$59,$8B,$5F,$C3,$B2,$53
; .db $AF,$C8,$9C,$8A,$CD,$A8,$BF,$36,$75,$8E,$52,$08,$6F,$E1,$56,$59
; .db $02,$22,$3E,$8F,$12,$02,$39,$41,$16,$B6,$3E,$6F,$12,$B5,$E3,$74
; .db $92,$6E,$03,$9B,$BB,$3C,$82,$F5,$D8,$2B,$8D,$DB,$91,$D9,$94,$6E
; .db $7F,$6A,$FC,$F5,$E2,$7F,$06,$A8,$07,$C3,$5A,$64,$80,$22,$A9,$6D
; .db $E3,$C2,$42,$60,$F4,$33,$51,$75,$3D,$D8,$53,$09,$C6,$D5,$B9,$52
; .db $FE,$6F,$7A,$AB,$D3,$16,$EF,$44,$93,$3F,$B3,$86,$11,$EE,$B2,$F4
; .db $D0,$76,$83,$1E,$5B,$8A,$EC,$4C,$85,$B7,$7A,$76,$ED,$8C,$C4,$08
; .db $78,$0C,$73,$AF,$6C,$B0,$E6,$26,$FE,$7C,$B8,$FE,$F5,$F1,$0B,$23
; .db $F3,$9B,$92,$BF,$C9,$2E,$CB,$A3,$6B,$22,$E0,$05,$0E,$C6,$D2,$64
; .db $02,$64,$0D,$B1,$36,$5F,$00,$30,$BF,$8A,$6B,$D6,$96,$1D,$14,$FE
; .db $05,$32,$74,$05,$47,$41,$B1,$1A,$72,$51,$40,$D8,$8E,$A2,$CF,$BF
; .db $D3,$5D,$FE,$64,$28,$C5,$81,$F5,$B5,$74,$BE,$41,$66,$67,$AF,$3D
; .db $AA,$7D,$C1,$E6,$D1,$2A,$7A,$FA,$0D,$16,$FE,$35,$33,$BA,$65,$99
; .db $36,$90,$6A,$3D,$CA,$09,$96,$25,$A1,$F0,$3F,$D8,$D0,$62,$18,$0F
; .db $A9,$8C,$48,$96,$EF,$39,$A5,$7D,$76,$96,$C5,$D0,$97,$25,$23,$5C
; .db $2B,$F6,$05,$CB,$F2,$82,$06,$77,$1D,$7A,$01,$89,$88,$F8,$1D,$10
; .db $1F,$AE,$EE,$77,$A2,$4C,$AC,$EC,$34,$DA,$2B,$55,$A4,$F5,$4C,$FE
; .db $51,$73,$AB,$9F,$99,$73,$50,$E7,$69,$73,$36,$D3,$B4,$33,$9F,$5A
; .db $84,$44,$27,$8C,$4C,$DA,$C4,$9A,$16,$07,$CC,$91,$DE,$80,$25,$70
; .db $93,$1E,$73,$AA,$C7,$65,$B7,$E3,$C6,$A4,$C3,$1A,$C6,$84,$0E,$92
; .db $F7,$F7,$EB,$61,$F2,$40,$03,$71,$AF,$A0,$77,$66,$31,$E7,$FF,$7E
; .db $58,$31,$5E,$2D,$16,$94,$A7,$A1,$7D,$55,$22,$D8,$9F,$2C,$C8,$58
; .db $00,$BC,$AC,$9E,$B7,$A3,$12,$15,$D6,$8A,$CC,$1E,$33,$B0,$F9,$02
; .db $1F,$B8,$B4,$C8,$41,$1E,$B4,$9A,$55,$99,$E1,$91,$C2,$68,$F4,$86
; .db $69,$DA,$C1,$D9,$C1,$7A,$5A,$D9,$33,$7E,$31,$CF,$86,$C4,$94,$CC
; .db $04,$1E,$07,$DD,$C9,$07,$EF,$8A,$C6,$B0,$FE,$7E,$74,$E2,$B6,$5A
; .db $87,$47,$6F,$53,$A5,$B1,$ED,$13,$A6,$4F,$08,$62,$72,$F2,$79,$CD
; .db $1B,$61,$90,$48,$C2,$E8,$58,$99,$6D,$0B,$77,$37,$61,$AC,$40,$6D
; .db $85,$9D,$73,$C5,$B8,$CB,$E6,$7F,$E5,$85,$4D,$E9,$EA,$E0,$4B,$87
; .db $29,$8D,$68,$7B,$07,$B2,$05,$91,$67,$B3,$AA,$65,$83,$A5,$04,$BD
; .db $4A,$B0,$1D,$A1,$A9,$68,$8A,$08,$38,$CD,$A3,$7C,$A9,$66,$00,$4A
; .db $7F,$49,$F3,$78,$1D,$C6,$99,$00,$85,$D4,$66,$48,$88,$18,$86,$E7
; .db $17,$D9,$04,$B2,$15,$67,$80,$52,$80,$6D,$38,$B4,$05,$C0,$46,$F4
; .db $07,$84,$2A,$BB,$9E,$AA,$04,$7B,$2A,$E1,$13,$0E,$89,$6F,$C5,$78
; .db $A3,$CF,$46,$95,$7C,$E8,$45,$F1,$FD,$D1,$C4,$20,$82,$6A,$81,$9F
; .db $47,$BA,$E7,$81,$25,$3E,$22,$D1,$A3,$D5,$BB,$A4,$07,$54,$1E,$00
; .db $55,$73,$94,$FE,$90,$FB,$B8,$B8,$E1,$86,$40,$A7,$AF,$70,$CE,$25
; .db $FF,$FC,$89,$C7,$2F,$19,$51,$AB,$AE,$30,$E8,$57,$04,$3A,$C9,$E5
; .db $8C,$5D,$FC,$32,$9C,$F4,$AE,$1D,$94,$0E,$26,$DD,$4F,$F5,$FD,$1E
; .db $36,$BB,$6B,$4A,$F9,$1E,$87,$BC,$67,$7B,$BE,$F3,$DE,$54,$8D,$EB
; .db $86,$B2,$95,$1A,$97,$62,$08,$4B,$44,$76,$03,$C4,$87,$5A,$69,$A9
; .db $23,$EA,$00,$7E,$2C,$F8,$01,$44,$09,$D5,$EC,$FD,$A9,$79,$48,$F5
; .db $CF,$55,$38,$89,$8B,$45,$E4,$96,$27,$07,$79,$C0,$FD,$75,$1B,$7E


attribute: ; sets background color groups
  ; .db %00000000, %00010000, %01010000, %00010000, %00000000, %00000000, %00000000, %00110000

  .db %11010101, %01111100, %10100010, %00101110, %11011001, %11111010, %00001001, %10001000
  .db %11001111, %11010110, %10001110, %00011110, %01010001, %01011001, %01111011, %11000101
  .db %00110110, %11011010, %10101101, %11011011, %11111110, %01101011, %01110101, %01111111
  .db %01100001, %10011101, %01000011, %01001000, %00001111, %00010000, %01000110, %11110011
  .db %00010101, %10000111, %01000011, %10000111, %10111001, %01011011, %11101011, %11111001
  .db %11000011, %10110111, %11101100, %11010001, %10011101, %01001101, %11010101, %00111000
  .db %01011010, %00011100, %10100010, %11010011, %10010001, %11001000, %01100001, %11101110
  .db %00010100, %00010111, %01000111, %01001110, %11000010, %01001001, %10111001, %11110001




  .org $FFFA   ; first of the three vectors starts here
  .dw NMI      ; when an NMI happens (once per frame if enabled) the processor will jump to the label NMI:
  .dw RESET    ; when the processor first turns on or is reset, it will jumpto the label RESET:
  .dw 0        ; external interrupt IRQ is not used in this code


;;;;;;;;;;;;;;  


  .bank 2
  .org $0000
  .incbin "mario.chr"   ; includes 8KB graphics file from SMB1