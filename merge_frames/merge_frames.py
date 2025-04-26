import sys
import cv2




# gets the fps, width, height, and frame count of a video file
def get_video_properties(video_path):
    video = cv2.VideoCapture(video_path)
    if not video.isOpened():
        raise ValueError(f"Could not open video file: {video_path}")

    fps = video.get(cv2.CAP_PROP_FPS)
    width = int(video.get(cv2.CAP_PROP_FRAME_WIDTH))
    height = int(video.get(cv2.CAP_PROP_FRAME_HEIGHT))
    frame_count = int(video.get(cv2.CAP_PROP_FRAME_COUNT))

    video.release()
    return fps, width, height, frame_count


def main():
    if len(sys.argv) > 1:
        video_path = sys.argv[1]
    else:
        input("Please drag and drop a video file onto this script!")
        sys.exit()
    fps, width, height, frame_count = get_video_properties(video_path)
    print(f"Video Path: {video_path}")
    print(f"FPS: {fps}, Width: {width}, Height: {height}, Frame Count: {frame_count}")
    
    input("Press Enter to continue...")
        

if __name__ == "__main__":
    main()