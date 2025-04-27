#!.\.venv\Scripts\python.exe

from PIL import Image, ImageDraw, ImageFont, ImageChops
from fontTools.ttLib import TTFont
from fontTools.unicode import Unicode
import time
import os
from os.path import isfile, join
import sys
import tkinter as tk
from tkinter import ttk, filedialog, simpledialog
from tkinter import font as tkfont
import cv2
import winreg

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
    
    files = [f for f in os.listdir(image_folder_path) if (isfile(join(image_folder_path, f)) and f.endswith('.png'))]

    file_count = len(files)
    c = 0 # counter
    print(f"\nProcessing {file_count} character images...\n")
    for file in files:
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

    

def process_font(font_path: str) -> bool:
    # process the given font, render character images,
    # calculate brightness, and save results
    return True


def check_processed_font(font_path: str) -> bool:
    # checks if a font has been processed into brightness data
    path = os.path.join(settings["script_path"], "fonts", "data", font_path.split("/")[-1].split(".")[0] + ".txt")
    if os.path.exists(path):
        with open(path, "r", encoding="utf-8") as f:
            lines = f.readlines()
            if len(lines) > 0:
                return True
    return False

     
def get_video_properties(file_path: str) -> dict:
    # get video properties using OpenCV
    capture = cv2.VideoCapture(file_path)
    if not capture.isOpened():
        return None
    
    properties = {
        "width": int(capture.get(cv2.CAP_PROP_FRAME_WIDTH)),
        "height": int(capture.get(cv2.CAP_PROP_FRAME_HEIGHT)),
        "fps": int(capture.get(cv2.CAP_PROP_FPS)),
        "frames": int(capture.get(cv2.CAP_PROP_FRAME_COUNT)),
    }
    
    capture.release()
    return properties


def get_installed_fonts():
    # gets the pc's installed fonts from the registry
    # copies fonts to local folder for use
    fonts = {}
    with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts") as key:
        for i in range(0, winreg.QueryInfoKey(key)[1]):
            name, path, _ = winreg.EnumValue(key, i)
            fonts[name] = os.path.join(r"C:/Windows/Fonts", path)
    # filter fonts to TrueType, clean names
    fonts = {k: v for k, v in fonts.items() if (v.endswith((".ttf", ".ttc", ".otf")) and "TrueType" in k)}
    fonts = {(k.replace(" (TrueType)", "")): v for k, v in fonts.items()}
    
    # copy fonts to folder
    fonts_path = os.path.join(settings["script_path"], "fonts", "raw")
    if not os.path.exists(fonts_path):
        os.makedirs(fonts_path)
    for name, path in fonts.items():
        if not os.path.exists(fonts_path + "/" + name + path[-4:]):
            try:
                with open(path, "rb") as f:
                    with open(fonts_path + "/" + name + path[-4:], "wb") as f2:
                        f2.write(f.read())
            except Exception as e:
                print(f"Failed to copy font {name} from path {path} to {fonts_path + "/" + name + path[-4:]}: {e}")
                
    # fonts list with local paths
    fonts_local = {}
    for name, path in fonts.items():
        fonts_local[name] = name + path[-4:]
    return fonts_local

def clear_temp():
    # create/clear the temp folder
    temp_path = os.path.join(settings["script_path"], "temp")
    if not os.path.exists(temp_path):
        os.makedirs(temp_path)
    for f in os.listdir(temp_path):
        file_path = os.path.join(temp_path, f)
        try:
            if os.path.isfile(file_path) and file_path.endswith('.png'):
                os.remove(file_path)
        except Exception as e:
            print(f'Failed to delete {file_path}. Reason: {e}')


def verify_font(font_path: str) -> bool:
    # check if the font file is valid
    if os.path.exists(font_path) and font_path.endswith((".ttf", ".otf")):
        try:
            TTFont(font_path)
            return True
        except Exception as e:
            return False
    return False


def verify_video(file_path: str) -> bool: 
    # check if the file is a valid video file using OpenCV
    if file_path.endswith((".mp4", ".mov", ".webm", ".mkv", ".wmv", ".avi," ".flv", ".mpeg", ".movie", ".m4v")):
        capture = cv2.VideoCapture(file_path)
        opened = capture.isOpened()
        capture.release()
        return opened
    return False





class FontChooser(simpledialog.Dialog):
    # class to create the font chooser window
    def body(self, master):
        # get monospaced fonts
        fonts = sorted(f for f in tkfont.families()
                if tkfont.Font(family=f).metrics('fixed'))
        
        # font list
        ttk.Label(master, text="font:").grid(row=0, column=0, padx=5, pady=5)
        self.combo = ttk.Combobox(master, values=fonts, state="readonly")
        self.combo.set(fonts[0])
        self.combo.grid(row=0, column=1, padx=5, pady=5)
        # self.combo.bind("<<ComboboxSelected>>", self._update_preview)
        self.combo.bind("<<ComboboxSelected>>", lambda event=None: self.preview_font.configure(family=self.combo.get()))
        
        # font preview
        self.preview_font = tkfont.Font(family=fonts[0], size=20)
        self.preview = ttk.Label(
            master, text="The quick brown fox", font=self.preview_font,
            relief="groove", anchor="center"
        )
        self.preview.grid(row=1, column=0, columnspan=2, padx=5, pady=(0,10), sticky="ew")
        return self.combo

    def apply(self):
        self.result = self.combo.get()



