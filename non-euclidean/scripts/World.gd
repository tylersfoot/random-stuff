extends Node3D

# Preload the player scene
var PlayerScene = preload("res://scenes/Player.tscn")

@onready var portal_camera = $Portal1/SubViewport/Camera3D


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

	# camera colors
	$Portal1/SubViewport/Camera3D.get_node("MeshInstance3D").get_mesh().surface_set_material(0, $PortalFrame1/FrameTop/MeshInstance3D.get_mesh().surface_get_material(0))
	$Portal2/SubViewport/Camera3D.get_node("MeshInstance3D").set_mesh($Portal2/SubViewport/Camera3D.get_node("MeshInstance3D").get_mesh().duplicate())
	$Portal2/SubViewport/Camera3D.get_node("MeshInstance3D").get_mesh().surface_set_material(0, $PortalFrame2/FrameTop/MeshInstance3D.get_mesh().surface_get_material(0))
func _process(_delta):
	# print("camblue position: ", $Portal1/SubViewport/Camera3D.global_transform.origin)
	# print("camorange position: ", $Portal2/SubViewport/Camera3D.global_transform.origin)
	# print("cam basis: ", portal_camera.global_transform.basis)
	pass
