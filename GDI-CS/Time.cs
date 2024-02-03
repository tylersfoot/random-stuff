using System.Threading;

namespace GDI_CS {
	public static class Time {
		public static double GetTime() {
			DateTime epoch = new DateTime(1970, 1, 1, 0, 0, 0, DateTimeKind.Utc);
			DateTime now = DateTime.UtcNow;
			return (now - epoch).TotalSeconds;
		}
		public static void Sleep(int seconds) {
			Thread.Sleep(seconds * 1000); // convert seconds to milliseconds
		}
	}
}
