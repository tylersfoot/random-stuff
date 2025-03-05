extends ColorRect

@export var player: Node2D
var animation_time = 0.0
var shadermaterial
@export var end_texture: Texture2D = preload("res://assets/end_portal_256.png")
const shaderColors = [
	Vector3(0.022087, 0.098399, 0.110818), Vector3(0.011892, 0.095924, 0.089485),
	Vector3(0.027636, 0.101689, 0.100326), Vector3(0.046564, 0.109883, 0.114838),
	Vector3(0.064901, 0.117696, 0.097189), Vector3(0.063761, 0.086895, 0.123646),
	Vector3(0.084817, 0.111994, 0.166380), Vector3(0.097489, 0.154120, 0.091064),
	Vector3(0.106152, 0.131144, 0.195191), Vector3(0.097721, 0.110188, 0.187229),
	Vector3(0.133516, 0.138278, 0.148582), Vector3(0.070006, 0.243332, 0.235792),
	Vector3(0.196766, 0.142899, 0.214696), Vector3(0.047281, 0.315338, 0.321970),
	Vector3(0.204675, 0.390010, 0.302066), Vector3(0.080955, 0.314821, 0.661491)
]
func _ready():
	await player.ready
	shadermaterial = self.material as ShaderMaterial
	shadermaterial.set_shader_parameter("end_texture", end_texture)
	shadermaterial.set_shader_parameter("scale", 2)
	shadermaterial.set_shader_parameter("canvas_size", player.rendersize)
	shadermaterial.set_shader_parameter("screen_size", player.rendersize)
	shadermaterial.set_shader_parameter("colors", shaderColors)

func _process(delta):
	animation_time += delta
	shadermaterial.set_shader_parameter("animation_time", animation_time / 1000)
	shadermaterial.set_shader_parameter("parallax", Vector2(1.0, 1.0))
