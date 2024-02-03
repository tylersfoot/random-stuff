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
		private int sampleTime;
		private int sampleLength;

		public ByteBeatPlayer(int sampleRate, int sampleTime) {
			this.sampleRate = sampleRate;
			this.sampleTime = sampleTime;
			this.sampleLength = sampleRate * sampleTime;
		}

		public void GenerateBuffer(string expression) {
			byte[] buffer = new byte[sampleLength];

			for (int t = 0; t < sampleLength; t++) {
				// Example evaluation of specific expressions
				int sampleValue = expression switch {
					"1" => (t * (t >> 9) * (t >> 11)) % 127,
					"2" => (t * (t >> 9) * (t >> 8) & (t >> 4)) % 127,
					"3" => (t * (t >> 5) * (t >> 8)) >> 3,
					_ => throw new ArgumentException($"expression index out of bound: {expression}"),
				};

				buffer[t] = (byte)sampleValue;
			}

			buffers.Add(buffer);
			//Console.WriteLine($"Generated buffer for expression: {expression}");
		}

		public void PlayBuffer(int index) {
			if (index < 0 || index >= buffers.Count) {
				Console.WriteLine("Buffer index out of range.");
				return;
			}
			//Console.WriteLine($"Playing bytebeat expression {index} {buffers[index]}");

			byte[] buffer = buffers[index];
			var waveProvider = new BufferedWaveProvider(new WaveFormat(sampleRate, 8, 1));
			waveProvider.BufferLength = sampleLength + 1;
			//using (var waveOut = new WaveOutEvent()) {
			using (var waveOut = new WaveOutEvent()) {
				waveOut.PlaybackStopped += (sender, e) => {
					//Console.WriteLine($"Playback completed for index {index}.");
					// You can set a flag here to exit the while loop if needed
				};
				waveProvider.AddSamples(buffer, 0, buffer.Length);
				waveOut.Init(waveProvider);
				waveOut.Play();

				while (waveOut.PlaybackState == PlaybackState.Playing) {
					if (waveProvider.BufferedBytes == 0) {
						waveOut.Stop();
					}
				}
			}
			//Console.WriteLine($"Playing over! {index}");
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
