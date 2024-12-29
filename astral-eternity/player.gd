extends CharacterBody2D

@export var speed = 400
#@export var projectile_scene: PackedScene
@export var metaball_manager: Node2D
@export var alternate_universe_viewport: SubViewport
@export var density_map_viewport: SubViewport
@export var screen_viewport: SubViewport
@export var metaball_render: Sprite2D

var rendersize = Vector2(640, 360)
var windowsize = Vector2(1280, 720)

func _ready():
	print("Window Size: ", DisplayServer.window_get_size())
	print("Render Size: ", rendersize)
	alternate_universe_viewport.size = rendersize
	density_map_viewport.size = rendersize
	screen_viewport.size = rendersize
	metaball_render.offset = rendersize / 2

func get_input():
	var input_direction = Input.get_vector("left", "right", "up", "down")
	velocity = input_direction * speed
	look_at(get_global_mouse_position())

func _physics_process(delta):
	get_input()
	move_and_slide()

func _input(event):
	if event is InputEventMouseButton and event.pressed:
		spawn_metaball(event.position)
		
func spawn_metaball(target_position: Vector2):
	var metaball_direction = (target_position - global_position).normalized()
	var metaball_size = randf_range(55.0, 70.0)
	var metaball_speed = metaball_size * 6

	metaball_manager.spawn_metaball(global_position, metaball_direction * metaball_speed, metaball_size, true, 5)
