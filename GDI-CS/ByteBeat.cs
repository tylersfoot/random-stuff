using System;
using System.Collections.Generic;
using NAudio.Wave;
// dotnet add package NAudio --version 2.2.1
using System.Threading;
using System.Linq.Expressions;

namespace GDI_CS {
	class ByteBeatPlayer {
 
		private List<byte[]> buffers = new List<byte[]>();
		private int sampleRate;

		public ByteBeatPlayer(int sampleRate) {
			this.sampleRate = sampleRate;
		}

		public void GenerateBuffer(string expression, int durationSeconds) {
			int totalSamples = sampleRate * durationSeconds;
			byte[] buffer = new byte[totalSamples];

			//for (int t = 0; t < totalSamples; t++) {
			//	// Simplified example using one of the expressions directly
			//	// This part should be replaced with actual expression evaluation
			//	byte sample = expression switch {
			//		"(t * (t >> 9) * (t >> 11)) % 127" => (byte)((t * (t >> 9) * (t >> 11)) % 127),
			//		"(t * (t >> 9) * (t >> 8) & (t >> 4)) % 127" => (byte)((t * (t >> 9) * (t >> 8) & (t >> 4)) % 127),
			//		"(t * (t >> 5) * (t >> 8)) >> 3" => (byte)((t * (t >> 5) * (t >> 8)) >> 3),
			//		_ => 0, // Default case for unhandled expressions
			//	};
			//	buffer[t] = sample;
			//}

			buffers.Add(buffer);
			Console.WriteLine($"Generated buffer for expression: {expression}");
		}

		public void PlayBuffer(int index) {
			if (index < 0 || index >= buffers.Count) {
				Console.WriteLine("Buffer index out of range.");
				return;
			}
			Console.WriteLine($"Playing bytebeat expression: {buffers[index]}");

			byte[] buffer = buffers[index];
			var waveProvider = new BufferedWaveProvider(new WaveFormat(sampleRate, 8, 1));
			using (var waveOut = new WaveOutEvent()) {
				waveProvider.AddSamples(buffer, 0, buffer.Length);
				waveOut.Init(waveProvider);
				waveOut.Play();

				// Wait for the buffer to finish playing
				while (waveOut.PlaybackState == PlaybackState.Playing) {
					Thread.Sleep(100);
				}
			}
		}

		public void PlayAll() {
			using (var waveOut = new WaveOutEvent()) {
				foreach (var buffer in buffers) {
					var waveProvider = new BufferedWaveProvider(new WaveFormat(sampleRate, 8, 1));
					waveProvider.AddSamples(buffer, 0, buffer.Length);
					waveOut.Init(waveProvider);
					waveOut.Play();

					while (waveOut.PlaybackState == PlaybackState.Playing) {
						Thread.Sleep(100);
					}
				}
			}
		}
	}
}
