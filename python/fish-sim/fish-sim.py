#!.\.venv\Scripts\python.exe

from time import perf_counter
import numpy as np
from numpy import random
import imageio.v2 as imageio
# import matplotlib
# matplotlib.use("Agg")
import matplotlib.pyplot as plt
from tqdm import tqdm
import cv2


def acosd(x):
    return np.degrees(np.arccos(x)) 

def atan2d(x, y):
    return np.degrees(np.arctan2(x, y))

def sind(x):
    return np.sin(np.radians(x))

def cosd(x):
    return np.cos(np.radians(x))


def world_to_screen(x, y, win_scale, width, height):
    scale_x = width / (2 * win_scale)
    scale_y = height / (2 * win_scale)
    x_screen = int((x + win_scale) * scale_x)
    y_screen = int((win_scale - y) * scale_y)  # flip y for OpenCV
    return x_screen, y_screen


def draw_unit_grid(frame, win_scale, frame_width, frame_height, spacing=1, color=(50, 50, 50)):
    half_width = win_scale
    half_height = win_scale

    for x in np.arange(-half_width, half_width + spacing, spacing):
        x_px1, y_px1 = world_to_screen(x, -half_height, win_scale, frame_width, frame_height)
        x_px2, y_px2 = world_to_screen(x, +half_height, win_scale, frame_width, frame_height)
        cv2.line(frame, (x_px1, y_px1), (x_px2, y_px2), color, 1)

    for y in np.arange(-half_height, half_height + spacing, spacing):
        x_px1, y_px1 = world_to_screen(-half_width, y, win_scale, frame_width, frame_height)
        x_px2, y_px2 = world_to_screen(+half_width, y, win_scale, frame_width, frame_height)
        cv2.line(frame, (x_px1, y_px1), (x_px2, y_px2), color, 1)



# if __name__ == "__main__":
start = perf_counter()
calc_time = 0 # time it took to calculate
render_time = 0 # time it took to render
t = 20 # simulation time, [sec]
fps = 30 # frames per second
dt = 1.0 / fps # time step, [sec]
# dt = 0.1 # time step, [sec]
frames = int(t / dt) # number of frames = time of animation / time for each frame
video_live = False
video_render = True

# ______________Initial parameters________________

sim_speed = 5

N = 3 # number of fish
alpha = 360 # vision volume, [deg]
rr = 1 # zone of repulsion [units]
ro = 5 # zone of orientation [units]
ra = 15 # zone of attraction [units]
omega = 40 * sim_speed # max rotation rate, [deg/sec]
speed = 3 * sim_speed # speed of each fish, [units/sec]

# INITIALIZE FISH

# ____random____
r = 5 * random.rand(3, N) - 2.5 # random positions around (0,0)
v = random.rand(3, N) - 0.5 # random velocity directions

# ____example 1____
# r[:, 0] = [5, 0, 0]
# r[:, 1] = [-5, 0, 0]
# v[:, 0] = [-1, 0, 0]
# v[:, 1] = [1, -1, 0]

#____example 2____
r[:, 0] = [5, 0, 0]
r[:, 1] = [-5, 0, 0]
r[:, 2] = [0, 0, 0]
v[:, 0] = [0, 1, 0]
v[:, 1] = [0, 1, 0]
v[:, 2] = [0, -1, 0]



# r[:, 0] = [5, 0, 0]
# r[:, 1] = [4, 5, -4]
# r[:, 2] = [-1, 5, -1]

# print(r)

# r = np.array([
#     [5,  4, -1],
#     [0,  5,  5],
#     [0, -4, -1]
# ])

# print(r)


# v[:, 0] = [0, 1, 0]
# v[:, 1] = [1, 7, -9]
# v[:, 2] = [-1, 1, 1]

# v = np.array([
#     [0,  1, -1],
#     [1,  7,  1],
#     [0, -9,  1]
# ]) 


# r = np.array([
#     [5,  4, -1, -3,  3,  4, -1, 2],
#     [0,  5,  5, -1,  0, -1,  3, 1],
#     [0, -4, -1,  4,  3,  2,  1, 5]
# ], dtype=float)

# v = np.array([
#     [0,  1, -1,  2,  4, -3,  1,  0],
#     [1,  7,  1,  2,  4, -1, -5, -3],
#     [0, -9,  1,  6,  2,  3,  1,  1]
# ], dtype=float) 

