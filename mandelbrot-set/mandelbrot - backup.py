from PIL import Image
import numpy as np
import os
import time

def mandelbrot(c: complex, p: dict) -> float:
    # checks if c (complex number) is in the mandelbrot set
    # print(f'input: {c}')
    steps = 0
    z = c
    for i in range(p['iterations']):
        z = z**2 + c
        steps += 1
        if (abs(z) > p['threshold']):
            return steps
    return -1

def mandelbrot_numpy(c_values: np.ndarray, p: dict):
    # stores steps taken
    steps = np.zeros_like(c_values, dtype=float)
    # stores the current z values
    z_values = np.zeros_like(c_values)
    last_percent = -1

    for i in range(p['iterations']):
        mask = np.abs(z_values) <= p['threshold']
        z_values[mask] = z_values[mask]**2 + c_values[mask]
        steps[mask] += 1
        
        if p['numpy'] == 'all':
            percent_done = int((i / p['iterations']) * 100)
            if percent_done > last_percent:
                bar = f"[{'#' * int(p['bar_length'] * i / p['iterations'])}{'-' * (p['bar_length'] - int(p['bar_length'] * i / p['iterations']))}] {percent_done}% Done\r"
                print(bar, end='')
                last_percent = percent_done

    return np.where(np.abs(z_values) <= p['threshold'], -1, steps)

def load_image(dimensions: list) -> Image:
    w, h = dimensions[0], (dimensions[1])
    image = Image.new("RGB", (w, h), "black")
    return image

def interpolate_color(color_1: list, color_2: list, alpha: float):
    # interpolate between two RGB colors based on a decimal input (range 0-1)
    color_out = [
        int((1 - alpha) * c1 + alpha * c2)
        for c1, c2 in zip(color_1, color_2)
    ]
    return color_out

def interpolate_color_numpy(p: dict, alpha: np.ndarray):
    # interpolate between color_1 and color_2
    # -1 = color_fill
    # [0, 1] = color blending
    
    fill_mask = (alpha == -1)
    
    # make a copy of 'alpha' with all values set to color_fill
    color_out = np.full(alpha.shape + (3,), p['color_fill'], dtype=int)
    
    # perform alpha blending/smoothing calculation for pixels where 'alpha' is not -1
    alpha[~fill_mask] = (alpha[~fill_mask]**p['color_blend']) / (p['iterations']**p['color_blend'])
    
    # interpolate colors for pixels where 'alpha' is not -1
    alpha = np.clip(alpha[~fill_mask], 0, 1)  # ensure alpha is in the valid range [0, 1]
    
    # idk how the fuck this works but it does
    color_interpolated = np.array(p['color_1']) * (1 - alpha[:, np.newaxis]) + np.array(p['color_2']) * alpha[:, np.newaxis]

    # assign the interpolated colors to the corresponding pixels in the output array (overwriting color_fill)
    color_out[~fill_mask] = color_interpolated.astype(int)

    return color_out
            
def render_image(p: dict, image: Image) -> Image:
    w, h = p['image_dimensions'][0], p['image_dimensions'][1]
    
    # calculates the real & imaginary coordinates based on the ranges and image dimensions
    real_step_size = (p['real_range'][1] - p['real_range'][0]) / w
    real_values = np.linspace(p['real_range'][0] + real_step_size / 2, p['real_range'][1] - real_step_size / 2, w)
    imaginary_step_size = (p['imaginary_range'][0] - p['imaginary_range'][1]) / h
    imaginary_values = np.linspace(p['imaginary_range'][1] + imaginary_step_size / 2, p['imaginary_range'][0] - imaginary_step_size / 2, h)
    
    pixels = image.load()
    total_pixels = w * h
    last_percent = -1
    
    # colors the pixel based on iterations
    for x in range(w):
        for y in range(h):
            res = mandelbrot(complex(real_values[x], imaginary_values[y]), p)
            if res == -1:
                pixels[x, y] = (p['color_fill'][0], p['color_fill'][1], p['color_fill'][2])
            else:
                color = interpolate_color(p['color_1'], p['color_2'], (res**p['color_blend']) / (p['iterations']**p['color_blend']))
                pixels[x, y] = (color[0], color[1], color[2])
            
        percent_done = int((x * h / total_pixels) * 100)
        if percent_done > last_percent:
            bar = f"[{'#' * int(p['bar_length'] * x / w)}{'-' * (p['bar_length'] - int(p['bar_length'] * x / w))}] {percent_done}% Done\r"
            print(bar, end='')
            last_percent = percent_done
            
    return image

