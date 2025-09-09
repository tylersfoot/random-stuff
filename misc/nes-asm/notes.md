### Basics

```
    .org $8000 ; Directive: put the following code starting at addr $8000
MeowFunction: ; Label: visual organization, assembler sets all "MeowFunction" to "$8000"
    LDA #$FF ; Operand: opcodes can have 1-3 operands, in this case, "#$FF"
    JMP MeowFunction ; Opcode: the instruction to run, in this case, jump to "MeowFunction" ($8000)
```

`#value` = use the raw number, ex. `#$0020` = 32 in decimal
otherwise, use the value at the address, ex. `$0020` = value at address `$0020`
`LDA #$05` means load the value 5, `LDA $0005` means load the value that is stored at address `$0005`


### Registers

**Accumulator (A):** 
The main 8 bit register for loading, storing, comparing, and doing math on data.

```
LDA #$FF  ; load the hex value $FF (decimal 256) into A
STA $0000 ; store the accumulator into memory location $0000, internal RAM
```

**Index Register X (X):**
Another 8 bit register, usually used for counting or memory access. In loops, this register is used to keep track of how many times the loop has gone, while using A to process data.

```
LDX $0000 ; load the value at memory location $0000 into X
INX       ; increment X   X = X + 1
```

**Index Register Y (Y):**
Works almost the same as X; Some instructions only work with X and not Y.

```
STY $00BA ; store Y into memory location $00BA
TYA       ; transfer Y into Accumulator
```

**Status Register:**
Holds flags with information about the last instruction; like after subtracting, you can check if the result was zero.


## Instructions / Opcodes

These are just the most common and basic instructions. Most have a few different options which will be used later.

[nesdev.org](https://www.nesdev.org/wiki/Instruction_reference)

Key:
v = value (#$FF)
a = address ($FF)
Z = zero flag
C = carry flag
N = negative flag

#### Load/Store

```
LDA [val/addr]  ; LoaD a value into the accumulator A
                ; [val = 0 -> zero flag is set]

LDX [val/addr]  ; LoaD a value into the index register X
                ; [val = 0 -> zero flag is set]

LDY [val/addr]  ; LoaD a value $FF into the index register Y
                ; [val = 0 -> zero flag is set]

STA [addr]      ; STore the value from accumulator A into an address

STX [addr]      ; STore the value from index register X into an address

STY [addr]      ; STore the value from index register Y into an address

TAX             ; Transfer the value from A into X
                ; [val = 0 -> zero flag is set]

TAY             ; Transfer the value from A into Y
                ; [val = 0 -> zero flag is set]

TXA             ; Transfer the value from X into A
                ; [val = 0 -> zero flag is set]

TYA             ; Transfer the value from Y into A
                ; [val = 0 -> zero flag is set]
```

**Math**

```
ADC [val]       ; ADd with Carry
                ; A = A + val + carry
                ; [val = 0 -> zero flag is set]

SBC [val]       ; SuBtract with Carry
                ; A = A - val - (1 - carry)
                ; [val = 0 -> zero flag is set]

CLC             ; CLear Carry flag in status register
                ; usually this should be done before ADC

SEC             ; SEt Carry flag in status register
                ; usually this should be done before SBC

INC [addr]      ; INCrement the value at an address
                ; [val = 0 -> zero flag is set]

DEC [addr]      ; DECrement the value at an address
                ; [val = 0 -> zero flag is set]

INY             ; INcrement Y register
                ; [val = 0 -> zero flag is set]

INX             ; INcrement X register
                ; [val = 0 -> zero flag is set]

DEY             ; DEcrement Y register
                ; [val = 0 -> zero flag is set]

DEX             ; DEcrement X register
                ; [val = 0 -> zero flag is set]

ASL [A/addr]    ; Arithmetic Shift Left
                ; shift all bits one position to the left
                ; this is a multiply by 2
                ; [val = 0 -> zero flag is set]

LSR [A/addr]    ; Logical Shift Right
                ; shift all bits one position to the right
                ; this is a divide by 2
                ; [val = 0 -> zero flag is set]
``` 

**Comparison**

```
CMP #$01   ; CoMPare A to the value $01
           ; this actually does a subtract, but does not keep the result
           ; instead you check the status register to check for equal, 
           ; less than, or greater than

CPX $0050  ; ComPare X to the value at address $0050

CPY #$FF   ; ComPare Y to the value $FF
```

**Control Flow**

```
JMP $8000  ; JuMP to $8000, continue running code there

BEQ $FF00  ; Branch if EQual, contnue running code there
           ; first you would do a CMP, which clears or sets the zero flag
           ; then the BEQ will check the zero flag
           ; if zero is set (values were equal) the code jumps to $FF00 and runs there
           ; if zero is clear (values not equal) there is no jump, runs next instruction

BNE $FF00  ; Branch if Not Equal - opposite above, jump is made when zero flag is clear
```



