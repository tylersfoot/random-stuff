#!.\.venv\Scripts\python.exe

# stdlib
import os, sys, ctypes, json, atexit, subprocess
from time import perf_counter, sleep
import numpy as np
from bisect import bisect_left
from contextlib import contextmanager
# imaging
from PIL import Image, ImageDraw, ImageFont, ImageStat
from fontTools.ttLib import TTFont
# video
import cv2
from matplotlib import font_manager
from moviepy import VideoFileClip, AudioFileClip
# misc
import tkinter as tk
from tkinter import ttk, simpledialog, filedialog, font as tkfont
import inquirer
from dataclasses import dataclass
from tqdm import tqdm


@dataclass
class Config:
    input_video_path: str
    font_name:          str
    font_path:          str
    font_size:          int
    output_video_path:  str
    output_video_fps:   int
    progress_bar_size:  int
    script_path:        str
    

CHAR_SETS = {
    "ASCII":              [chr(i) for i in range(0x20, 0x7F)],
    "Extended ASCII":     [chr(i) for i in range(0x20, 0x100)],
    "Lowercase Alphabet": list("abcdefghijklmnopqrstuvwxyz"),
    "Uppercase Alphabet": list("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    "Digits":             list("0123456789"),
    "Symbols":            list("[#$%&()*+,-./0123456789<=>?@[\\]^_`{|}~]"),
    "Hex":                list("0123456789ABCDEF"),
    "Ramp":               list(" .:-=+*#%@"),
}

@contextmanager
def suppress_stdout_stderr():
    with open(os.devnull, 'w') as devnull:
        old_stdout = sys.stdout
        old_stderr = sys.stderr
        sys.stdout = devnull
        sys.stderr = devnull
        try:
            yield
        finally:
            sys.stdout = old_stdout
            sys.stderr = old_stderr

        
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
    

def render_video(allowed_cp, allow_all):
    """Render the video using the given character sets, then re-attach audio."""
    start = perf_counter()
    
    out_path        = settings["output_video_path"]
    vdir(os.path.dirname(out_path))
    silent_path     = sdir("temp", "silent.mp4")
    font_size       = settings["font_size"]
    brightness_path = sdir("font_data", f"{settings["font_name"]}.json")
    font            = ImageFont.truetype(settings["font_path"], font_size)
    cell_w, cell_h  = font.getbbox("█")[2:]
    
    # load character brightness data
    with open(brightness_path, "r", encoding="utf-8") as f:
        bright_map = json.load(f)

    # sort by brightness, filter by allowed characters
    chars = sorted(
        ( (int(k,16), float(v)) for k,v in bright_map.items()
          if allow_all or int(k,16) in allowed_cp ),
        key=lambda kv: kv[1]
    )
    brights = [b for _, b in chars] # brightness values
    lut = [             # lookup table for index 0..255 -> codepoint
        chars[min(bisect_left(brights, i/255), len(chars)-1)][0]
        for i in range(256)
    ]
    
    glyphs: dict[int, np.ndarray] = {}
    for cp,_ in chars:
        ch = chr(cp)
        img = Image.new("RGB", (cell_w, cell_h), "black")
        ImageDraw.Draw(img).text((0,0), ch, font=font, fill="white")
        glyphs[cp] = np.array(img)
    
    # build an array of all glyphs stacked:
    #   glyph_arr.shape == (N, cell_h, cell_w, 3)
    #   and build a map cp2idx: codepoint -> index in glyph_arr
    glyph_list = [glyphs[cp] for cp,_ in chars]
    glyph_arr = np.stack(glyph_list, axis=0) # stack glyphs into a single array
    cp2idx = {cp: i for i, (cp,_) in enumerate(chars)} # codepoint to index mapping
    lut_idx = np.array([cp2idx[lut[i]] for i in range(256)], dtype=np.int32)
    
    # open source video and prepare writer for silent video
    cap       = cv2.VideoCapture(settings["input_video_path"])
    src_w     = int(cap.get(cv2.CAP_PROP_FRAME_WIDTH))
    src_h     = int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
    src_fps   = cap.get(cv2.CAP_PROP_FPS)
    out_fps   = settings["output_video_fps"] or src_fps
    out_fps   = max(1, min(int(src_fps), int(out_fps)))
    ratio     = src_fps / out_fps
    total_in  = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    total_out = int(total_in / ratio)
    
    cols = src_w // cell_w
    rows = src_h // cell_h
    
    fourcc = cv2.VideoWriter_fourcc(*'mp4v')
    writer = cv2.VideoWriter(silent_path, fourcc, out_fps, (cols * cell_w, rows * cell_h))

    accum, rendered = 0.0, 0
    print(f"Rendering ASCII video: {src_w}x{src_h} ({cols}x{rows}@{out_fps}FPS)")
    pbar = tqdm(total=total_out, desc="Rendering", unit="frames")
    while True:
        # loop through the video frames
        ret, frame = cap.read()
        if not ret: # no more frames
            break
        accum += 1.0
        
        # only process when we've “collected” enough source frames
        if accum < ratio:
            continue
        accum -= ratio
        
        # grayscale and resize frame into cell size
        gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
        small = cv2.resize(gray, (cols, rows), interpolation=cv2.INTER_AREA)
        
        # map every small[r,c] directly to an index into glyph_arr
        idxs   = lut_idx[small] # shape (rows,cols)
        blocks = glyph_arr[idxs] # shape (rows,cols,cell_h,cell_w,3)
        blocks = blocks.transpose(0,2,1,3,4) # (rows,cell_h,cols,cell_w,3)
        canvas = blocks.reshape(rows*cell_h, cols*cell_w, 3)
                
        writer.write(canvas)
        rendered += 1
        pbar.update(1)
        
    cap.release()
    writer.release()
    pbar.close()
    
    print(f"Done! Rendered {rendered}/{total_out} frames in {perf_counter()-start:.2f}s")
    print(f"Video saved to {out_path}\n")

    
def generate_character_images():
    """Generate character images from the given font."""
    start = perf_counter()
    
    font_size = 200 # resolution/size of the images
    font = ImageFont.truetype(settings["font_path"], font_size)

    w, h = (font.getbbox("█")[2], font.getbbox("█")[3])
    print(f"Image size: {w}x{h}")

    cmap = {}
    for table in TTFont(settings["font_path"])["cmap"].tables:
        cmap.update(table.cmap)
    codepoints = sorted(cmap.keys())

    total = len(codepoints)
    gen = 0
    print(f"\nGenerating characters...")
    for cp in tqdm(codepoints, desc="Characters", total=total, unit="chars"):
        c = chr(cp) # char
        x0, y0, x1, y1 = font.getbbox(c)
        
        # if the character fits in the image
        if (x0 >= 0) and (y0 >= 0) and (x1 <= w) and (y1 <= h):
            img = Image.new("RGB", (w, h), "black")
            ImageDraw.Draw(img).text((0, 0), c, font=font, fill="white")
            img.save(sdir("temp", f"0x{cp:06X}.png"))
            gen += 1
    
    print(f"\nSucessfully generated {gen}/{total} characters! ({perf_counter() - start:.2f}s)\n")
    
    
def process_character_images():
    def mean_brightness(path):
        with Image.open(path) as img:
            img = img.convert("L") # grayscale, just in case
            stat = ImageStat.Stat(img) # mean color value
            return stat.mean[0] / 255.0 
        
    start = perf_counter()
    vdir(sdir("font_data"))
    font_data_path = sdir("font_data", f"{settings["font_name"]}.json")
    brightness_data = {}
    
    files = [f for f in os.listdir(sdir("temp")) if (os.path.isfile(sdir("temp", f)) and f.lower().endswith('.png'))]
    
    file_count = len(files)
    print(f"Calculating font brightness...")
    for file in tqdm(files, desc="Brightness", total=file_count, unit="img"):
        b = mean_brightness(sdir("temp", file))
        brightness_data[file[:-4]] = f"{b:.4f}"
        
    with open(font_data_path, "w", encoding="utf-8") as f:
        json.dump(brightness_data, f, indent=2)
        
    print(f"\nCalculated {file_count} characters' brightness for the {settings["font_name"]} font! ({perf_counter() - start:.2f}s)\n")


def check_processed_font(font_name) -> bool:
    """Check if a font has been processed into brightness data."""
    path = sdir("font_data", f"{font_name}.json")
    return os.path.isfile(path) and os.path.getsize(path) > 0


def clear_temp():
    """Create/clear the temporary folder."""
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
    """Check if a file is a valid video."""
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
        "input_video_path": "",
        "font_name": "", # font family name
        "font_path": "", # path to ttf/otf font
        "font_size": 10, # for rendering
        # "terminal_width": 100,
        # "terminal_height": 30,
        "output_video_path": "", # where rendered video will be saved
        "output_video_fps": 0,
        "progress_bar_size": 100,
        "script_path": os.path.dirname(os.path.abspath(__file__)),
    }
    
    # config = Config(
    #     input_video_path = "",
    #     font_name = "",
    #     font_path = "",
    #     font_size = 10,
    #     output_video_path = "",
    #     output_video_fps = 0,
    #     progress_bar_size = 100,
    #     script_path = os.path.dirname(os.path.abspath(__file__))
    # )
    
    clear_temp()

    root = tk.Tk()
    root.withdraw()
    
    
    # get video path
    if len(sys.argv) == 2:
        settings["input_video_path"] = sys.argv[1]
    elif len(sys.argv) > 2:
        print("Multiple files detected, only the first one will be used.")
        settings["input_video_path"] = sys.argv[1]
    else:
        print("Please select a video file:")
        settings["input_video_path"] = filedialog.askopenfilename(
            filetypes=[("Video files", "*.mp4;*.mov;*.webm;*.mkv;*.wmv;*.avi;*.flv;*.mpeg;*.movie;*.m4v")],
            title="Select a video file",
        )

    while verify_video(settings["input_video_path"]) == False:
        print("Invalid video file, please select a valid video file:")
        settings["input_video_path"] = filedialog.askopenfilename(
            filetypes=[("Video files", "*.mp4;*.mov;*.webm;*.mkv;*.wmv;*.avi;*.flv;*.mpeg;*.movie;*.m4v")],
            title="Select a video file",
        )
    print(f"Video path selected: {settings["input_video_path"]}")
        
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
    
    root.destroy()
    # output name based on input video name
    settings["output_video_path"] = sdir("output", f"{os.path.splitext(os.path.basename(settings["input_video_path"]))[0]}.mp4") 
    
    questions = [
        inquirer.Text(
            'fps',
            message="Please enter the output video FPS (leave blank to match input)",
            validate=lambda _, x: x.isdigit() or x == "",
            default=settings["output_video_fps"],
        ),
        inquirer.Checkbox(
            "character_set",
            message="Please choose a character set (or multiple to combine)",
            choices=["ASCII", "Extended ASCII", "Lowercase Alphabet", "Uppercase Alphabet",
                     "Digits", "Symbols", "Hex", "Ramp", "All"],
        ),
    ]

    answers = inquirer.prompt(questions)
    settings["output_video_fps"] = int(answers["fps"]) if answers["fps"] else 0
    allow_all = "All" in answers["character_set"] or not answers["character_set"]
    allowed_cp = {
        ord(c)
        for cs in answers["character_set"]
        for c in CHAR_SETS.get(cs, [])
    } if not allow_all else None
    
    render_video(allowed_cp, allow_all)
    subprocess.run(["explorer", "/select,", settings["output_video_path"]])

    
if __name__ == "__main__":
    main()
    for i in range(5, 0, -1):
        print(f"Exiting in {i} seconds...", end="\r")
        sleep(1)
    sys.exit(0)
