extends Node2D

@export var velocity: Vector2 = Vector2.ZERO
@export var size: float = 50.0
@export var shrink_rate: float = 0.995
@export var friction: float = 0.996
@export var is_main: bool = false # if its a main metaball
@export var emit_interval: float = 0.01 # interval to emit particles, in sec
@export var lifespan: float = 0.6 # lifespan in seconds
var lifetime: float = 0 # how long its been alive, in sec
@export var is_explosion: bool = false # if part of end explosion

var time_since_last_emit: float = 0.0

func _process(delta):
	position += velocity * delta
	lifetime += delta
	velocity *= friction
	if is_main:
		if (size >= 20):
			size *= shrink_rate
		else:
			size -= 0.07
	else:
		if is_explosion:
			size *= 0.992
		else:
			size *= shrink_rate
		
	# explosion at end
	if is_main and (size < 1.8 and size > 0):
		explode()

	
	# if this is a main metaball, emit mini metaballs at intervals
	if is_main:
		time_since_last_emit += delta
		if time_since_last_emit >= emit_interval:
			time_since_last_emit = 0.0
			emit_mini_metaball(randf_range(20.0, 50.0), max(size * 0.4, 3.0), false)
			
func explode():
	# explosion on hit or death
	size = 0
	for i in 25:
		emit_mini_metaball(randf_range(20.0, 80.0), randf_range(2.0, 7.0), true)

func emit_mini_metaball(velocity, size, mini_is_explosion):
	var random_direction = Vector2(randf_range(-1.0, 1.0), randf_range(-1.0, 1.0)).normalized()
	var mini_velocity = random_direction * velocity
	var mini_size = size  # mini metaballs are smaller
	var mini_lifespan = lifespan - lifetime
	# inform the manager to spawn the mini metaball
	get_parent().emit_signal("request_spawn_metaball", global_position, mini_velocity, mini_size, false, mini_lifespan, mini_is_explosion)
	
