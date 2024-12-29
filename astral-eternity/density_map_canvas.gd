extends Node2D

@export var metaball_manager: Node2D
@export var radial_gradient_texture: Texture2D = preload("res://RadialGradient.png")

func _draw():
	var metaballs = metaball_manager.metaballs
	for metaball in metaballs:
		# original blank white circle
		#draw_circle(metaball.global_position, metaball.size, Color(1, 1, 1, 0.5))
		
		var radius = metaball.size
		var gradient_rect = Rect2(metaball.global_position - Vector2(radius, radius), Vector2(radius * 2, radius * 2))

		# Draw the radial gradient image for the metaball
		draw_texture_rect(radial_gradient_texture, gradient_rect, false)

func _process(_delta):
	queue_redraw()
