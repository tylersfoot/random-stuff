class_name Player
extends CharacterBody3D

@onready var speed = 5
@onready var jump_speed = 5
@onready var mouse_sensitivity = 0.002
@onready var portals: Array = [] # portals that the player is currently touching (should only be one, but just in case it's an array)
@onready var camera_target = $CameraTarget
@onready var camera = $Camera3D

# get the gravity from the project settings to be synced with RigidBody nodes
@onready var gravity = ProjectSettings.get_setting("physics/3d/default_gravity")


func _physics_process(delta):
	velocity.y += -gravity * delta

	var current_speed = speed
	if Input.is_action_pressed("crouch"):  # default action for left control key in Godot
		current_speed *= 0.1

	if Input.is_action_pressed("sprint"):  # default action for left control key in Godot
		current_speed *= 2


	var input = Input.get_vector("left", "right", "forward", "back")
	var movement_dir = transform.basis * Vector3(input.x, 0, input.y)
	velocity.x = movement_dir.x * current_speed
	velocity.z = movement_dir.z * current_speed

	move_and_slide()
	if is_on_floor() and Input.is_action_just_pressed("jump"):
		velocity.y = jump_speed
		
	SimpleGrass.set_player_position(global_position)

func _input(event):
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		rotate_y(-event.relative.x * mouse_sensitivity) # rotates player (and camera) horizontally
		# camera_target.rotation.y = rotation.y
		camera.rotate_x(-event.relative.y * mouse_sensitivity) # pitches/rotates camera vertically
		camera.rotation.x = clampf(camera.rotation.x, -deg_to_rad(70), deg_to_rad(70)) # clamps camera's y rotation from doing a barrel roll
		camera_target.rotate_x(-event.relative.y * mouse_sensitivity)
		camera_target.rotation.x = clampf(camera_target.rotation.x, -deg_to_rad(70), deg_to_rad(70))
		# camera.rotation = camera_target.rotation
		# print("--------------------")
		# print("player: ", rotation)
		# print("target: ", camera_target.rotation)
		# print("camera: ", camera.rotation)

func _process(_delta):
	# smoothly lerp the camera position to prevent 60fps jitter
	# camera.global_transform.origin = lerp(camera.global_transform.origin, camera_target.global_transform.origin, 5.0 * delta)
	var t = 1
	t = t+1