def render_image_numpy_row(p: dict, image: Image) -> Image:
    w, h = p['image_dimensions'][0], p['image_dimensions'][1]
    
    # calculates the real & imaginary coordinates based on the ranges and image dimensions
    real_step_size = (p['real_range'][1] - p['real_range'][0]) / w
    real_values = np.linspace(p['real_range'][0] + real_step_size / 2, p['real_range'][1] - real_step_size / 2, w)
    imaginary_step_size = (p['imaginary_range'][0] - p['imaginary_range'][1]) / h
    imaginary_values = np.linspace(p['imaginary_range'][1] + imaginary_step_size / 2, p['imaginary_range'][0] - imaginary_step_size / 2, h)
    
    pixels = image.load()
    total_pixels = w * h
    last_percent = -1
    
    # colors the pixel based on iterations
    for x in range(w):
        res = mandelbrot_numpy(np.array(real_values[x] + 1j * np.array(imaginary_values), dtype=np.complex128), p)
        for y in range(h):
            if res[y] == -1:
                pixels[x, y] = (p['color_fill'][0], p['color_fill'][1], p['color_fill'][2])
            else:
                color = interpolate_color(p['color_1'], p['color_2'], (res[y]**p['color_blend']) / (p['iterations']**p['color_blend']))
                pixels[x, y] = (color[0], color[1], color[2])
            
        percent_done = int((x * h / total_pixels) * 100)
        if percent_done > last_percent:
            bar = f"[{'#' * int(p['bar_length'] * x / w)}{'-' * (p['bar_length'] - int(p['bar_length'] * x / w))}] {percent_done}% Done\r"
            print(bar, end='')
            last_percent = percent_done
            
    return image

def render_image_numpy_all(p: dict, image: Image) -> Image:
    w, h = p['image_dimensions'][0], p['image_dimensions'][1]
    
    # calculates the real & imaginary coordinates based on the ranges and image dimensions
    real_step_size = (p['real_range'][1] - p['real_range'][0]) / w
    real_values = np.linspace(p['real_range'][0] + real_step_size / 2, p['real_range'][1] - real_step_size / 2, w)
    imaginary_step_size = (p['imaginary_range'][0] - p['imaginary_range'][1]) / h
    imaginary_values = np.linspace(p['imaginary_range'][1] + imaginary_step_size / 2, p['imaginary_range'][0] - imaginary_step_size / 2, h)
    
    # colors the pixel based on iterations
    c_values = real_values[:, np.newaxis] + 1j * imaginary_values
    res = mandelbrot_numpy(c_values, p)
    print(f'\nCalculations done in: {(time.time() - p['start_time']):.2f} seconds')
    
    # create np array that represents all the pixels
    data = np.zeros((w, h, 3), dtype=np.uint8)

    # interpolate colors for other points (accounts for res being -1)
    data[:, :] = interpolate_color_numpy(p, res)
    
    data = np.transpose(data, (1, 0, 2))
    
    # convert color array into image
    image = Image.fromarray(data, 'RGB')
            
    return image

def mirror_image(image: Image) -> Image:
    width, height = image.size
    
    # Create a new image with double the height
    mirrored_image = Image.new("RGB", (width, height * 2), "black")
    
    # Paste the original image onto the top and bottom halves
    mirrored_image.paste(image, (0, 0))
    mirrored_image.paste(image.transpose(Image.FLIP_TOP_BOTTOM), (0, height))
    
    return mirrored_image

def main():
    
    p = {
        'start_time': time.time(),
        'iterations': 200,
        'threshold': 2.0,
        # range for mandelbrot set in image
        'real_range': [-1.5, 0.5],
        'imaginary_range': [-1, 1],
        # size of image in pixels
        'image_dimensions': [1000, 1000],
        # image output path
        'output': os.path.join(os.path.dirname(__file__), './output.png'),
        # two colors to gradient between
        'color_1': (0, 0, 0),
        'color_2': (255, 0, 0),
        'color_fill': (0, 0, 0),
        # exponent factor of color easing
        'color_blend': 0.4,
        'bar_length': 40,
        # only calculate half image, and mirror after
        'mirror': True,
        # use numpy calculations to calculate every pixel at once (all, row, none)
        'numpy': 'all'
    }
    
    print(
    f'''
===================
Information:
Output Destination: {p['output']}
Image Dimensions: {p['image_dimensions'][0]}px by {p['image_dimensions'][1]}px
Iterations: {p['iterations']}
Threshold: {p['threshold']}
Real Range: {p['real_range']}
Imaginary Range: {p['imaginary_range']}
===================
    '''
    )
    
    # remove bottom half of dimensions to only generate top half of image
    if p['mirror']:
        p['image_dimensions'][1] = int(p['image_dimensions'][1] / 2)
        p['imaginary_range'][0] = 0
    
    image = load_image(p['image_dimensions'])
    
    if p['numpy'] == 'all':
        image = render_image_numpy_all(p, image)
    elif p['numpy'] == 'row':
        image = render_image_numpy_row(p, image)
    else:
        image = render_image(p, image)

        
    image = mirror_image(image) if p['mirror'] else image
    
    # calculate time elapsed
    print(f'\nTotal time taken: {(time.time() - p['start_time']):.2f} seconds')
    
    # save the image to disk
    image.save(p['output'])
    image.show()

if __name__ == "__main__":
    main()
