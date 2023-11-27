from win32gui import *
from win32api import *
from win32ui import *
from win32con import *
from win32file import *
from random import *
import time

def main():
    if MessageBox("Do you want free robux???? :3", "Free Robux!", MB_ICONWARNING | MB_YESNO) == 7:
        return
    if MessageBox("Are you suuree?????", "Free Robux Forever!", MB_ICONWARNING | MB_YESNO) == 7:
        return

main()

desk = GetDC(0)
x = GetSystemMetrics(0)
y = GetSystemMetrics(1)

print('3')
for i in range(0, 300):
    brush = CreateSolidBrush(RGB(
        randrange(255),
        randrange(255),
        randrange(255)
    ))
    SelectObject(desk, brush)
    PatBlt(desk, randrange(x), randrange(y), randrange(x), randrange(y), PATINVERT)
    DeleteObject(brush)
    # time.sleep(0.01)
    # print(i)
ReleaseDC(desk, GetDesktopWindow())
DeleteDC(desk)