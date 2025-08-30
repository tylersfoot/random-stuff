// from win32gui import *
// from win32api import *
// from win32ui import *
// from win32con import *
// from win32file import *
// from win32gui_struct import *
// from random import *
// import time
// import ctypes
// from sys import exit
// import multiprocessing
// import keyboard
// from ByteBeat import *
// import threading
// import os

// def Exit():
//     os._exit()
//     exit()

// def Warning():
// 	if MessageBox("Do you want free robux???? :3", 
// 		"Free Robux!", # The title of the warning.
// 		MB_YESNO | MB_ICONWARNING) == 7: # If the user pressed no to our warning, exit the program.
// 		Exit()
// 	if MessageBox("Are you suuree?????", 
// 		"Free Robux Forever!", # The title of the warning.
// 		MB_YESNO | MB_ICONWARNING) == 7: # If the user pressed no to our warning, exit the program.
// 		Exit()
  
// def Music():
//     global stage
//     beats = []
//     # beats.append(ByteBeat.GenerateBuffer('a=t&t>>6,b=t|t>>8,c=t|t>>7,d=t|t>>9,a+b+c+d', 16, 32000))
//     # beats.append(ByteBeat.GenerateBuffer('(((t/91)&t)^((t/90)&t))-1', 16, 8000))
//     # beats.append(ByteBeat.GenerateBuffer('((~t>>max((t>>10)%16,(t>>12)%16))&t*"H$TT`0l6".charCodeAt((t>>11)%8)/19)*(10-(t>>16))', 16, 8000))
//     beats.append(ByteBeat.GenerateBuffer('(t * (t >> 9) * (t >> 11)) % 127', 16, 8000))
//     beats.append(ByteBeat.GenerateBuffer('(t * (t >> 9) * (t >> 8) & (t >> 4)) % 127', 16, 8000))
//     beats.append(ByteBeat.GenerateBuffer('(t * (t >> 5) * (t >> 8)) >> 3', 16, 8000))
//     # beats.append(ByteBeat.GenerateBuffer('((t%16000>=0&t%16000<8000)*(((t%8000)*10000000)**0.5)|(t%16000>=8000&t%16000<16000)*(((8000-(t%8000))*10000000)**0.5))|((t%8000>=0&t%8000<1000)*t*1|(t%8000>=1000&t%8000<2000)*t*2|(t%8000>=2000&t%8000<3000)*t*3|(t%8000>=3000&t%8000<4000)*t*4|(t%8000>=4000&t%8000<5000)*t*5|(t%8000>=5000&t%8000<6000)*t*6|(t%8000>=6000&t%8000<7000)*t*8|(t%8000>=7000&t%8000<8000)*t*10)<<(t%16000<=8000)&128', 16, 8000))

//     while True:
//         if playAudio:
//             stage = 0
//             ByteBeat.PlayFromBuffer(beats[0], 16, 8000, True)
//             stage = 1
//             ByteBeat.PlayFromBuffer(beats[1], 16, 8000, True)
//             stage = 2
//             ByteBeat.PlayFromBuffer(beats[2], 16, 8000, True)
//             stage = 3
//             return
        
//     # ByteBeat.Play(
//     #     't%0.81*t', # the mathematical input as a string
//     #     10, # the amount of seconds to play
//     #     8000, # kHz for the ByteBeat
//     #     True # wait until the ByteBeat finished or not
//     # )
  
// class Data:
// 	sites = (
// 		"http://google.co.ck/search?q=how+to+get+money",
// 		"calc",
// 		"notepad",
// 		"write",
// 		"explorer",
// 		"taskmgr",
// 		"msconfig",
// 		"mspaint",
// 		"devmgmt.msc",
// 		"control"
// 		)
// 	IconWarning = LoadIcon(None, 32515)
// 	IconError = LoadIcon(None, 32513)

// class Payloads:
//     def Invert(xstart, ystart, w, h):
//         # inverts random rectangle of the screen with a random color filter applied
        
//         hdc = GetDC(0)
//         brush = CreateSolidBrush(RGB(
//             randrange(255),
//             randrange(255),
//             randrange(255)
//         ))
//         SelectObject(hdc, brush)
//         PatBlt(hdc, xstart, ystart, w, h, PATINVERT)
//         DeleteObject(brush)
//         ReleaseDC(0, hdc)

    
//     def Blur(xdest, ydest, wdest, hdest, xsrc, ysrc, wsrc, hsrc):
//         # blurs the screen
//         # dest: where the blended image goes
//         # src:  the image that will be blended
        
//         hdc = GetDC(0)
//         mhdc = CreateCompatibleDC(hdc)
        
