extends CharacterBody3D

@export_group("Speeds")
@export var base_speed : float = 0.1
@export var sprint_speed : float = 0.5
var mouse_captured : bool = false
var move_speed : float = 0.0
@onready var head: Node3D = $Head
@onready var camera: Camera3D = $Camera
@export var window: Node3D
@export var debug_drawer: MeshInstance3D

var server := PacketPeerUDP.new()

func _ready():
	# start UDP server
	var err = server.bind(5000)
	if err == OK:
		print("Listening on port 5000")
	else:
		print("Failed to bind to port 5000: %s" % [err])

func _process(_delta: float) -> void:
	_server()
	if window:
		_update_window_projection()

func _server() -> void:
	while server.get_available_packet_count() > 0:
		var packet := server.get_packet()
		var data := packet.get_string_from_utf8()
		# print("Received packet: %s" % data)
		# data looks like "0.740,0.9,7.2"
		var parts := data.split(",")
		if parts.size() == 3:
			var x = parts[0].to_float()
			var y = parts[1].to_float()
			var z = parts[2].to_float()
			# set position relative to window
			self.global_position = window.to_global(Vector3(x, y, z))


func _update_window_projection() -> void:
	camera.projection = Camera3D.PROJECTION_FRUSTUM

	# lock camera rotation
	camera.global_basis = window.global_basis
	
	# camera's position relative to window, where window = (0,0,0)
	var camera_relative_pos = window.to_local(camera.global_position)

	# use absolute z distance
	var z_dist = max(abs(camera_relative_pos.z), 0.01)

	# scale the physical monitor height (0.195m) by the camera's distance from the window
	var scaled_size = (0.195 / z_dist) * camera.near
	camera.size = scaled_size

	var scaled_offset = Vector2(
		(-camera_relative_pos.x / z_dist) * camera.near,
		(-camera_relative_pos.y / z_dist) * camera.near
	)
	camera.frustum_offset = scaled_offset

func _unhandled_input(_event: InputEvent) -> void:
	if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		capture_mouse()
	if Input.is_key_pressed(KEY_ESCAPE):
		release_mouse()
	if Input.is_key_pressed(KEY_BACKSPACE):
		get_tree().quit()


func _physics_process(_delta: float) -> void:
	_handle_movement()
	move_and_slide()

func _handle_movement() -> void:
	move_speed = sprint_speed if Input.is_action_pressed("sprint") else base_speed

	var input_dir := Input.get_vector("move_left", "move_right", "move_forward", "move_back")
	var vertical_input := Input.get_axis("move_down", "move_up")
	
	var horizontal_motion = transform.basis * Vector3(input_dir.x, 0, input_dir.y)
	var vertical_motion = Vector3(0, vertical_input, 0)
	
	var move_dir := (horizontal_motion + vertical_motion).normalized()
	
	if move_dir:
		velocity = move_dir * move_speed
	else:
		velocity = velocity.move_toward(Vector3.ZERO, move_speed)

func capture_mouse() -> void:
	Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)
	mouse_captured = true

func release_mouse() -> void:
	Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)
	mouse_captured = false