# r[2, :] = 0
# v[2, :] = 0

for i in range(N): # normalize these vectors
    v[:, i] = v[:, i] / np.linalg.norm(v[:, i])


if video_live:
    win_scale = 15
    window = [-win_scale, win_scale, -win_scale, win_scale]
    writer = imageio.get_writer("movie.mp4", fps=1/dt)
    
    fig = plt.figure(figsize=(6.4, 6.4), dpi=100)
    ax = plt.gca()
    ax.set_aspect('equal', adjustable='box')
    scatter = plt.plot([], [], 'o')[0]
    quivers = plt.quiver(r[0, :], r[1, :], v[0, :], v[1, :], angles='xy', scale_units='xy', scale=1)

if video_render:
    win_scale = 15  # half-width of visible world (world goes from -15 to +15 in x and y)
    frame_width = 1080
    frame_height = 1080
    frame_scale = min(frame_width, frame_height)
    fourcc = cv2.VideoWriter_fourcc(*"mp4v")
    writer = cv2.VideoWriter("movie.mp4", fourcc, 1/dt, (frame_width, frame_height))

# matrices for information about the system, recorded each frame
r_group = np.zeros((3, frames)) # COM Position
v_group = np.zeros((3, frames)) # COM velocity
p_group = np.zeros((3, frames)) # COM linear momentum (equal to velocity if mass of system = 1)
h_group = np.zeros((3, frames)) # angular momentum about COM
r_inter = np.zeros((3, frames)) # distance of each fish from COM, used in calculating h_group

