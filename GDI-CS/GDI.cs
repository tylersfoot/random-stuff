using System.Text.RegularExpressions;
using System;
using System.Runtime.InteropServices;
using System.Linq.Expressions;

namespace GDI_CS {
	internal class GDI {

		private static ThreadLocal<Random> random = new ThreadLocal<Random>(() => new Random());

		[DllImport("user32.dll")]
		public static extern int GetSystemMetrics(int nIndex);

		[DllImport("user32.dll")]
		private static extern bool SetProcessDPIAware();

		public static void Exit() {
			Environment.Exit(0);
		}

		static void Main(string[] args) {
			Console.WriteLine("Hello, World!");

			SetProcessDPIAware();

			int x = GetSystemMetrics(0);
			int y = GetSystemMetrics(1);
			int stage = 2;

			int timer = 0;

			while (true) {
				timer += 1;

				if (timer == 500) {
					Exit();
				}

				switch (stage) {
					case 0:
						if (timer % 2 == 0) {
							Payloads.Melt(random.Value!.Next(50, 500), random.Value!.Next(-10, 10), x, y);
						}
						if (timer % 200 == 0) {
							Payloads.Tunnel((int)(x / 50), (int)(y / 50), false, false, x, y);
						}
						break;
					case 1:
						if (timer % 5 == 0) {
							Payloads.Invert(random.Value!.Next(x / 2), random.Value!.Next(y / 2), random.Value!.Next(x), random.Value!.Next(y));
						}
						break;
					case 2:
						if (timer % 2 == 0) {
							//Payloads.Melt(random.Value!.Next(50, 500), random.Value!.Next(-10, 10), x, y);
						}
						if (timer % 5 == 0) {
							//Payloads.Invert(random.Value!.Next(x / 2), random.Value!.Next(y / 2), random.Value!.Next(x), random.Value!.Next(y));
						}
						if (timer % 15 == 0) {
							//Payloads.Blur(random.Value!.Next(-2, 2), random.Value!.Next(-2, 2), x, y, 0, 0, x, y, x, y);
						}
						if (timer % 10 == 0) {
							//Payloads.Tunnel((int)(x / 100), (int)(y / 100), true, true, x, y);
						}

						break;
					case 3:
						Exit();
						break;
					default:
						break;
				}
			}
		}
	}
}


//    global x, y, playAudio, stage
//    global hdc, mhdc, hbit, holdbit


//    playAudio = False
//    stage = -1


//    p = Payloads

//    # screen resolution will be correct
//    ctypes.windll.user32.SetProcessDPIAware()


//    x = GetSystemMetrics(0)
//    y = GetSystemMetrics(1)

//    # generate music buffers while user is distracted by warning
//    threading.Thread(target = Music, args = ()).start()

//    # throw message boxes
//    Warning()


//    time.sleep(1)


//    playAudio = True


//    timer = 0
//    # t_end = time.time() + 16
//    # while time.time() < t_end:
//    while True:
//        if keyboard.is_pressed('esc'):
//            Exit()


//        timer += 1


//        match stage:
//            case 0:
//                if (timer % 2 == 0):
//                    p.Melt(randrange(50, 500), randrange(-10, 10))
//                if (timer % 200 == 0):
//                    p.Tunnel(int(x / 50), int(y / 50), False, False)
//                if (timer % 200 == 0):
//                    p.DrawError(False, True)
//                if (timer % 200 == 50):
//                    p.DrawError(True, True)
//            case 1:
//                if (timer % 5 == 0):
//                    p.Rotate(randrange(-100, 100), int(x / 100), int(y / 100), int(x / 50), int(y / 50))
//                if (timer % 1 == 0):
//                    p.Puzzle(500)
//                if (timer % 5 == 0):
//                    p.Invert(randrange(x//2), randrange(y//2), randrange(x), randrange(y))
//                if (timer % 30 == 0):
//                    p.DrawError(False, True)
//                if (timer % 30 == 15):
//                    p.DrawError(True, True)
//            case 2:
//                if (timer % 2 == 0):
//                    p.Melt(randrange(50, 500), randrange(-50, 50))


//                if (timer % 5 == 0):
//                    p.Invert(randrange(x//2), randrange(y//2), randrange(x), randrange(y))

//                #if (timer % 15 == 0):
//                  # p.Blur(randrange(-2, 2), randrange(-2, 2), x, y, 0, 0, x, y)

//                if (timer % 10 == 0):
//                    p.Tunnel(int(x/100), int(y/100), True, True)

//                if (timer % 1 == 0):
//                    p.DrawError(False, False)
//                    p.DrawError(True, True)

//                if (timer % 1 == 0):
//                    p.Puzzle(500)

//                if  (timer % 5 == 0):
//                    p.Rotate(randrange(-100, 100), int(x/100), int(y/100), int(x/50), int(y/50))
//            case 3:
//                Exit()
//            case _:
//                timer = timer 

//    Exit()

//    if (timer % 2 == 0):
//        p.Melt(randrange(50, 500), randrange(-50, 50))

//    if (timer % 5 == 0):
//        p.Invert(randrange(x//2), randrange(y//2), randrange(x), randrange(y))

//    if (timer % 15 == 0):
//        p.Blur(randrange(-2, 2), randrange(-2, 2), x, y, 0, 0, x, y)

//    if (timer % 10 == 0):
//        p.Tunnel(int(x/100), int(y/100), True, True)

//    if (timer % 1 == 0):
//        p.DrawError(False, False)
//        p.DrawError(True, True)

//    if (timer % 1 == 0):
//        p.Puzzle(500)

//    if  (timer % 5 == 0):
//        p.Rotate(randrange(-100, 100), int(x/100), int(y/100), int(x/50), int(y/50))