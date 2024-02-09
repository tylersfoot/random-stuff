using System.Text.RegularExpressions;
using System;
using System.Runtime.InteropServices;
using System.Linq.Expressions;
using System.Threading;

namespace GDI_CS {
	internal class GDI {
		private static ThreadLocal<Random> random = new ThreadLocal<Random>(() => new Random());
		public static int stage = -1;
		public static bool playAudio = false;

		public static void Exit() {
			Environment.Exit(0);
		}
		public static void Warning() {
			if (WinApi.MessageBox(IntPtr.Zero, "Do you want free roblox tix???? :3",
				"Free Tix!", // The title of the warning.
				WinApi.MB_YESNO | WinApi.MB_ICONWARNING) == WinApi.IDNO) {
				Exit();
			}
			Time.Sleep(1);
			if (WinApi.MessageBox(IntPtr.Zero, "Are you suuree?????",
				"Free Tix Forever!", // The title of the warning.
				WinApi.MB_YESNO | WinApi.MB_ICONWARNING) == WinApi.IDNO) {
				Exit();
			}
		}

		public static void Music() {
			var player = new ByteBeatPlayer(8000, 16); // 8000 Hz sample rate

			// Generate buffers
			player.GenerateBuffer("1");
			player.GenerateBuffer("2");
			player.GenerateBuffer("3");

			while (true) {
				if (playAudio) {
					stage = 0;
					player.PlayBuffer(0);
					stage = 1;
					player.PlayBuffer(1);
					stage = 2;
					player.PlayBuffer(2);
					stage = 3;
					return;
				}
			}
			//player.GenerateBuffer('(((t/91)&t)^((t/90)&t))-1", 16);
			//player.GenerateBuffer('((~t>>max((t>>10)%16,(t>>12)%16))&t*"H$TT`0l6".charCodeAt((t>>11)%8)/19)*(10-(t>>16))", 16);
			//player.GenerateBuffer('((t%16000>=0&t%16000<8000)*(((t%8000)*10000000)**0.5)|(t%16000>=8000&t%16000<16000)*
			//(((8000-(t%8000))*10000000)**0.5))|((t%8000>=0&t%8000<1000)*t*1|(t%8000>=1000&t%8000<2000)*t*2|(t%8000>=2000&t%8000<3000)*t*3|
			//(t%8000>=3000&t%8000<4000)*t*4|(t%8000>=4000&t%8000<5000)*t*5|(t%8000>=5000&t%8000<6000)*t*6|(t%8000>=6000&t%8000<7000)*t*8|
			//(t%8000>=7000&t%8000<8000)*t*10)<<(t%16000<=8000)&128", 16);
		}

		static void Main(string[] args) {
			WinApi.SetProcessDPIAware();

			int x = WinApi.GetSystemMetrics(0);
			int y = WinApi.GetSystemMetrics(1);

			// generate music buffers while user is distracted by warning
			//threading.Thread(target = Music, args = ()).start()
			Thread musicThread = new Thread(new ThreadStart(Music));
			musicThread.Start();

			Warning();

			Time.Sleep(1);

			playAudio = true;

			int timer = 0;

			while (true) {
				timer += 1;

				switch (stage) {
					case 0:
						if (timer % 2 == 0) {
							Payloads.Melt(random.Value!.Next(50, 500), random.Value!.Next(-10, 10), x, y);
						}
						if (timer % 200 == 0) {
							Payloads.Tunnel((int)(x / 50), (int)(y / 50), false, false, x, y);
						}
						if (timer % 200 == 0) {
							Payloads.DrawError(false, true, x, y);
						}
						if (timer % 200 == 50) {
							Payloads.DrawError(true, true, x, y);
						}
						if (timer % 500 == 0) {
							//Payloads.OpenSite();
						}
						break;
					case 1:
						if (timer % 5 == 0) {
							Payloads.Invert(random.Value!.Next(x / 2), random.Value!.Next(y / 2), random.Value!.Next(x), random.Value!.Next(y));
						}
						if (timer % 1 == 0) {
							Payloads.Puzzle(500, x, y);
						}
						if (timer % 30 == 0) {
							Payloads.DrawError(false, true, x, y);
						}
						if (timer % 30 == 15) {
							Payloads.DrawError(true, true, x, y);
						}
						if (timer % 150 == 0) {
							//Payloads.OpenSite();
						}
						break;
					case 2:
						if (timer % 2 == 0) {
							Payloads.Melt(random.Value!.Next(50, 500), random.Value!.Next(-10, 10), x, y);
						}
						if (timer % 5 == 0) {
							Payloads.Invert(random.Value!.Next(x / 2), random.Value!.Next(y / 2), random.Value!.Next(x), random.Value!.Next(y));
						}
						if (timer % 15 == 0) {
							Payloads.Blur(random.Value!.Next(-2, 2), random.Value!.Next(-2, 2), x, y, 0, 0, x, y, x, y);
						}
						if (timer % 10 == 0) {
							Payloads.Tunnel((int)(x / 100), (int)(y / 100), true, true, x, y);
						}
						if (timer % 1 == 0) {
							Payloads.DrawError(false, false, x, y);
							Payloads.DrawError(true, true, x, y);
						}
						if (timer % 1 == 0) {
							Payloads.Puzzle(500, x, y);
						}
						if (timer % 5 == 0) {
							Payloads.Rotate(random.Value!.Next(-100, 100), (int)(x / 100), (int)(y / 100), (int)(x / 50), (int)(y / 50), x, y);
						}
						if (timer % 20 == 0) {
							//Payloads.OpenSite();
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


//    global playAudio
//    playAudio = False
//    stage = -1

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