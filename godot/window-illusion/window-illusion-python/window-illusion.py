import socket
import cv2
import time
import mediapipe as mp
from mediapipe.tasks import python
from mediapipe.tasks.python import vision
from mediapipe.tasks.python.vision import drawing_utils
from mediapipe.tasks.python.vision import drawing_styles
import numpy as np

def main():
    UDP_IP = "127.0.0.1"
    UDP_PORT = 5000
    print(f"Target UDP Address: {UDP_IP}:{UDP_PORT}")
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    
    # set up mediapipe
    base_options = python.BaseOptions(model_asset_path='face_landmarker.task')
    options = vision.FaceLandmarkerOptions(
        base_options=base_options,
        running_mode=vision.RunningMode.VIDEO,
        output_facial_transformation_matrixes=True,
        num_faces=1
    )
    detector = vision.FaceLandmarker.create_from_options(options)
    
    cap = cv2.VideoCapture(0)
    if not cap.isOpened():
        print("Error: Could not open webcam.")
        return
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
    cap.set(cv2.CAP_PROP_FPS, 60)
    
    # smoothing
    alpha = 0.35
    smoothed_x, smoothed_y, smoothed_z = 0.0, 0.0, 0.0
    first_frame = True
    
    while True:
        # grab a frame
        ret, frame = cap.read()
        if not ret:
            print("Error: Could not read frame.")
            break
        
        # format for mediapipe (BGR -> RGB)
        rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        mp_image = mp.Image(image_format=mp.ImageFormat.SRGB, data=rgb_frame)
        
        # detect faces and landmarks
        timestamp_ms = int(time.time() * 1000)
        detection_result = detector.detect_for_video(mp_image, timestamp_ms=timestamp_ms)
        
        if len(detection_result.face_landmarks) > 0:
            # annotated_frame = draw_landmarks_on_image(rgb_frame, detection_result)
            matrix = detection_result.facial_transformation_matrixes[0]
            
            # extract and convert to godot coordinates/meters
            x = -matrix[0][3] / 100
            y = (matrix[1][3] / 100) + 0.1015
            z = -matrix[2][3] / 100
            
            if first_frame:
                smoothed_x, smoothed_y, smoothed_z = x, y, z
                first_frame = False
            else:
                smoothed_x = (alpha * x) + ((1.0 - alpha) * smoothed_x)
                smoothed_y = (alpha * y) + ((1.0 - alpha) * smoothed_y)
                smoothed_z = (alpha * z) + ((1.0 - alpha) * smoothed_z)
                
            msg = f"{smoothed_x:.4f},{smoothed_y:.4f},{smoothed_z:.4f}".encode('utf-8')
            # print(f"Sending: {msg}")
            sock.sendto(msg, (UDP_IP, UDP_PORT))

        # cv2.imshow('Webcam', annotated_frame)
        if cv2.waitKey(1) & 0xFF == ord('q'):
            break
    
    cap.release()
    cv2.destroyAllWindows()

if __name__ == "__main__":
    main()
    
    
def draw_landmarks_on_image(rgb_image, detection_result):
  face_landmarks_list = detection_result.face_landmarks
  annotated_image = np.copy(rgb_image)
  for idx in range(len(face_landmarks_list)):
    face_landmarks = face_landmarks_list[idx]
    drawing_utils.draw_landmarks(
        image=annotated_image,
        landmark_list=face_landmarks,
        connections=vision.FaceLandmarksConnections.FACE_LANDMARKS_TESSELATION,
        landmark_drawing_spec=None,
        connection_drawing_spec=drawing_styles.get_default_face_mesh_tesselation_style())
    drawing_utils.draw_landmarks(
        image=annotated_image,
        landmark_list=face_landmarks,
        connections=vision.FaceLandmarksConnections.FACE_LANDMARKS_CONTOURS,
        landmark_drawing_spec=None,
        connection_drawing_spec=drawing_styles.get_default_face_mesh_contours_style())
    drawing_utils.draw_landmarks(
        image=annotated_image,
        landmark_list=face_landmarks,
        connections=vision.FaceLandmarksConnections.FACE_LANDMARKS_LEFT_IRIS,
          landmark_drawing_spec=None,
          connection_drawing_spec=drawing_styles.get_default_face_mesh_iris_connections_style())
    drawing_utils.draw_landmarks(
        image=annotated_image,
        landmark_list=face_landmarks,
        connections=vision.FaceLandmarksConnections.FACE_LANDMARKS_RIGHT_IRIS,
          landmark_drawing_spec=None,
          connection_drawing_spec=drawing_styles.get_default_face_mesh_iris_connections_style())
  return annotated_image
