import binascii
import random
import io
from PIL import Image, ImageTk
import tkinter as tk
from tkinter import filedialog, Scale, ttk


def load_image():
    global img, img_label
    file_path = filedialog.askopenfilename()
    if file_path:
        img = Image.open(file_path)
        img.thumbnail((300, 300))  # Resize for display
        img_tk = ImageTk.PhotoImage(img)
        img_label.config(image=img_tk)
        img_label.image = img_tk


# def random_hex(length):
#     return ''.join(random.choice('1234567890abcdef') for _ in range(length))


def button_close():
    print('yey')

def main():
    # input_path = 'image-corruption/image.png'
    # output_path = 'image-corruption/glitched_image'
    # header_length = 256 # default, grabs value from header later
    # tail_length = 256
    # corrupt_iterations = 1
    # corrupt_amount = 100
    # offset_amount = 10
    # method = 2 # 1 = random byte corruption, 2 = byte offset shift
    
    global img, img_label, corrupted_img, corrupted_img_label
    
    # main window
    root = tk.Tk()
    root.title('Image Corruptor') # set title
    root.geometry('1600x800') # set size
    root.configure(bg = 'white') # set background color
    root.iconphoto(False, tk.PhotoImage(file = 'image-corruption/icon.png')) # set icon
    root.resizable(False, False) # disable resize
    style = ttk.Style()
    style.theme_use('clam')

    # image display labels
    img_label = tk.Label(root, text = 'Original Image')
    img_label.grid(row = 0, column = 0, padx = 10, pady = 10)

    corrupted_img_label = tk.Label(root, text = 'Corrupted Image')
    corrupted_img_label.grid(row = 0, column = 1, padx = 10, pady = 10)

    # buttons
    load_button = tk.Button(root, text='Load Image', command=load_image)
    load_button.grid(row = 1, column = 0, padx = 10, pady = 10)

    # corrupt_button = tk.Button(root, text="Corrupt Image", command=corrupt_image)
    # corrupt_button.grid(row=1, column=1, padx=10, pady=10)

    # corruption intensity slider
    corruption_slider = Scale(root, from_=1, to_=100, orient=tk.HORIZONTAL, label="Freaky Factor")
    corruption_slider.grid(row=2, column=0, columnspan=2, padx=10, pady=10)

    # run the main loop
    root.mainloop()
    

    # print('converting to bmp')

    # img = Image.open(input_path)
    # with io.BytesIO() as output:
    #     img.save(output, format='BMP')
    #     bmp_data = output.getvalue()

    # print('corrupting... ;w;')

    # # convert binary data to hex
    # hex_data = binascii.hexlify(bmp_data).decode()


    # # header_length = int(hex_data[4:6], 16)

    # # preserve the header and tail
    # header = hex_data[:header_length]
    # tail = hex_data[-tail_length:]
    # hex_data = hex_data[header_length:-tail_length]
    # hex_data_original = hex_data


    # # edit the hex data
    # if method == 1:
    #     for _ in range(corrupt_iterations):
    #         index = random.randint(0, len(hex_data) - corrupt_amount)
    #         hex_data = hex_data[:index] + random_hex(corrupt_amount) + hex_data[index + corrupt_amount:] 
    # elif method == 2:   
    #     for _ in range(corrupt_iterations):
    #         index = random.randint(0, len(hex_data) - corrupt_amount)
    #         # loop through each byte and offset it by offset_amount, if its over 0xff, it will loop back to 0x00
    #         for i in range(index, index + corrupt_amount, 2):
    #             byte = hex_data[i:i+2]
    #             byte = (int(byte, 16) + offset_amount) % 0xff
    #             byte = format(byte, '02x')
    #             hex_data = hex_data[:i] + byte + hex_data[i+2:]
            
            
            
    #         # for i in range(index, index + corrupt_amount, 2):
    #         #     byte = hex_data[i:i+2]
    #         #     byte = int(byte, 16) + offset_amount
    #         #     byte = format(byte, 'x')
    #         #     print(byte)
    #         #     hex_data = hex_data[:i] + byte + hex_data[i+2:]

    # # hex_data = random_hex(corrupt_amount) + hex_data[corrupt_amount:]

    # # convert edited hex data back to binary
    # binary_data = binascii.unhexlify(header + hex_data + tail)
    # binary_data_original = binascii.unhexlify(header + hex_data_original + tail) # for comparison

    # # save the binary data as a new image file
    # with io.BytesIO(binary_data) as input_io:
    #     img = Image.open(input_io)
    #     img.save(output_path + '.bmp', format='BMP')
        
    # with io.BytesIO(binary_data_original) as input_io:
    #     img = Image.open(input_io)
    #     img.save(output_path + '_original.bmp', format='BMP')

    # print('done :3')


    
if __name__ == "__main__":
    main()