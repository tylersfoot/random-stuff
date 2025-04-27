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