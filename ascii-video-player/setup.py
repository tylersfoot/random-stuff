from PIL import Image, ImageDraw, ImageFont, ImageChops
from fontTools.ttLib import TTFont
from fontTools.unicode import Unicode
import time
from os import listdir
from os.path import isfile, join

# with open("character_brightness.txt", "w", encoding="utf-8") as f:




def has_glyph(c, font):
    # checks if the character will be rendered or
    # if it will be 'tofu' (a square box)
    for table in font['cmap'].tables:
        if ord(c) in table.cmap.keys():
            return True
    return False

    
def generate_character_images():
    start_time = time.time()
    fontsize = 200
    full_block_char = "█"
    font_path = "./jetbrainsmono-regular.ttf"

    font = ImageFont.truetype(font_path, fontsize)
    fontFT = TTFont(font_path) # fontTools font, only used for has_glyph()
    full_block_w = font.getbbox(full_block_char)[2]
    full_block_h = font.getbbox(full_block_char)[3]
    results = [0, 0, 0] # total, generated, skipped
    
    # render tofu character to compare/mask against E000 alt
    img_tofu = Image.new("RGB", (full_block_w, full_block_h), (0, 0, 0))
    draw = ImageDraw.Draw(img_tofu)
    draw.text((0, 0), chr(0x10FFFF), font=font, fill=(255, 255, 255))

    min = 0x0000
    max = 0x20000 #0x110000
    print(f"\nGenerating {max-min} character images...\n")
    for i in range(min, max):
        progress = int((results[0]) / (max-min) * 100)+1
        bar = '=' * progress + ' ' * (100 - progress)
        print(f'\r[{bar}]   {results[0]+1}/{(max-min)}    ', end='')
        
        results[0] += 1
        c = chr(i) # char
        o = str(ord(c)).zfill(6) # unicode codepoint
        h = "0x" + hex(ord(c))[2:].zfill(5) # unicode hex padded to 4 digits
        bb = font.getbbox(c) # bounding box of the character
        
        # flags for skipping:
        if (
            (not c.isprintable())
            or (bb[0] < 0)
            or (bb[1] < 0)
            or (bb[2] > full_block_w)
            or (bb[3] > full_block_h)
            or (bb[0] == bb[2])
            or (bb[1] == bb[3])
            or (not has_glyph(c, fontFT))
            ):
            results[2] += 1
            continue
        
        img_char = Image.new("RGB", (full_block_w, full_block_h), (0, 0, 0))
        draw = ImageDraw.Draw(img_char)
        draw.text((0, 0), c, font=font, fill=(255, 255, 255))
        if ImageChops.difference(img_char, img_tofu).getbbox() is None:
            # rendered as tofu, skip
            results[2] += 1
            continue
        img_char.save("character_images/" + o + "-" + h + ".png")
        results[1] += 1
    
    print("\n\nImage generation complete!")
    print("Generated characters: " + str(results[1]) + " / " + str(results[0]))
    print("Skipped characters:   " + str(results[2]) + " / " + str(results[0]))
    print("Time taken: " + str(round(time.time() - start_time, 2)) + " seconds")
    print()
    


def process_character_images():
    start_time = time.time()
    image_folder_path = "character_images/"
    brightness_data_path = "character_brightness.txt"
    brightness_data = ""
    
    files = [f for f in listdir(image_folder_path) if (isfile(join(image_folder_path, f)) and f.endswith('.png'))]
    # files.sort()
    file_count = len(files)
    c = 0 # counter
    print(f"\nProcessing {file_count} character images...\n")
    for file in files:
