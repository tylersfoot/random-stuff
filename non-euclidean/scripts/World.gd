extends Node3D

# Preload the player scene
var PlayerScene = preload("res://scenes/Player.tscn")

func _ready():
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	SimpleGrass.set_interactive(true)
	# get_viewport().transparent_bg = true
	# get_window().transparent = true
	# get_window().transparent_bg = true
	# get_tree().get_root().set_transparent_background(true)
	# DisplayServer.window_set_flag(DisplayServer.WINDOW_FLAG_TRANSPARENT, true, 0)

	get_viewport().transparent_bg = false
	get_window().transparent = false
	get_window().transparent_bg = false
	get_tree().get_root().set_transparent_background(false)
	DisplayServer.window_set_flag(DisplayServer.WINDOW_FLAG_TRANSPARENT, false, 0)
	
func _process(_delta):
	pass
