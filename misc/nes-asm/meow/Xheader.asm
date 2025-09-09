; iNES Header
; The 16 byte iNES header gives the emulator all the information about the game 
; including mapper, graphics mirroring, and PRG/CHR sizes. 

  .inesprg 1 ; 1x 16KB bank of PRG code
  .ineschr 1 ; 1x 8KB bank of CHR data
  .inesmap 0 ; mapper 0 = NROM, no bank swapping
  .inesmir 1 ; background mirroring (ignore for now)


; Banking
; NESASM arranges everything in 8KB code and 8KB graphics banks. 
; To fill the 16KB PRG space 2 banks are needed. 
; Like most things in computing, the numbering starts at 0. 
; For each bank you have to tell the assembler where in memory it will start.

  .bank 0
  .org $C000
 ;some code here

  .bank 1
  .org $E000
; more code here

  .bank 2
  .org $0000
; graphics here


; Adding Binary Files
; Additional data files are frequently used for graphics data or level data.
; The incbin directive can be used to include that data in your .NES file.
; This data will not be used yet, but is needed to make the .NES file size match the iNES header.

  .bank 2
  .org $0000
  .incbin "mario.chr" ; includes 8KB graphics file from SMB1


; Vectors
; There are three times when the NES processor will interrupt your code and jump to a new location.
; These vectors, held in PRG ROM tell the processor where to go when that happens.

; NMI Vector - this happens once per video frame, when enabled. 
;              The PPU tells the processor it is starting the VBlank time 
;              and is available for graphics updates.
; RESET Vector - this happens every time the NES starts up, or the reset button is pressed.
; IRQ Vector - this is triggered from some mapper chips or audio interrupts and will not be covered.

; These three must always appear in your assembly file the right order. 
 ; The .dw directive is used to define a Data Word (1 word = 2 bytes):

  .bank 1
  .org $FFFA ; first of the three vectors starts here
  .dw NMI    ; when an NMI happens (once per frame if enabled) the processor will jump to the label NMI:
  .dw RESET  ; when the processor first turns on or is reset, it will jumpto the label RESET:
  .dw 0      ; external interrupt IRQ is not used in this tutorial


; Reset Code
; The reset vector was set to the label RESET, so when the processor starts up,
; it will start from RESET: using the .org directive that code is set to a space in game ROM.
; A couple modes are set right at the beginning. We are not using IRQs, so they are turned off.
; The NES 6502 processor does not have a decimal mode, so that is also turned off.
; This section does NOT include everything needed to run code on the real NES,
; but will work with the FCEUXD SP emulator. More reset code will be added later.

  .bank 0
  .org $C000
RESET:
  SEI ; disable IRQs
  CLD ; disable decimal mode


; Completing The Program
; Your first program will be very exciting, displaying an entire screen of one color!
; To do this the first PPU settings need to be written. This is done to memory address $2001.
; The 76543210 is the bit number, from 7 to 0.
; Those 8 bits form the byte you will write to $2001.

; PPUMASK ($2001)

; 76543210
; ||||||||
; |||||||+- Grayscale
; |||||||    0: normal color
; |||||||    1: AND all palette entries with 0x30,
; |||||||       effectively producing a monochrome display
; |||||||       note that colour emphasis STILL works when this is on!
; ||||||+-- Disable background clipping in leftmost 8 pixels of screen
; |||||+--- Disable sprite clipping in leftmost 8 pixels of screen
; ||||+---- Enable background rendering
; |||+----- Enable sprite rendering
; ||+------ Intensify reds (and darken other colors)
; |+------- Intensify greens (and darken other colors)
; +-------- Intensify blues (and darken other colors)

; So if you want to enable the sprites, you set bit 3 to 1. 
; For this program bits 7, 6, 5 will be used to set the screen color:

  LDA %10000000 ; intensify blues
  STA $2001
Forever:
  JMP Forever ; infinite loop