//         hbit = CreateCompatibleBitmap(hdc, x, y)
//         holdbit = SelectObject(mhdc, hbit)
        
//         BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, SRCCOPY)
//         blendfunction = (0, 0, 70, 0)
//         AlphaBlend(hdc, xdest, ydest, wdest, hdest, mhdc, xsrc, ysrc, wsrc, hsrc, blendfunction)
//         SelectObject(mhdc, holdbit)
//         DeleteObject(holdbit)
//         DeleteObject(hbit)
//         DeleteDC(mhdc)
//         ReleaseDC(0, hdc)   
        
//     def Melt(slicewidth, vshift):
//         # slicewidth: width of the slice
//         # vshift: where to shift the slice vertically
        
//         hdc = GetDC(0)
//         r = randrange(x)
        
//         BitBlt(hdc, r, vshift, slicewidth, y, hdc, r, 0, SRCCOPY)
//         ReleaseDC(0, hdc)
        
//     def Tunnel(strength, jiggle, invert, rainbow):
//         # makes a tunnel effect on the screen - loop it
//         # strength: how deep the tunnel is
//         # jiggle: how much the tunnel moves around
//         # invert: inverts on each tunnel
//         # rainbow: adds random brush color
        
//         hdc = GetDC(0)
//         mhdc = CreateCompatibleDC(hdc)
//         hbit = CreateCompatibleBitmap(hdc, x, y)
//         SelectObject(mhdc, hbit)

        
//         if invert:
//             # invert hdc
//             PatBlt(hdc, 0, 0, x, y, PATINVERT)
//             if rainbow:
//                 brush = CreateSolidBrush(RGB(
//                     randrange(255),
//                     randrange(255),
//                     randrange(255)
//                 ))
//                 SelectObject(hdc, brush)
//             # copy hdc to mhdc
//             BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, SRCCOPY)
//             # invert hdc
//             PatBlt(hdc, 0, 0, x, y, PATINVERT)
//         else:
//             # copy hdc to mhdc
//             BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, SRCCOPY)
        
//         # copy mhdc onto hdc but smaller
//         StretchBlt(hdc, strength + randrange(-jiggle, jiggle), strength + randrange(-jiggle, jiggle), x - (strength*2), y - (strength*2), mhdc, 0, 0, x, y, SRCCOPY)
//         DeleteObject(hbit)
//         DeleteDC(mhdc)
//         ReleaseDC(0, hdc)
    
//     def Pixellize(size):
//         # pixellizes the screen
//         # size: size of pixels
    
//         hdc = GetDC(0)
//         mhdc = CreateCompatibleDC(hdc)
//         hbit = CreateCompatibleBitmap(hdc, x, y)
//         SelectObject(mhdc, hbit)
        
//         BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, SRCCOPY)
        
//         # cut the image and stretch it back to size
//         StretchBlt(hdc, 0, 0, x, y, mhdc, 0, 0, x // size, y // size, SRCCOPY)
        
//         DeleteObject(hbit)
//         DeleteDC(mhdc)
//         ReleaseDC(0, hdc)
        
//     def DrawError(cursor, error):
//         # draws error icons on screen
//         # cursor: draws at cursor, else randomly
//         # error: use error icon, else warning icon
//         hdc = GetDC(0)
//         if cursor:
//             mouseX,mouseY = GetCursorPos()
//             if error:
//                 DrawIcon(hdc, mouseX, mouseY, Data.IconError)
//             else:
//                 DrawIcon(hdc, mouseX, mouseY, Data.IconWarning)
//         else:
//             if error:
//                 DrawIcon(hdc, randrange(x), randrange(y), Data.IconError)
//             else:
//                 DrawIcon(hdc, randrange(x), randrange(y), Data.IconWarning)
                
//     def Puzzle(size):
//         # puzzles up the screen
//         # size: size of the blocks
//         hdc = GetDC(0)
//         x1 = randrange(x-100)
//         y1 = randrange(y-100)
//         x2 = randrange(x-100)
//         y2 = randrange(y-100)

//         width = randrange(size)
//         height = randrange(size)

//         BitBlt(hdc, x1, y1, width, height, hdc, x2, y2, SRCCOPY)
        
//     def Rotate(amount, xoffset, yoffset, w, h):
//         # rotates the screen
//         # amount: amount the screen rotates
//         hdc = GetDC(0)
//         mhdc = CreateCompatibleDC(hdc)
        
//         hbit = CreateCompatibleBitmap(hdc, x, y)
//         SelectObject(mhdc, hbit)
        
//         points = [(amount, -amount), (x+amount, amount), (-amount, y-amount)]
        
