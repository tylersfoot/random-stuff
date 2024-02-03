using System;
using System.Runtime.InteropServices;
using System.Threading;
using System.Drawing;
using System.Xml.Linq;
using System.Net.NetworkInformation;
using static System.Runtime.InteropServices.JavaScript.JSType;

namespace GDI_CS {
	public static class Payloads {

		private static ThreadLocal<Random> random = new ThreadLocal<Random>(() => new Random());

		public static void Invert(int xstart, int ystart, int w, int h) {
			// inverts random rectangle of the screen with a random color filter applied

			IntPtr hdc = WinApi.GetDC(IntPtr.Zero); // get device context for screen
			uint color = (uint)((random.Value!.Next(256) << 16) | (random.Value!.Next(256) << 8) | random.Value!.Next(256));
			IntPtr brush = WinApi.CreateSolidBrush(color);
			WinApi.SelectObject(hdc, brush);
			WinApi.PatBlt(hdc, xstart, ystart, w, h, WinApi.PATINVERT);
			WinApi.DeleteObject(brush);
			WinApi.ReleaseDC(IntPtr.Zero, hdc);
		}

		public static void Blur(int xdest, int ydest, int wdest, int hdest, int xsrc, int ysrc, int wsrc, int hsrc, int x, int y) {
			// blurs the screen
			// dest: where the blended image goes
			// src:  the image that will be blended

			IntPtr hdc = WinApi.GetDC(IntPtr.Zero);
			IntPtr mhdc = WinApi.CreateCompatibleDC(hdc);


			IntPtr hbit = WinApi.CreateCompatibleBitmap(hdc, x, y);
			IntPtr holdbit = WinApi.SelectObject(mhdc, hbit);

			WinApi.BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, WinApi.SRCCOPY);

			WinApi.BLENDFUNCTION blendFunction = new WinApi.BLENDFUNCTION {
				BlendOp = 0x00, // AC_SRC_OVER
				BlendFlags = 0x00,
				SourceConstantAlpha = 70, // Use a constant alpha value for blending (0-255)
				AlphaFormat = 0x00 // AC_SRC_ALPHA
			};

			WinApi.AlphaBlend(hdc, xdest, ydest, wdest, hdest, mhdc, xsrc, ysrc, wsrc, hsrc, blendFunction);
			WinApi.SelectObject(mhdc, holdbit);
			WinApi.DeleteObject(holdbit);
			WinApi.DeleteObject(hbit);
			WinApi.DeleteDC(mhdc);
			WinApi.ReleaseDC(IntPtr.Zero, hdc);
		}

		public static void Melt(int slicewidth, int vshift, int x, int y) {
			// slicewidth: width of the slice
			// vshift: where to shift the slice vertically

			IntPtr hdc = WinApi.GetDC(IntPtr.Zero);
			int r = random.Value!.Next(x);


			WinApi.BitBlt(hdc, r, vshift, slicewidth, y, hdc, r, 0, WinApi.SRCCOPY);
			WinApi.ReleaseDC(IntPtr.Zero, hdc);
		}

