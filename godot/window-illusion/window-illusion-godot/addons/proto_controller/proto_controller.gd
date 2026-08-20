extends CharacterBody3D

## Can we move around?
@export var can_move : bool = true
## Can we hold to run?
@export var can_sprint : bool = true

@export_group("Speeds")
## Look around rotation speed (Mouse).
@export var mouse_look_speed : float = 0.002
## Look around rotation speed (Keyboard).
@export var keyboard_look_speed : float = 2.5
## Normal movement speed.
@export var base_speed : float = 7.0
## Sprinting movement speed.
@export var sprint_speed : float = 15.0

@export_group("Movement Input Actions")
@export var input_left : String = "move_left"
@export var input_right : String = "move_right"
@export var input_forward : String = "move_forward"
@export var input_back : String = "move_back"
@export var input_up : String = "move_up"
@export var input_down : String = "move_down"
@export var input_sprint : String = "sprint"

@export_group("Camera Look Input Actions")
@export var look_left : String = "look_left"
@export var look_right : String = "look_right"
@export var look_up : String = "look_up"
@export var look_down : String = "look_down"

var mouse_captured : bool = false
var look_rotation : Vector2
var move_speed : float = 0.0

## IMPORTANT REFERENCES
@onready var head: Node3D = $Head

func _ready() -> void:
    # Disable the collider permanently since we are basically a floating camera now
    if has_node("Collider"):
        $Collider.disabled = true
        
    look_rotation.y = rotation.y
    look_rotation.x = head.rotation.x

func _unhandled_input(event: InputEvent) -> void:
    # Mouse capturing
    if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
        capture_mouse()
    if Input.is_key_pressed(KEY_ESCAPE):
        release_mouse()
    
    # Mouse Look
    if mouse_captured and event is InputEventMouseMotion:
        # We pass the relative motion directly to our rotation function
        rotate_look(event.relative * mouse_look_speed)

func _physics_process(delta: float) -> void:
    # --- 1. KEYBOARD CAMERA LOOK ---
    var kb_look_input := Input.get_vector(look_left, look_right, look_up, look_down)
    if kb_look_input.length() > 0:
        # Multiply by delta and speed to keep it frame-rate independent
        rotate_look(kb_look_input * keyboard_look_speed * 60.0 * delta * mouse_look_speed)

    # --- 2. MOVEMENT ---
    if can_move:
        # Determine current speed
        move_speed = sprint_speed if (can_sprint and Input.is_action_pressed(input_sprint)) else base_speed

        # Get movement inputs
        var input_dir := Input.get_vector(input_left, input_right, input_forward, input_back)
        var vertical_input := Input.get_axis(input_down, input_up)
        
        # Calculate horizontal movement relative to where the body is facing
        var horizontal_motion = transform.basis * Vector3(input_dir.x, 0, input_dir.y)
        
        # Calculate vertical movement (strictly global up/down so Space always goes up)
        var vertical_motion = Vector3(0, vertical_input, 0)
        
        # Combine and normalize
        var move_dir := (horizontal_motion + vertical_motion).normalized()
        
        if move_dir:
            velocity = move_dir * move_speed
        else:
            velocity = velocity.move_toward(Vector3.ZERO, move_speed) # Smooth stop
    else:
        velocity = Vector3.ZERO
    
    # Use velocity to actually move
    move_and_slide()

## Rotate us to look around.
func rotate_look(rot_input : Vector2):
    look_rotation.x -= rot_input.y
    look_rotation.x = clamp(look_rotation.x, deg_to_rad(-85), deg_to_rad(85))
    look_rotation.y -= rot_input.x
    
    transform.basis = Basis()
    rotate_y(look_rotation.y)
    
    head.transform.basis = Basis()
    head.rotate_x(look_rotation.x)

func capture_mouse():
    Input.set_mouse_mode(Input.MOUSE_MODE_CAPTURED)
    mouse_captured = true

func release_mouse():
    Input.set_mouse_mode(Input.MOUSE_MODE_VISIBLE)
    mouse_captured = false