import binascii
import random
import io
from PIL import Image

def random_hex(length):
    return ''.join(random.choice('1234567890abcdef') for _ in range(length))

input_path = 'image-corruption/image.png'
output_path = 'image-corruption/glitched_image'
header_length = 256 # default, grabs value from header later
tail_length = 256
corrupt_iterations = 1
corrupt_amount = 100
offset_amount = 10
method = 2 # 1 = random, 2 = offset

print('converting to bmp')

img = Image.open(input_path)
with io.BytesIO() as output:
    img.save(output, format='BMP')
    bmp_data = output.getvalue()

print('corrupting... ;w;')

# convert binary data to hex
hex_data = binascii.hexlify(bmp_data).decode()


# header_length = int(hex_data[4:6], 16)

# preserve the header and tail
header = hex_data[:header_length]
tail = hex_data[-tail_length:]
hex_data = hex_data[header_length:-tail_length]
hex_data_original = hex_data


# edit the hex data
if method == 1:
    for _ in range(corrupt_iterations):
        index = random.randint(0, len(hex_data) - corrupt_amount)
        hex_data = hex_data[:index] + random_hex(corrupt_amount) + hex_data[index + corrupt_amount:] 
elif method == 2:   
    for _ in range(corrupt_iterations):
        index = random.randint(0, len(hex_data) - corrupt_amount)
        # loop through each byte and offset it by offset_amount, if its over 0xff, it will loop back to 0x00
        for i in range(index, index + corrupt_amount, 2):
            byte = hex_data[i:i+2]
            byte = (int(byte, 16) + offset_amount) % 0xff
            byte = format(byte, '02x')
            hex_data = hex_data[:i] + byte + hex_data[i+2:]
        
        
        
        # for i in range(index, index + corrupt_amount, 2):
        #     byte = hex_data[i:i+2]
        #     byte = int(byte, 16) + offset_amount
        #     byte = format(byte, 'x')
        #     print(byte)
        #     hex_data = hex_data[:i] + byte + hex_data[i+2:]

# hex_data = random_hex(corrupt_amount) + hex_data[corrupt_amount:]

# convert edited hex data back to binary
binary_data = binascii.unhexlify(header + hex_data + tail)
binary_data_original = binascii.unhexlify(header + hex_data_original + tail) # for comparison

# save the binary data as a new image file
with io.BytesIO(binary_data) as input_io:
    img = Image.open(input_io)
    img.save(output_path + '.bmp', format='BMP')
    
with io.BytesIO(binary_data_original) as input_io:
    img = Image.open(input_io)
    img.save(output_path + '_original.bmp', format='BMP')

print('done :3')
