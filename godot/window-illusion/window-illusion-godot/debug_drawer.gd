extends MeshInstance3D

var lines = []

func _process(_delta):
	mesh.clear_surfaces()
	if lines.size() > 0:
		mesh.surface_begin(Mesh.PRIMITIVE_LINES)
		for line in lines:
			mesh.surface_set_color(line.color)
			mesh.surface_add_vertex(line.start)
			mesh.surface_add_vertex(line.end)
		mesh.surface_end()
	
	# Clear lines every frame so they only persist if updated
	lines.clear()

func draw_line(start: Vector3, end: Vector3, color: Color = Color.RED):
	lines.append({"start": start, "end": end, "color": color})