# ______________________Loop for each frame to calculate____________________
for frame in tqdm(range(frames), desc="Frames", unit="frame"):
    calc_start = perf_counter()
    # print(f"frame {frame}, r avg = {np.mean(r[:2, :], axis=1)}")
    dir = np.zeros((3, N)) # desired direction for each fish at the  of the frame
    
    for n in range(N):
        dis = np.zeros((3, N))
        dirTemp = np.zeros((3, 1)) # temporary desired direction
        tempIndex = np.zeros(N)
        temp2 = np.zeros((3, N))
        temp3 = np.zeros((3, N))
        inrr = False
        inro = False
        inra = False
        angInit = 0 # initial angle calculated from velocity vector
        angTarg = 0 # target angle calcualted from dir vector
        dif1 = 0
        dif2 = 0
        angFinal = 0 # final angle to turn to
        
        for i in range(N):
            dis[:, i] = r[:, i] - r[:, n]
            d = np.linalg.norm(dis[:, i])
            
            # if i == n:
            #     tempIndex[i] = 3
            #     continue

            
            #_____blind spot______
            denom = np.linalg.norm(v[:, n]) * np.linalg.norm(dis[:, i])
            if denom != 0:
                dot_val = np.dot(v[:, n], dis[:, i]) / denom
                # clamp to valid acos input range [-1, 1]
                dot_val = np.clip(dot_val, -1.0, 1.0)
                angle = acosd(dot_val)
            else:
                angle = 0

            # angle = acosd(np.dot(v[:, n], dis[:, i]) / np.linalg.norm(v[:, n]) / np.linalg.norm(dis[:, i]));  
            
            if angle > 0.5 * alpha:
                tempIndex[i] = 3
            
            if (d <= rr) and (tempIndex[i] != 3):
                tempIndex[i] = 0
            elif (d <= ro) and (tempIndex[i] != 3):
                tempIndex[i] = 1
            elif (d <= ra) and (tempIndex[i] != 3):
                tempIndex[i] = 2
            else:
                tempIndex[i] = 3
                
            tempIndex[n] = 3
            
        # repulsion zone
        
        fishCount = 0
        for i in range(N):
            if tempIndex[i] == 0:
                inrr = True
                fishCount = fishCount + 1
        
        if inrr:
            for i in range(N):
                if tempIndex[i] == 0:
                    temp2[:, i] = r[:, i]
            target = np.array([
                np.sum(temp2[0, :] / fishCount),
                np.sum(temp2[1, :] / fishCount),
                0
            ])
            dir[:, n] = (r[:, n] - target) / np.linalg.norm(r[:, n] - target)
            continue

        # orientation zone
        
        fishCount = 0
        for i in range(N):
            if tempIndex[i] == 1:
                inro = True
                fishCount += 1
                
        if inro:
            for i in range(N):
                if tempIndex[i] == 1:
                    temp2[:, i] = v[:, i]
                    
            target = np.array([
                np.sum(temp2[0, :] / fishCount),
                np.sum(temp2[1, :] / fishCount),
                0
            ])
            # dirTemp = target / np.linalg.norm(target)
            norm = np.linalg.norm(target)
            if norm != 0:
                dirTemp = target / norm
            else:
                dirTemp = np.zeros(3)

            
        # attraction zone
        
        fishCount = 0
        for i in range(N):
            if tempIndex[i] == 2:
                inra = True
                fishCount += 1
    
        if inra:
            for i in range(N):
                if tempIndex[i] == 2:
                    temp3[:, i] = r[:, i]
                    
            target = np.array([
                np.sum(temp3[0, :] / fishCount),
                np.sum(temp3[1, :] / fishCount),
                0
            ])
            # dir[:, n] = -(r[:, n] - target) / np.linalg.norm(r[:, n] - target)
            vec = r[:, n] - target
            norm = np.linalg.norm(vec)
            if norm != 0:
                dir[:, n] = -vec / norm
            else:
                dir[:, n] = np.zeros(3)
            
        # combine zones
        
        if inro:
            if inra:
                dir[:, n] = (0.8 * dir[:, n] + 0.2 * dirTemp)
                # dir[:, n] = dir[:, n] / np.linalg.norm(dir[:, n])
                norm = np.linalg.norm(dir[:, n])
                if norm != 0:
                    dir[:, n] = dir[:, n] / norm
                else:
                    dir[:, n] = np.zeros(3)
            else:
                dir[:, n] = dirTemp
                
    # print(f"fish dirs: {[np.round(np.linalg.norm(dir[:, i]), 3) for i in range(N)]}")

    #__________________________________MOVEMENT______________________________
    
    for i in range(N):
        
        # no fish in any zone, move in current direction
        if np.allclose(dir[:, i], 0):
            r[:, i] = r[:, i] + speed * dt * v[:, i]
            continue  

        angFinal = 0
        
        # current velocity angle (deg)
        angInit = atan2d(v[1, i], v[0, i])
        if angInit < 0:
            angInit = angInit + 360
        

        # target angle (deg)
        angTarg = atan2d(dir[1, i], dir[0, i])
        if angTarg < 0:
            angTarg = angTarg + 360
        

        # calculates clockwise and counterclockwise distances between angles
        dif1 = angTarg - angInit
        dif2 = 360 - abs(dif1)
        if dif1 > 0:
            # first distance is pos, set dif1 -> pos, dif2 -> neg
            counterclockwise = dif1
            clockwise = dif2
        else:
            # first distance is neg, set abs(dif1) -> neg, dif2 -> pos
            counterclockwise = dif2
            clockwise = abs(dif1)
        
        
        # checks which direction is shortest, applies movement in that direction
        # omega * dt for smooth turning relative to time
        if counterclockwise < clockwise:
            if counterclockwise < omega * dt:
                angFinal = angTarg
            else:
                angFinal = angInit + omega * dt
        else:
            if clockwise < omega * dt:
                angFinal = angTarg
            else:
                angFinal = angInit - omega * dt
                
        angFinal = angFinal % 360
        # if i == 0:
            # print(f"fish {i}: angInit={angInit:.2f}, angTarg={angTarg:.2f}, angFinal={angFinal:.2f}")


        
        # set new velocity direction, and update position (move)
        v[:, i] = np.array([cosd(angFinal), sind(angFinal), 0])
        r[:, i] = r[:, i] +  speed * dt * v[:, i]
    

    # _____________________________MOMENTUM_ETC_________________________

    # center of mass / mean position of all fish
    r_group[:, frame] = np.array([np.mean(r[0, :]), np.mean(r[1, :]), 0])
    
    # average velocity / mean velocity of all fish
    v_group[:, frame] = speed * np.array([np.mean(v[0, :]), np.mean(v[1, :]), 0])

    # linear momentum / mean velocity of all fish
    p_group[:, frame] = (speed / N) * np.array([sum(v[0, :]), sum(v[1, :]), 0])

    # angular momentum
    # for each fish, calculate distance from COM, then cross product with velocity
    for j in range(N):
        r_inter[0, j] = r[0, j] - r_group[0, frame]
        r_inter[1, j] = r[1, j] - r_group[1, frame]
        
        h_group[:, frame] = h_group[:, frame] + speed/N * np.cross(r_inter[:, j], v[:, j])
    
    calc_time += perf_counter() - calc_start
    #_______Animation stuff__________
    if video_live:
        render_start = perf_counter()
        # update scatter plot (dots for fish)
        scatter.set_data(r[0, :], r[1, :])

        # update quiver (arrows for velocity)
        quivers.set_offsets(r[:2, :].T)
        quivers.set_UVC(v[0, :], v[1, :])
        
        # title and axis
        plt.title(f"t = {frame * dt:.2f}")

        # center camera on average fish position
        avg_x = np.mean(r[0, :])
        avg_y = np.mean(r[1, :])
        plt.xlim(window[0] + avg_x, window[1] + avg_x)
        plt.ylim(window[2] + avg_y, window[3] + avg_y)

        plt.pause(0.001)  # short pause to update plot

        fig.canvas.draw()
        width, height = fig.canvas.get_width_height()
        
        argb = np.frombuffer(fig.canvas.tostring_argb(), dtype=np.uint8)
        argb = argb.reshape((height, width, 4))
        frame_data = argb[:, :, 1:]
        
        writer.append_data(frame_data)
        render_time += perf_counter() - render_start
        
    if video_render:
        render_start = perf_counter()

        # create blank canvas (black background)
        img = np.zeros((frame_height, frame_width, 3), dtype=np.uint8)
        
        draw_unit_grid(img, win_scale, frame_width, frame_height)

        for i in range(N):
            # convert position to screen coordinates
            x, y = world_to_screen(r[0, i], r[1, i], win_scale, frame_width, frame_height)

            # draw velocity arrow
            vx, vy = v[0, i], v[1, i]
            x_end, y_end = world_to_screen(r[0, i] + vx, r[1, i] + vy, win_scale, frame_width, frame_height)
            arrow_thickness = int(frame_scale / 300)
            cv2.arrowedLine(img, (x, y), (x_end, y_end), (50, 50, 250), arrow_thickness,
                            tipLength=0.3, line_type=cv2.LINE_AA)
            
            # draw fish body
            fish_radius = int(frame_scale / 100)
            cv2.circle(img, (x, y), fish_radius, (250, 50, 50), -1, lineType=cv2.LINE_AA)

        # simulation time
        timestamp = f"t = {frame * dt:.2f}s"
        font_scale = int(frame_scale / 1000)
        font_thickness = int(frame_scale / 500)
        cv2.putText(img, timestamp, (10, 30), cv2.FONT_HERSHEY_SIMPLEX, 
                    font_scale, (200, 200, 200), font_thickness, cv2.LINE_AA)

        # write the frame
        writer.write(img)
        render_time += perf_counter() - render_start

if video_live:
    writer.close()
if video_render:
    writer.release()

# ___________________PLOTTING_____________________
# tspan = np.arange(dt, t + dt, dt)


# # plot center of mass over time
# plt.plot([1, 2, 3, 4], [1, 4, 9, 16])
# plt.xlim(tspan[0], tspan[frames - 1])
# plt.xlabel("Time (sec)")
# plt.ylabel("Center of mass")
# plt.show()

# # plot average velocity/linear momentum
# plt.plot([1, 2, 3, 4], [1, 4, 9, 16])
# plt.xlim(tspan[0], tspan[frames - 1])
# plt.xlabel("Time (sec)")
# plt.ylabel("Velocity/linear momentum of C.O.M.")
# plt.show()

# # plot angular momentum (z-axis only)
# plt.plot([1, 2, 3, 4], [1, 4, 9, 16])
# plt.xlim(tspan[0], tspan[frames - 1])
# plt.xlabel("Time (sec)")
# plt.ylabel("Angular Momentum around C.O.M.")
# plt.show()

print(f"Simulated {N} fish and {frames} frames ({perf_counter() - start:.2f}s)")
print(f"Calculation time: {calc_time:.2f}s")
print(f"Render time: {render_time:.2f}s")