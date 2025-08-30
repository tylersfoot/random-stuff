# file for storing old versions of code/functions for the hell of it

def process_character_images() -> bool:
    start_time = time.time()
    image_folder_path = os.path.join(settings["script_path"], "temp")
    font_data_folder_path = os.path.join(settings["script_path"], "font_data")
    font_data_path = os.path.join(font_data_folder_path, f"{settings["font_name"]}.json")
    brightness_data = {}
    
    if not os.path.exists(font_data_folder_path):
        os.makedirs(font_data_folder_path)
    
    files = [f for f in os.listdir(image_folder_path) if (isfile(os.path.join(image_folder_path, f)) and f.lower().endswith('.png'))]
    
    file_count = len(files)
    c = 0 # counter
    print(f"Calculating font brightness...")
    for file in files:
        image = Image.open(os.path.join(image_folder_path, file))
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
        brightness_data[file[:-4]] = str(brightness)
        # brightness_data += file[:14] + "-" + str(brightness) + "\n"
        
        c += 1
        progress = int(c / file_count * 100)
        bar = '=' * progress + ' ' * (100 - progress)
        print(f'\r[{bar}]   {c}/{file_count}    ', end='')
        
    with open(font_data_path, "w", encoding="utf-8") as f:
        f.write(json.dumps(brightness_data))
        
        
    print(f"\nSucessfully calculated brightness data for {c} characters! (Took {round(time.time() - start_time, 2)}s)\n")
    return True



def worker_setup(font_path, font_size, tofu_path):
    global shared_resources
    shared_resources["font"] = ImageFont.truetype(font_path, font_size)
    shared_resources["fontFT"] = TTFont(font_path)
    shared_resources["tofu_image"] = Image.open(tofu_path)
    
    
def render_save_character(cp, dimensions, temp_folder_path):
    # renders the character (from codepoint) and saves image
    global shared_resources
    font = shared_resources["font"]
    fontFT = shared_resources["fontFT"]
    tofu_image = shared_resources["tofu_image"]
    
    c = chr(cp) # char
    h = f"0x{cp:06X}"
    bb = font.getbbox(c) # bounding box of the character
    # flags for skipping:
    if (
        (not c.isprintable())
        or (bb[0] < 0)
        or (bb[1] < 0)
        or (bb[2] > dimensions[0])
        or (bb[3] > dimensions[1])
        or (bb[0] == bb[2])
        or (bb[1] == bb[3])
        or (not has_glyph(c, fontFT))
        ):
        return cp, False

    img = Image.new("RGB", dimensions, "black")
    ImageDraw.Draw(img).text((0, 0), c, font=font, fill="white")
    if ImageChops.difference(img, tofu_image).getbbox() is None:
        # rendered as tofu, skip
        return cp, False
    img.save(os.path.join(temp_folder_path, f"{h}_v2.png"))
    return cp, True




    with ProcessPoolExecutor(initializer=worker_setup, initargs=(font_path, font_size, tofu_path)) as exe:
        futures = [exe.submit(
            render_save_character, cp, dimensions, temp_folder_path) for cp in range(codepoints)]
        print(f"\r[{' ' * 100}]   0/{len(futures)}   0% ", end="")
        for done, fut in enumerate(as_completed(futures), 1):
            cp, success = fut.result()   # Get the result of the finished task
            if success:
                generated.append(cp)
                
            # progress bar
            if (done % (round(len(futures)/400)) == 0) or (done == len(futures) - 1):
                progress = int(((cp + 1) / (codepoints)) * 100)
                bar = "=" * progress + " " * (100 - progress)
                print(f"\r[{bar}]   {cp+1}/{(codepoints)}   {progress}% ", end="")
                
                
                
                
                
# def generate_character_images() -> bool:
#     start_time = time.time()
#     fontsize = 10 # resolution/size of the images
#     full_block_char = "█"

#     font = ImageFont.truetype(settings["font_path"], fontsize)
#     fontFT = TTFont(settings["font_path"]) # fontTools font, only used for has_glyph()
#     full_block_w = font.getbbox(full_block_char)[2]
#     full_block_h = font.getbbox(full_block_char)[3]
#     results = [0, 0, 0] # total, generated, skipped
    