//         PlgBlt(hdc, points, hdc, -xoffset, -yoffset, x+w, y+h)
        
//         DeleteObject(hbit)
//         DeleteDC(mhdc)
//         ReleaseDC(0, hdc)
        
        
        
//     def ReverseText():
//         HWND = GetDesktopWindow() # check for all open windows
//         EnumChildWindows(HWND, EnumChildProc, None) # enumerate through all open windows and call reverse text function
    
// def EnumChildProc(hwnd, lParam): 
//     # callback function for reversing text
// 	try:
// 		buffering = PyMakeBuffer(255) # create buffering
// 		length = SendMessage(hwnd, WM_GETTEXT, 255, buffering) # get length
// 		result = str(buffering[0:length*2].tobytes().decode('utf-16'))
// 		# reverse text
// 		result = result[::-1]

// 		SendMessage(hwnd, WM_SETTEXT, None, result) # set the windows text

// 	except: pass

// def main():
    
//     global x, y, playAudio, stage
//     global hdc, mhdc, hbit, holdbit
    
//     playAudio = False
//     stage = -1
    
//     p = Payloads
    
//     # screen resolution will be correct
//     ctypes.windll.user32.SetProcessDPIAware()
    
//     x = GetSystemMetrics(0)
//     y = GetSystemMetrics(1)
    
//     # generate music buffers while user is distracted by warning
//     threading.Thread(target = Music, args=()).start()
    
//     # throw message boxes
//     Warning()
    
//     time.sleep(1)
    
//     playAudio = True
    
//     timer = 0
//     # t_end = time.time() + 16
//     # while time.time() < t_end:
//     while True:
//         if keyboard.is_pressed('esc'):
//             Exit()
            
//         timer += 1
        
//         match stage:
//             case 0:
//                 if (timer % 2 == 0):
//                     p.Melt(randrange(50, 500), randrange(-10, 10))
//                 if (timer % 200 == 0):
//                     p.Tunnel(int(x/50), int(y/50), False, False)
//                 if (timer % 200 == 0):
//                     p.DrawError(False, True)
//                 if (timer % 200 == 50):
//                     p.DrawError(True, True)
//             case 1:
//                 if  (timer % 5 == 0):
//                     p.Rotate(randrange(-100, 100), int(x/100), int(y/100), int(x/50), int(y/50))
//                 if (timer % 1 == 0):
//                     p.Puzzle(500)
//                 if (timer % 5 == 0):
//                     p.Invert(randrange(x//2), randrange(y//2), randrange(x), randrange(y))
//                 if (timer % 30 == 0):
//                     p.DrawError(False, True)
//                 if (timer % 30 == 15):
//                     p.DrawError(True, True)
//             case 2:
//                 if (timer % 2 == 0):
//                     p.Melt(randrange(50, 500), randrange(-50, 50))
                    
//                 if (timer % 5 == 0):
//                     p.Invert(randrange(x//2), randrange(y//2), randrange(x), randrange(y))
                    
//                 # if (timer % 15 == 0):
//                 #     p.Blur(randrange(-2, 2), randrange(-2, 2), x, y, 0, 0, x, y)
                    
//                 if (timer % 10 == 0):
//                     p.Tunnel(int(x/100), int(y/100), True, True)
                    
//                 if (timer % 1 == 0):
//                     p.DrawError(False, False)
//                     p.DrawError(True, True)
                
//                 if (timer % 1 == 0):
//                     p.Puzzle(500)
                
//                 if  (timer % 5 == 0):
//                     p.Rotate(randrange(-100, 100), int(x/100), int(y/100), int(x/50), int(y/50))
//             case 3:
//                 Exit()
//             case _:
//                 timer = timer 
                
//     Exit()
            
//     if (timer % 2 == 0):
//         p.Melt(randrange(50, 500), randrange(-50, 50))
        
//     if (timer % 5 == 0):
//         p.Invert(randrange(x//2), randrange(y//2), randrange(x), randrange(y))
        
//     if (timer % 15 == 0):
//         p.Blur(randrange(-2, 2), randrange(-2, 2), x, y, 0, 0, x, y)
        
//     if (timer % 10 == 0):
//         p.Tunnel(int(x/100), int(y/100), True, True)
        
//     if (timer % 1 == 0):
//         p.DrawError(False, False)
//         p.DrawError(True, True)
    
//     if (timer % 1 == 0):
//         p.Puzzle(500)
    
//     if  (timer % 5 == 0):
//         p.Rotate(randrange(-100, 100), int(x/100), int(y/100), int(x/50), int(y/50))

// if __name__ == "__main__":
//     main()
    
static void Main(string[] args) {
    Console.WriteLine("Hello World!");
}