using System;
using System.Runtime.InteropServices;
using System.Threading;
using System.Drawing;

public static class Payloads {
    [DllImport("user32.dll")]
    public static extern IntPtr GetDC(IntPtr hWnd);

    [DllImport("gdi32.dll")]
    public static extern IntPtr CreateSolidBrush(uint crColor);

    [DllImport("gdi32.dll")]
    public static extern IntPtr SelectObject(IntPtr hdc, IntPtr h);

    [DllImport("gdi32.dll")]
    public static extern bool PatBlt(IntPtr hdc, int x, int y, int w, int h, uint rop);

    [DllImport("gdi32.dll")]
    public static extern bool DeleteObject(IntPtr hObject);

    [DllImport("user32.dll")]
    public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);

    // Constants for PatBlt
    private const uint PATINVERT = 0x005A0049;

    // private static readonly Random random = new Random();
    private static ThreadLocal<Random> random = new ThreadLocal<Random>(() => new Random());

    public static void Invert(int xstart, int ystart, int w, int h) {
        // inverts random rectangle of the screen with a random color filter applied

        IntPtr hdc = GetDC(IntPtr.Zero); // get device context for screen
        IntPtr brush = CreateSolidBrush(RGB(random.Next(256), random.Next(256), random.Next(256)));
        SelectObject(hdc, brush);
        PatBlt(hdc, xstart, ystart, w, h, PATINVERT);
        DeleteObject(brush);
        ReleaseDC(IntPtr.Zero, hdc);
    }
}
