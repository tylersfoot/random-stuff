﻿using System;
using System.Runtime.InteropServices;
using System.Threading;
using System.Drawing;
using System.Xml.Linq;
using System.Net.NetworkInformation;

public static class Payloads
{
    [DllImport("user32.dll")]
    public static extern IntPtr GetDC(IntPtr hWnd);

    [DllImport("gdi32.dll")]
    public static extern IntPtr CreateSolidBrush(uint crColor);

    [DllImport("gdi32.dll")]
    public static extern IntPtr SelectObject(IntPtr hdc, IntPtr h);

    [DllImport("gdi32.dll")]
    public static extern bool PatBlt(IntPtr hdc, int x, int y, int w, int h, uint rop);

    [DllImport("gdi32.dll")]
    public static extern bool BitBlt(IntPtr hdcDest, int xDest, int yDest, int wDest, int hDest, IntPtr hdcSrc, int xSrc, int ySrc, int Rop);

    [DllImport("gdi32.dll", EntryPoint = "GdiAlphaBlend")]
    public static extern bool AlphaBlend(IntPtr hdcDest, int xDest, int yDest, int wDest, int hDest, IntPtr hdcSrc, int xSrc, int ySrc, int wSrc, int hSrc, BLENDFUNCTION blendFunction);


    [DllImport("gdi32.dll")]
    public static extern bool DeleteObject(IntPtr hObject);

    [DllImport("user32.dll")]
    public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);

    [DllImport("gdi32.dll")]
    public static extern bool DeleteDC(IntPtr hdc);

    [DllImport("gdi32.dll")]
    public static extern void RGB(int red, int green, int blue);

    [DllImport("gdi32.dll")]
    public static extern IntPtr CreateCompatibleDC(IntPtr hdc);

    [DllImport("gdi32.dll")]
    public static extern IntPtr CreateCompatibleBitmap(IntPtr hdc, int nWidth, int nHeight);

    [StructLayout(LayoutKind.Sequential)]
    public struct BLENDFUNCTION {
        public byte BlendOp;
        public byte BlendFlags;
        public byte SourceConstantAlpha;
        public byte AlphaFormat;
    }

    private const uint PATINVERT = 0x005A0049;
    const int SRCCOPY = 0x00CC0020;

    private static ThreadLocal<Random> random = new ThreadLocal<Random>(() => new Random());

    public static void Invert(int xstart, int ystart, int w, int h) {
        // inverts random rectangle of the screen with a random color filter applied

        IntPtr hdc = GetDC(IntPtr.Zero); // get device context for screen
        uint color = (uint)((random.Value.Next(256) << 16) | (random.Value.Next(256) << 8) | random.Value.Next(256));
        IntPtr brush = CreateSolidBrush(color);
        SelectObject(hdc, brush);
        PatBlt(hdc, xstart, ystart, w, h, PATINVERT);
        DeleteObject(brush);
        ReleaseDC(IntPtr.Zero, hdc);
    }

    public static void Blur(int xdest, int ydest, int wdest, int hdest, int xsrc, int ysrc, int wsrc, int hsrc, int x, int y) {
        // blurs the screen
        // dest: where the blended image goes
        // src:  the image that will be blended

        IntPtr hdc = GetDC(IntPtr.Zero);
        IntPtr mhdc = CreateCompatibleDC(hdc);


        IntPtr hbit = CreateCompatibleBitmap(hdc, x, y);
        IntPtr holdbit = SelectObject(mhdc, hbit);


        BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, SRCCOPY);

        BLENDFUNCTION blendFunction = new BLENDFUNCTION
        {
            BlendOp = 0x00, // AC_SRC_OVER
            BlendFlags = 0x00,
            SourceConstantAlpha = 70, // Use a constant alpha value for blending (0-255)
            AlphaFormat = 0x00 // AC_SRC_ALPHA
        };

        AlphaBlend(hdc, xdest, ydest, wdest, hdest, mhdc, xsrc, ysrc, wsrc, hsrc, blendFunction);
        SelectObject(mhdc, holdbit);
        DeleteObject(holdbit);
        DeleteObject(hbit);
        DeleteDC(mhdc);
        ReleaseDC(IntPtr.Zero, hdc);
    }
}