def main():
    global settings
    settings = {
        # input video properties
        "input_video_path": "",
        "input_video_size": [0, 0], # w, h in px
        "input_video_fps": 0,
        "input_video_frames": 0,
        # font properties
        "font_path": "./jetbrainsmono-regular.ttf", # path to ttf/otf font
        "font_size": 200, # for rendering
        # terminal/console settings
        "terminal_width": 100,
        "terminal_height": 30,
        # export/output video settings
        "output_video_path": "", # where rendered video will be saved
        "output_video_size": [0, 0], # w, h in px
        "output_video_fps": 0,
        # path to this python file
        "script_path": os.path.dirname(os.path.abspath(__file__)),
    }
    

    root = tk.Tk()
    root.withdraw()
    
    font = FontChooser(root, title="Choose a monospaced font").result
    print("You picked:", font)

     

    
    
    # # get video path
    # if len(sys.argv) == 2:
    #     settings["input_video_path"] = sys.argv[1]
    # elif len(sys.argv) > 2:
    #     print("Multiple files detected, only the first one will be used.")
    #     settings["input_video_path"] = sys.argv[1]
    # else:
    #     print("Please select a video file:")
    #     settings["input_video_path"] = filedialog.askopenfilename(
    #         filetypes=[("Video files", "*.mp4;*.mov;*.webm;*.mkv;*.wmv;*.avi;*.flv;*.mpeg;*.movie;*.m4v")],
    #         title="Select a video file",
    #     )

    # while verify_video(settings["input_video_path"]) == False:
    #     print("Invalid video file, please select a valid video file:")
    #     settings["input_video_path"] = filedialog.askopenfilename(
    #         filetypes=[("Video files", "*.mp4;*.mov;*.webm;*.mkv;*.wmv;*.avi;*.flv;*.mpeg;*.movie;*.m4v")],
    #         title="Select a video file",
    #     )
    # print(f"Video path selected: {settings["input_video_path"]}")
    # # get video properties
    # video_properties = get_video_properties(settings["input_video_path"])
    # if video_properties is not None:
    #     settings["input_video_size"] = [video_properties["width"], video_properties["height"]]
    #     settings["input_video_fps"] = video_properties["fps"]
    #     settings["input_video_frames"] = video_properties["frames"]
    # else:
    #     ValueError("Could not extract video properties.")
    #     return
    # print(f"Video properties: {settings['input_video_size'][0]}x{settings['input_video_size'][1]} @ {settings['input_video_fps']} fps, {settings['input_video_frames']} frames")
    
    
    # get_installed_fonts()
    
    # print("Please select a font:")
    # # NOTE: i have to do this fucking stdout devnull shit bc windows left a stray printf in comdlg32.dll
    # fd = os.dup(1)
    # with open(os.devnull, 'w') as dn:
    #     os.dup2(dn.fileno(), 1) # redirect stdout to null
    #     settings["font_path"] = filedialog.askopenfilename(
    #         filetypes=[("TrueType/OpenType Fonts", "*.ttf;*.ttc;*.otf")],
    #         title="Select a font",
    #         initialdir=os.path.join(settings["script_path"], "fonts", "raw"),
    #     )
    # os.dup2(fd, 1) 
    # os.close(fd)

    # while verify_font(settings["font_path"]) == False:
    #     print("Invalid font, please select a valid font:")
    #     fd = os.dup(1) 
    #     with open(os.devnull, 'w') as dn:
    #         os.dup2(dn.fileno(), 1) # redirect stdout to null
    #         settings["font_path"] = filedialog.askopenfilename(
    #             filetypes=[("TrueType/OpenType Fonts", "*.ttf;*.ttc;*.otf")],
    #             title="Select a font",
    #             initialdir=os.path.join(settings["script_path"], "fonts", "raw"),
    #         )
    #     os.dup2(fd, 1) 
    #     os.close(fd)
        
    # print(f"Font selected: {settings["font_path"]}")
    # # check if font has been processed
    # if not check_processed_font(settings["font_path"]):
    #     process_font(settings["font_path"])
    #     print("Font processed!")
    
    # output video settings

        
    
    
    


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"An error occurred: {e}")
    for i in range(5, 0, -1):
        print(f"Exiting in {i} seconds...", end="\r")
        time.sleep(1)
    os._exit(0)
   
    
# generate_character_images()
# process_character_images()
# tester()
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
