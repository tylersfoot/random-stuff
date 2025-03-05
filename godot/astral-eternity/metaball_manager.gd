extends Node2D

@export var metaball_scene: PackedScene
@export var density_map_canvas: Node2D
var metaballs = []

signal request_spawn_metaball(position: Vector2, velocity: Vector2, size: float, is_main: bool, lifespan: float, is_explosion: bool)

func _ready():
	self.request_spawn_metaball.connect(_on_request_spawn_metaball)

func _process(delta):
	for i in range(metaballs.size() - 1, -1, -1):
		var metaball = metaballs[i]
		if (metaball.size < 0.7):
			metaballs.remove_at(i)
			metaball.queue_free()
		if (!metaball.is_main) and (metaball.lifespan < metaball.lifetime):
			metaballs.remove_at(i)
			metaball.queue_free()
		#if (metaball.lifespan < metaball.lifetime):
			#metaballs.remove_at(i)
			#metaball.queue_free()

func spawn_metaball(position: Vector2, velocity: Vector2, size: float, is_main: bool = false, lifespan: float = 10.0, is_explosion: bool = false):
	var metaball = metaball_scene.instantiate()
	metaball.global_position = position
	metaball.velocity = velocity
	metaball.size = size
	metaball.is_main = is_main
	metaball.lifespan = lifespan
	metaball.is_explosion = is_explosion
	add_child(metaball)
	metaballs.append(metaball)

func _on_request_spawn_metaball(position: Vector2, velocity: Vector2, size: float, is_main: bool = false, lifespan: float = 10.0, is_explosion: bool = false):
	spawn_metaball(position, velocity, size, is_main, lifespan, is_explosion)
