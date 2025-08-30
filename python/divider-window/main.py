import tkinter as tk

# Create a top-level window
root = tk.Tk()
root.attributes('-topmost', True)  # Keep the window always on top

# Configure the window to be thin and tall
root.geometry("1000x1000")  # Set the width and height as desired

# Make the window transparent
root.attributes('-alpha', 0.01)  # Set the transparency level (0.0 to 1.0, where 0.0 is fully transparent)

# Run the tkinter main loop
root.mainloop()
