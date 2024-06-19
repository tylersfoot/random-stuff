class_name Player
extends CharacterBody3D

@export var normal_collider: CollisionShape3D = null
@export var portal_collider: CollisionShape3D = null

@onready var speed = 5
@onready var jump_speed = 5
@onready var mouse_sensitivity = 0.002
@onready var portals: Array = [] # portals that the player is currently touching (should only be one, but just in case it's an array)

# get the gravity from the project settings to be synced with RigidBody nodes
@onready var gravity = ProjectSettings.get_setting("physics/3d/default_gravity")


func _physics_process(delta):
	velocity.y += -gravity * delta
	var input = Input.get_vector("left", "right", "forward", "back")
	var movement_dir = transform.basis * Vector3(input.x, 0, input.y)
	velocity.x = movement_dir.x * speed
	velocity.z = movement_dir.z * speed

	move_and_slide()
	if is_on_floor() and Input.is_action_just_pressed("jump"):
		velocity.y = jump_speed
		
	SimpleGrass.set_player_position(global_position)

func _input(event):
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		rotate_y(-event.relative.x * mouse_sensitivity)
		$Camera3D.rotate_x(-event.relative.y * mouse_sensitivity)
		$Camera3D.rotation.x = clampf($Camera3D.rotation.x, -deg_to_rad(70), deg_to_rad(70))