#     # render tofu character to compare/mask against E000 alt
#     img_tofu = Image.new("RGB", (full_block_w, full_block_h), (0, 0, 0))
#     draw = ImageDraw.Draw(img_tofu)
#     draw.text((0, 0), chr(0x10FFFF), font=font, fill=(255, 255, 255))

#     min = 0x0000
#     max = 0x110000 #0x110000 #0x20000
#     print(f"\nGenerating characters...")
#     for i in range(min, max):
#         progress = int((results[0]) / (max-min) * 100)+1
#         bar = '=' * progress + ' ' * (100 - progress)
#         print(f'\r[{bar}]   {results[0]+1}/{(max-min)}   {progress}% ', end='')
        
#         results[0] += 1
#         c = chr(i) # char
#         h = "0x" + hex(ord(c))[2:].zfill(5) # unicode hex padded to 4 digits
#         bb = font.getbbox(c) # bounding box of the character
        
#         # flags for skipping:
#         if (
#             (not c.isprintable())
#             or (bb[0] < 0)
#             or (bb[1] < 0)
#             or (bb[2] > full_block_w)
#             or (bb[3] > full_block_h)
#             or (bb[0] == bb[2])
#             or (bb[1] == bb[3])
#             or (not has_glyph(c, fontFT))
#             ):
#             results[2] += 1
#             continue
        
#         img_char = Image.new("RGB", (full_block_w, full_block_h), (0, 0, 0))
#         draw = ImageDraw.Draw(img_char)
#         draw.text((0, 0), c, font=font, fill=(255, 255, 255))
#         if ImageChops.difference(img_char, img_tofu).getbbox() is None:
#             # rendered as tofu, skip
#             results[2] += 1
#             continue
#         img_char.save(os.path.join(settings["script_path"], "temp", f"{h}.png"))
#         results[1] += 1
    
#     print(f"\nSucessfully generated {str(results[1])}/{str(results[0])} characters! (" + str(round(time.time() - start_time, 2)) + "s)\n")
#     return True





# def get_installed_fonts():
#     # gets the pc's installed fonts from the registry
#     # copies fonts to local folder for use
#     fonts = {}
#     key_paths = [
#         (winreg.HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts"),
#         (winreg.HKEY_CURRENT_USER,   r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts")
#     ]
#     for key_path in key_paths: 
#         with winreg.OpenKey(key_path[0], key_path[1]) as key:
#             for i in range(winreg.QueryInfoKey(key)[1]):
#                 name, path, _ = winreg.EnumValue(key, i)
#                 if path.lower().endswith(('.ttf','.otf','.ttc')):
#                     fonts[name] = os.path.join(r"C:\Windows\Fonts", path)
#                 # if path.lower().endswith((".ttf", ".ttc", ".otf")) and "TrueType" in name:
#                 #     fonts[name.replace(" (TrueType)", "")] = os.path.join(r"C:\Windows\Fonts", path)

#     # filter for monospace fonts
#     fonts = {fam: filepath for fam, filepath in fonts.items() if tkfont.Font(family=fam).metrics("fixed")}
#     return fonts
    
    # copy fonts to folder
    # fonts_path = os.path.join(settings["script_path"], "fonts", "raw")
    # if not os.path.exists(fonts_path):
    #     os.makedirs(fonts_path)
    # for name, path in fonts.items():
    #     if not os.path.exists(fonts_path + "/" + name + path[-4:]):
    #         try:
    #             with open(path, "rb") as f:
    #                 with open(fonts_path + "/" + name + path[-4:], "wb") as f2:
    #                     f2.write(f.read())
    #         except Exception as e:
    #             print(f"Failed to copy font {name} from path {path} to {fonts_path + "/" + name + path[-4:]}: {e}")
                
    # fonts list with local paths
    # fonts_local = {}
    # for name, path in fonts.items():
    #     fonts_local[name] = name + path[-4:]
    # return fonts_local
    
    
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
    
    
    
    
    
def verify_font(font_path: str) -> bool:
    # check if the font file is valid
    if os.path.exists(font_path) and font_path.endswith((".ttf", ".otf")):
        try:
            TTFont(font_path)
            return True
        except Exception as e:
            return False
    return False