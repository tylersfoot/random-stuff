extends Sprite2D
@export var density_map_viewport: SubViewport
@export var alternate_universe_viewport: SubViewport
@export var player: Node2D

func _ready():
	await player.ready
	
	var density_map_texture = density_map_viewport.get_texture()
	var alternate_universe_texture = alternate_universe_viewport.get_texture()
	
	var material = self.material as ShaderMaterial

	material.set_shader_parameter("metaball_map", density_map_texture)
	material.set_shader_parameter("alternate_universe", alternate_universe_texture)
	material.set_shader_parameter("screen_size", player.rendersize)
	material.set_shader_parameter("outline_color", Vector4(0.455, 0.498, 0.949, 1.0))
	#material.set_shader_parameter("outline_color", Vector4(1.0, 0.0, 0.0, 1.0))
	material.set_shader_parameter("outline_radius", 1.0)
