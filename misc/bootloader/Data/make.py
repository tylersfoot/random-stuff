import sys, math, struct, glob, os
from PIL import Image
import midi

# DOS color palette
doscolors = [
    (0x00, 0x00, 0x00), (0x00, 0x00, 0xa8), (0x00, 0xa8, 0x00), (0x00, 0xa8, 0xa8),
    (0xa8, 0x00, 0x00), (0xa8, 0x00, 0xa8), (0xa8, 0xa8, 0x00), (0xa8, 0xa8, 0xa8),
    (0x54, 0x54, 0x54), (0x54, 0x54, 0xff), (0x54, 0xff, 0x54), (0x54, 0xff, 0xff),
    (0xff, 0x54, 0x54), (0xff, 0x54, 0xff), (0xff, 0xff, 0x54), (0xff, 0xff, 0xff)
]

# Helper functions for color conversion
def color_distance(a, b):
    return math.sqrt((a[0]-b[0])**2 + (a[1]-b[1])**2 + (a[2]-b[2])**2)

def nearest_color(color):
    return min(range(len(doscolors)), key=lambda i: color_distance(color, doscolors[i]))

# Convert PNG frames to binary
def convert_images_to_binary(input_files):
    buf = b""
    for imgframe in input_files:
        img = Image.open(imgframe).convert("RGB")
        if img.size != (80, 50):
            print(f"Warning: {imgframe} has unexpected dimensions {img.size}")
        w, h = img.size
        
        for y in range(0, h, 2):
            for x in range(w):
                b = (nearest_color(img.getpixel((x, y))) << 4)
                if y + 1 < h:
                    b |= nearest_color(img.getpixel((x, y + 1)))
                buf += bytes([b])  # Append as bytes
        img.close()
    return buf

# Convert MIDI file to binary
def convert_midi_to_binary(midi_file):
    pattern = midi.read_midifile(midi_file)
    buf = b""

    def pitchconv(pitch):
        return int(round(1193180.0 / (2**((pitch-69)/12.0)*440), 0))

    for event in pattern[1]:
        if isinstance(event, midi.NoteOnEvent):
            if event.velocity == 0:
                tick_duration = int(round(event.tick / 48.0, 0))
                pitch = pitchconv(event.pitch)
                buf += bytes([(pitch & 0xff), ((tick_duration << 5) | (pitch >> 8))])
    return buf

# Read text file as binary
def convert_text_to_binary(text_file):
    with open(text_file, "r") as f:
        text_data = f.read().encode("ascii") + b"\x00"  # Null-terminated
    return text_data

# Main script logic
if len(sys.argv) != 5:
    print("Usage: python script.py <image_pattern> <midi_file> <text_file> <output_file>")
    sys.exit(1)

image_pattern = sys.argv[1]
midi_file = sys.argv[2]
text_file = sys.argv[3]
output_file = sys.argv[4]

# Convert inputs to binary data
image_files = sorted(glob.glob(image_pattern))
image_data = convert_images_to_binary(image_files)
midi_data = convert_midi_to_binary(midi_file)
text_data = convert_text_to_binary(text_file)

# Calculate offsets and sizes for the header
text_offset = 16  # Header size
text_size = len(text_data)
midi_offset = text_offset + text_size
midi_size = len(midi_data)
frame_offset = midi_offset + midi_size
frame_size = len(image_data)

# Create header (4 bytes each for offset and size)
header = struct.pack("<IIIIII", text_offset, text_size, midi_offset, midi_size, frame_offset, frame_size)

# Write combined binary output
with open(output_file, "wb") as out:
    out.write(header)       # Write header
    out.write(text_data)    # Write text data
    out.write(midi_data)    # Write MIDI data
    out.write(image_data)   # Write image data

print(f"Binary data written to {output_file}")