		public static void Tunnel(int strength, int jiggle, bool invert, bool rainbow, int x, int y) {
			// makes a tunnel effect on the screen - loop it
			// strength: how deep the tunnel is
			// jiggle: how much the tunnel moves around
			// invert: inverts on each tunnel
			// rainbow: adds random brush color

			IntPtr hdc = WinApi.GetDC(IntPtr.Zero);

			IntPtr mhdc = WinApi.CreateCompatibleDC(hdc);

			IntPtr hbit = WinApi.CreateCompatibleBitmap(hdc, x, y);

			WinApi.SelectObject(mhdc, hbit);

			if (invert) {
				// invert hdc
				WinApi.PatBlt(hdc, 0, 0, x, y, WinApi.PATINVERT);

				if (rainbow) {
					uint color = (uint)((random.Value!.Next(256) << 16) | (random.Value!.Next(256) << 8) | random.Value!.Next(256));
					IntPtr brush = WinApi.CreateSolidBrush(color);
					WinApi.SelectObject(hdc, brush);
				}

				// copy hdc to mhdc
				WinApi.BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, WinApi.SRCCOPY);
				// invert hdc
				WinApi.PatBlt(hdc, 0, 0, x, y, WinApi.PATINVERT);
			}
			else {
				// copy hdc to mhdc
				WinApi.BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, WinApi.SRCCOPY);
			}

			// copy mhdc onto hdc but smaller
			WinApi.StretchBlt(hdc, strength + random.Value!.Next(-jiggle, jiggle),
				strength + random.Value!.Next(-jiggle, jiggle),
				x - (strength * 2), y - (strength * 2),
				mhdc, 0, 0, x, y, WinApi.SRCCOPY);
			WinApi.DeleteObject(hbit);

			WinApi.DeleteDC(mhdc);

			WinApi.ReleaseDC(IntPtr.Zero, hdc);
		}

		public static void Pixellize(int size, int x, int y) {
			// pixellizes the screen
			// size: size of pixels

			IntPtr hdc = WinApi.GetDC(IntPtr.Zero);

			IntPtr mhdc = WinApi.CreateCompatibleDC(hdc);

			IntPtr hbit = WinApi.CreateCompatibleBitmap(hdc, x, y);

			WinApi.SelectObject(mhdc, hbit);

			WinApi.BitBlt(mhdc, 0, 0, x, y, hdc, 0, 0, WinApi.SRCCOPY);

			// cut the image and stretch it back to size
			WinApi.StretchBlt(hdc, 0, 0, x, y, mhdc, 0, 0, x / size, y / size, WinApi.SRCCOPY);

			WinApi.DeleteObject(hbit);
			WinApi.DeleteDC(mhdc);
			WinApi.ReleaseDC(IntPtr.Zero, hdc);
		}

		public static void DrawError(bool cursor, bool error, int x, int y) {
			// draws error icons on screen
			// cursor: draws at cursor, else randomly
			// error: use error icon, else warning icon
			IntPtr hdc = WinApi.GetDC(IntPtr.Zero);

			if (cursor) {
				WinApi.POINT cursorPos;
				if (WinApi.GetCursorPos(out cursorPos)) {
					if (error) {
						WinApi.DrawIcon(hdc, cursorPos.X, cursorPos.Y, Data.IconError);
					}
					else {
						WinApi.DrawIcon(hdc, cursorPos.X, cursorPos.Y, Data.IconError);
					}
				}
			}
			else {

				if (error) {
					WinApi.DrawIcon(hdc, random.Value!.Next(x), random.Value!.Next(y), Data.IconError);
				}
				else {
					WinApi.DrawIcon(hdc, random.Value!.Next(x), random.Value!.Next(y), Data.IconWarning);
				}
			}
		}

		public static void Puzzle(int size, int x, int y) {
			// puzzles up the screen
			// size: size of the blocks
			IntPtr hdc = WinApi.GetDC(IntPtr.Zero);

			int x1 = random.Value!.Next(x - 100);
			int y1 = random.Value!.Next(y - 100);
			int x2 = random.Value!.Next(x - 100);
			int y2 = random.Value!.Next(y - 100);

			int width = random.Value!.Next(size);
			int height = random.Value!.Next(size);

			WinApi.BitBlt(hdc, x1, y1, width, height, hdc, x2, y2, WinApi.SRCCOPY);
		}


		public static void Rotate(int amount, int xoffset, int yoffset, int w, int h, int x, int y) {
			// rotates the screen
			// amount: amount the screen rotates
			IntPtr hdc = WinApi.GetDC(IntPtr.Zero);

			IntPtr mhdc = WinApi.CreateCompatibleDC(hdc);

			IntPtr hbit = WinApi.CreateCompatibleBitmap(hdc, x, y);

			WinApi.SelectObject(mhdc, hbit);

			WinApi.POINT[] points = new WinApi.POINT[] {
			new WinApi.POINT(amount, -amount),
			new WinApi.POINT(x + amount, amount),
			new WinApi.POINT(-amount, y - amount)
			};

			WinApi.PlgBlt(hdc, points, hdc, -xoffset, -yoffset, x + w, y + h, IntPtr.Zero, 0, 0);

			WinApi.DeleteObject(hbit);

			WinApi.DeleteDC(mhdc);

			WinApi.ReleaseDC(IntPtr.Zero, hdc);
		}

		//public static void ReverseText() {
		//	// check for all open windows
		//	IntPtr desktopHandle = WinApi.GetDesktopWindow();
		//	// enumerate through all open windows and call reverse text function
		//	WinApi.EnumChildWindows(desktopHandle, WinApi.ChildWindowCallback, IntPtr.Zero);
		//}
	}
}