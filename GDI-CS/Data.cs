using System.Runtime.InteropServices;

namespace GDI_CS {
	public static class Data {
		[DllImport("user32.dll", SetLastError = true)]
		public static extern IntPtr LoadIcon(IntPtr hInstance, IntPtr lpIconName);

		public static string[] sites = new string[] {
		"http://google.co.ck/search?q=how+to+get+money",
		"calc",
		"notepad",
		"write",
		"explorer",
		"taskmgr",
		"msconfig",
		"mspaint",
		"devmgmt.msc",
		"control"
		};

		// defines IconWarning and IconError using P/Invoke to call LoadIcon
		public static IntPtr IconWarning = LoadIcon(IntPtr.Zero, (IntPtr)32515);
		public static IntPtr IconError = LoadIcon(IntPtr.Zero, (IntPtr)32513);

	}
}