# 008988-0x0231c.png
# 008989-0x0231d.png
# 008990-0x0231e.png

        image = Image.open(image_folder_path + file)
        image = image.convert("L") # convert to grayscale, just in case
        total_brightness = 0
        pixel_count = image.size[0] * image.size[1] # total number of pixels in the image
        # loop through all pixels
        for x in range(image.size[0]):
            for y in range(image.size[1]):
                pixel = image.getpixel((x, y))
                total_brightness += pixel
        brightness = total_brightness / pixel_count / 255 # average brightness
        brightness = format(round(brightness, 4), '.4f')
        # write data
        brightness_data += file[:14] + "-" + str(brightness) + "\n"
        
        c += 1
        progress = int(c / file_count * 100)
        bar = '=' * progress + ' ' * (100 - progress)
        print(f'\r[{bar}]   {c}/{file_count}    ', end='')
        
    with open(brightness_data_path, "w", encoding="utf-8") as f:
        f.write(brightness_data)
        
        
    print("\n\nCharacter images processed! Wrote to " + brightness_data_path + " (" + str(len(brightness_data)) + " chars)")
    
    print("Processed character images: " + str(c) + " / " + str(file_count))
    print("Time taken: " + str(round(time.time() - start_time, 2)) + " seconds")
    print()



def tester():
    chars = [] # array of 3 element arrys [ordinal, hex, brightness]
    with open ("character_brightness.txt", "r", encoding="utf-8") as f:
        lines = f.readlines()
        for line in lines:
            chars += [line.strip().split("-")]
    chars.sort(key=lambda x: float(x[2]), reverse=True)
    print(chars)
    data = ""
    data2 = ""
    with open("character_brightness_sorted.txt", "w", encoding="utf-8") as f:
        for i in range(0, len(chars)):
            data += str(chars[i][0]) + "-" + str(chars[i][1]) + "-" + str(chars[i][2]) + "\n"
            data2 += chr(int(chars[i][0]))
        f.write(data)
        with open("character_brightness_sorted_chars.txt", "w", encoding="utf-8") as f2:
            f2.write(data2)
            
    print("Sorted character brightness data written to character_brightness_sorted.txt")
            


if __name__=="__main__":
    # generate_character_images()
    process_character_images()
    tester()
    # font = TTFont('./jetbrainsmono-regular.ttf')
    # for table in font['cmap'].tables:
    #     print(f"Platform: {table.platformID}, Encoding: {table.platEncID}, Entries: {len(table.cmap)}")


# bbmaxX = 0
# bbmaxY = 0
# bbminX = 0
# bbminY = 0
# maxXchar = ""
# maxYchar = ""
# minXchar = ""
# minYchar = ""
# xcountmax = 0
# ycountmax = 0
# xcountmin = 0
# ycountmin = 0

# for i in range(0x110000):
#     ch = chr(i)
#     if ch.isprintable():
#         bb = font.getbbox(ch)
#         if bb[0] < font.getbbox(full_block_char)[0]:
#             xcountmin += 1
#         if bb[1] < font.getbbox(full_block_char)[1]:
#             ycountmin += 1
#         if bb[2] > font.getbbox(full_block_char)[2]:
#             xcountmax += 1
#         if bb[3] > font.getbbox(full_block_char)[3]:
#             ycountmax += 1
            
#         if bb[0] < bbminX:
#             bbminX = bb[0]
#             minXchar = ch
#         if bb[1] < bbminY:
#             bbminY = bb[1]
#             minYchar = ch
#         if bb[2] > bbmaxX:
#             bbmaxX = bb[2]
#             maxXchar = ch
#         if bb[3] > bbmaxY:
#             bbmaxY = bb[3]
#             maxYchar = ch
    
# print("maxX: " + str(bbmaxX) + " for char: " + maxXchar + " " + str(ord(maxXchar)) + " count: " + str(xcountmax))
# print("maxY: " + str(bbmaxY) + " for char: " + maxYchar + " " + str(ord(maxYchar)) + " count: " + str(ycountmax))
# print("minX: " + str(bbminX) + " for char: " + minXchar + " " + str(ord(minXchar)) + " count: " + str(xcountmin))
# print("minY: " + str(bbminY) + " for char: " + minYchar + " " + str(ord(minYchar)) + " count: " + str(ycountmin))




# c = 0
# with open("unicode_sample.txt", "w", encoding="utf-8") as f:
#     for i in range(0x110000):
#         ch = chr(i)
#         if ch.isprintable():
#             c += 1
#             f.write(ch)
#             if c > 150:
#                 f.write("\n")
#                 c = 0
