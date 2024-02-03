using System.Runtime.InteropServices;
using System.Text;

namespace GDI_CS {
	public static class WinApi {
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

		[DllImport("gdi32.dll", EntryPoint = "StretchBlt", SetLastError = true)]
		[return: MarshalAs(UnmanagedType.Bool)]
		public static extern bool StretchBlt(
			IntPtr hdcDest,     // handle to destination DC
			int nXOriginDest,   // x-coord of destination upper-left corner
			int nYOriginDest,   // y-coord of destination upper-left corner
			int nWidthDest,     // width of destination rectangle
			int nHeightDest,    // height of destination rectangle
			IntPtr hdcSrc,      // handle to source DC
			int nXOriginSrc,    // x-coord of source upper-left corner
			int nYOriginSrc,    // y-coord of source upper-left corner
			int nWidthSrc,      // width of source rectangle
			int nHeightSrc,     // height of source rectangle
			uint dwRop          // raster operation code
		);

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

		[DllImport("user32.dll", SetLastError = true)]
		public static extern bool GetCursorPos(out POINT lpPoint);

		[DllImport("user32.dll", SetLastError = true)]
		public static extern bool DrawIcon(IntPtr hDC, int X, int Y, IntPtr hIcon);

		[StructLayout(LayoutKind.Sequential)]
		public struct POINT {
			public int X;
			public int Y;
			public POINT(int x, int y) {
				X = x;
				Y = y;
			}
		}

		[DllImport("gdi32.dll", SetLastError = true)]
		public static extern bool PlgBlt(IntPtr hdcDest, POINT[] lpPoint, IntPtr hdcSrc,
		int nXSrc, int nYSrc, int nWidth, int nHeight,
		IntPtr hbmMask, int xMask, int yMask);

		[DllImport("user32.dll", SetLastError = true)]
		public static extern IntPtr GetDesktopWindow();

		[DllImport("user32.dll", SetLastError = true)]
		public static extern bool EnumChildWindows(IntPtr hwndParent, EnumWindowsProc lpEnumFunc, IntPtr lParam);

		// delegate to be used by EnumChildWindows as callback
		public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

		[DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
		public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

		[DllImport("user32.dll", SetLastError = true, CharSet = CharSet.Auto)]
		public static extern bool SetWindowText(IntPtr hwnd, String lpString);

		[DllImport("user32.dll")]
		public static extern int GetSystemMetrics(int nIndex);

		[DllImport("user32.dll")]
		public static extern bool SetProcessDPIAware();

		[DllImport("user32.dll", CharSet = CharSet.Auto)]
		public static extern int MessageBox(IntPtr hWnd, String text, String caption, uint type);

		// MessageBox constants
		public const uint MB_YESNO = 0x00000004;
		public const uint MB_ICONWARNING = 0x00000030;
		public const int IDNO = 7; // the value returned by MessageBox when 'No' is clicked

		public const uint PATINVERT = 0x005A0049;
		public const int SRCCOPY = 0x00CC0020;
	}
}