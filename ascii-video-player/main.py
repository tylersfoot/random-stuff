#!.\.venv\Scripts\python.exe

# stdlib
import os, sys, ctypes, json, atexit
from time import perf_counter, sleep

# tkinter
import tkinter as tk
from tkinter import ttk, simpledialog, filedialog, font as tkfont

# imaging
from PIL import Image, ImageDraw, ImageFont, ImageStat
from fontTools.ttLib import TTFont

# video
import cv2
from matplotlib import font_manager

        
        
def sdir(*sub: any) -> str:
    """Make a subdirectory from the script path."""
    return os.path.join(settings["script_path"], *sub)

def vdir(path):
    """Ensure the directory exists."""
    os.makedirs(path, exist_ok=True)
    
    
def progress_bar(i, total, size):
    """Display a progress bar."""
    pct = (i + 1) * 100 // total
    bar = "=" * pct + " " * (size - pct)
    print(f"\r[{bar}]  {i+1}/{total}  {pct}% ", end="")
        
    
def generate_character_images():
    """Generate character images from the given font."""
    start = perf_counter()
    
    font_size = 200 # resolution/size of the images
    font = ImageFont.truetype(settings["font_path"], font_size)

    w, h = (font.getbbox("█")[2], font.getbbox("█")[3])

    cmap = {}
    for table in TTFont(settings["font_path"])["cmap"].tables:
        cmap.update(table.cmap)
    codepoints = sorted(cmap.keys())

    total = len(codepoints)
    step = max(1, total//400)
    gen = 0
    print(f"\nGenerating characters...")
    for i, cp in enumerate(codepoints): # cp = unicode codepoint (int)
        c = chr(cp) # char
        x0, y0, x1, y1 = font.getbbox(c)
        
        # if the character fits in the image
        if (0 <= x0 < x1 <= w) and (0 <= y0 < y1 <= h):
            img = Image.new("RGB", (w, h), "black")
            ImageDraw.Draw(img).text((0, 0), c, font=font, fill="white")
            img.save(sdir("temp", f"0x{cp:06X}.png"))
            gen += 1
        
        if (i % step == 0) or (i == total - 1):
            progress_bar(i, total, 100)
    
    print(f"\nSucessfully generated {gen}/{total} characters! ({perf_counter()-start:.2f}s)\n")
    
    
def process_character_images():
    def mean_brightness(path):
        with Image.open(path) as img:
            img = img.convert("L") # grayscale, just in case
            stat = ImageStat.Stat(img) # mean color value
            return stat.mean[0] / 255.0 
        
    start = perf_counter()
    image_folder_path = sdir("temp")
    vdir(sdir("font_data"))
    font_data_path = sdir("font_data", f"{settings["font_name"]}.json")
    brightness_data = {}
    
    files = [f for f in os.listdir(sdir("temp")) if (os.path.isfile(sdir("temp", f)) and f.lower().endswith('.png'))]
    
    file_count = len(files)
    step = max(1, file_count//400)
    print(f"Calculating font brightness...")
    for i, file in enumerate(files):
        b = mean_brightness(sdir("temp", file))
        brightness_data[file[:-4]] = f"{b:.4f}"
        
        if (i % step == 0) or (i == file_count - 1):
            progress_bar(i, file_count, 100)
        
    with open(font_data_path, "w", encoding="utf-8") as f:
        json.dump(brightness_data, f, indent=2)
        
    print(f"\nCalculated {file_count} characters' brightness for the {settings["font_name"]} font! ({perf_counter()-start:.2f}s)\n")


def check_processed_font(font_name) -> bool:
    # checks if a font has been processed into brightness data
    path = sdir("font_data", f"{font_name}.json")
    return os.path.isfile(path) and os.path.getsize(path) > 0

     
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


def clear_temp():
    # create/clear the temp folder
    temp_path = sdir("temp")
    vdir(temp_path)
    for f in os.listdir(temp_path):
        file_path = sdir("temp", f)
        try:
            if os.path.isfile(file_path) and file_path.endswith('.png'):
                os.remove(file_path)
        except Exception as e:
            print(f'Failed to delete {file_path}. Reason: {e}')


def verify_video(file_path: str) -> bool: 
    """ Check if a video file is valid. """
    cap = cv2.VideoCapture(file_path)
    opened = cap.isOpened()
    cap.release()
    return opened


@atexit.register
def exit_handler():
    # clear temp on exit
    try:
        clear_temp()
    except Exception:
        pass
    

class FontChooser(simpledialog.Dialog):
    # class to create the font chooser window
    # based on tkFontChooser
    def body(self, master):
        self.fonts = {} # name/family, file path
        for f in sorted(tkfont.families()):
            # filter for monospaced
            try:
                if not tkfont.Font(family=f).metrics('fixed'):
                    continue
            except tk.TclError:
                continue
                
            # filter for fonts with a file path
            try:
                path = font_manager.findfont(
                    font_manager.FontProperties(family=f),
                    fallback_to_default=False
                )
                if path and os.path.exists(path):
                    self.fonts[f] = path
            except Exception:
                continue
            
        self.geometry("400x200") # window size
        self.resizable(False, False) # disable resizing
        
        # font list
        self.combo = ttk.Combobox(master, values=sorted(self.fonts.keys()), state="readonly")
        self.combo.set(sorted(self.fonts.keys())[0])
        self.combo.grid(row=0, column=0, columnspan=2, padx=50, pady=5, sticky="ew")
        self.combo.bind("<<ComboboxSelected>>", self._update_preview)
        
        # font preview
        self.preview_font = tkfont.Font(family=sorted(self.fonts.keys())[0], size=16)
        self.preview = ttk.Label(
            master, text="The quick brown fox", font=self.preview_font,
            relief="groove", anchor="center"
        )
        self.preview.grid(row=1, column=0, columnspan=2, padx=5, pady=(10,10), ipadx=10, ipady=5, sticky="ew")
        
        # button to choose font
        btn = ttk.Button(master, text="Browse Font", command=self._browse_font)
        btn.grid(row=2, column=0, columnspan=2, pady=(5,10), padx=80, ipady=3, sticky="ew")
        
        return self.combo
    
    def _update_preview(self, event=None):
        # change the preview's font family on selection
        self.preview_font.configure(family=self.combo.get())

    def apply(self):
        selected_font = self.combo.get()

        self.result = selected_font, self.fonts[selected_font]
        
    def _browse_font(self):
        path = filedialog.askopenfilename(
            filetypes=[("TrueType/OpenType Fonts", "*.ttf;*.ttc;*.otf")],
            title="Load a TrueType/OpenType font file"
        )
        if not path:
            return

        # load font
        try:
            ctypes.windll.gdi32.AddFontResourceExW(path, 0x10, 0)

            # extract font family name
            tt = TTFont(path)
            name_rec = tt['name'].getName(nameID=1, platformID=3, platEncID=1)
            family = name_rec.toUnicode()
            tt.close()
        except Exception as e:
            print(f"Failed to load font: {e}")
            return
        
        vals = sorted(list(self.combo['values']) + [family])
        self.combo['values'] = vals
        self.combo.set(family)

        self._update_preview()

        # record it into loaded_fonts
        self.fonts[family] = path
    

def main():
    global settings
    settings = {
        # input video properties
        "input_video_path": "",
        "input_video_size": [0, 0], # w, h in px
        "input_video_fps": 0,
        "input_video_frames": 0,
        # font properties
        "font_name": "", # font family name
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
    
    clear_temp()

    root = tk.Tk()
    root.withdraw()
    
    
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
    
        
    settings["font_name"], settings["font_path"] = FontChooser(root, title="Choose a monospaced font").result
    print(f"Font selected: {settings["font_name"]}")

    # check if font has been processed
    if not check_processed_font(settings["font_name"]):
        print("Font has not been processed yet, processing now!")
        # process the given font, render character images,
        # calculate brightness, and save results
        generate_character_images()
        process_character_images()
        print("Font processed!")
    
    # TODO set output video settings and then render video

    


if __name__ == "__main__":
    main()
    for i in range(5, 0, -1):
        print(f"Exiting in {i} seconds...", end="\r")
        sleep(1)
    sys.exit(0)
