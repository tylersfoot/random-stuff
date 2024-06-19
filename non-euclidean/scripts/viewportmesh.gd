extends MeshInstance3D

@onready var sub_viewport = $"../SubViewport"

func _ready() -> void:
	RenderingServer.frame_post_draw.connect(on_draw.bind())

func on_draw() -> void:
	var tex = sub_viewport.get_texture()
	var mat = StandardMaterial3D.new()
	mat.albedo_texture = tex
	set_surface_override_material(0, mat)
	RenderingServer.frame_post_draw.disconnect(on_draw.bind())
