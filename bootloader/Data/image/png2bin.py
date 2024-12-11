import sys, math, struct, os, glob
from PIL import Image

doscolors = [
	(0x00, 0x00, 0x00), # 0
	(0x00, 0x00, 0xa8), # 1
	(0x00, 0xa8, 0x00), # 2
	(0x00, 0xa8, 0xa8), # 3
	(0xa8, 0x00, 0x00), # 4
	(0xa8, 0x00, 0xa8), # 5
	(0xa8, 0xa8, 0x00), # 6
	(0xa8, 0xa8, 0xa8), # 7
	(0x54, 0x54, 0x54), # 8
	(0x54, 0x54, 0xff), # 9
	(0x54, 0xff, 0x54), # 10
	(0x54, 0xff, 0xff), # 11
	(0xff, 0x54, 0x54), # 12
	(0xff, 0x54, 0xff), # 13
	(0xff, 0xff, 0x54), # 14
	(0xff, 0xff, 0xff), # 15
]

input_files = sorted(glob.glob(sys.argv[1]))
output_file = sys.argv[-1]


def color_distance(a, b):
	return math.sqrt( (a[0]-b[0])**2 + (a[1]-b[1])**2 + (a[2]-b[2])**2 )
	
def nearest_color(color):
	nearest = 0
	
	for i in range(len(doscolors)):
		if color_distance(color, doscolors[i]) < color_distance(color, doscolors[nearest]):
			nearest = i
	
	return nearest

def apply_dos_palette(img, imgframe):
	"""Applies the DOS palette to the image and saves the visual preview."""
	w, h = img.size
	output_img = Image.new("RGB", (w, h))
	for y in range(h):
		for x in range(w):
			original_color = img.getpixel((x, y))
			dos_color = doscolors[nearest_color(original_color)]
			output_img.putpixel((x, y), dos_color)
	# Save the output image with '_dos' appended to the filename
	output_img.save(f"dos_{os.path.basename(imgframe)}")
	print(f"DOS palette applied to {imgframe} and saved as {os.path.splitext(imgframe)[0]}_dos.png")


buf = b""

for imgframe in input_files:
	img = Image.open(imgframe).convert("RGB")
	if img.size != (80, 50):
		print(f"Warning: {imgframe} has unexpected dimensions {img.size}")
	w, h = img.size

	# Apply DOS palette and save the visual representation
	apply_dos_palette(img, imgframe)
	
	for y in range(0, h, 2):
		for x in range(w):
			b = (nearest_color(img.getpixel((x, y))) << 4)
			if y + 1 < h:
				b |= nearest_color(img.getpixel((x, y + 1)))
			buf += bytes([b])  # Append as bytes

	img.close()

with open(output_file, "wb") as out:
	out.write(buf)
