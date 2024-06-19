class_name Portal
extends Node3D

@export var player_node_path: NodePath  # path to the player node
@export var outbound_portal_path: NodePath  # path to the destination portal

@onready var mesh_instance = $MeshInstance3D
@onready var viewport = $SubViewport
@onready var portal_camera = $SubViewport/Camera3D
@onready var player = get_node(player_node_path)
@onready var player_camera: Camera3D = player.get_node("Camera3D") as Camera3D
@onready var outbound_portal = get_node(outbound_portal_path) # the destination portal
@onready var collider = $Area3D

var inbound_portal_position_offset: Vector3
var portal_position_offset: Vector3
var portal_rotation_offset: Basis
var y_rotation_offset: Basis


func _ready():

	portal_position_offset = outbound_portal.global_transform.origin - global_transform.origin
	portal_rotation_offset = (global_transform.basis.inverse() * outbound_portal.global_transform.basis).inverse()

	# just in case
	portal_camera.fov = player_camera.fov

	collider.monitoring = true
	collider.body_entered.connect(Callable(self, "on_portal_enter"))
	collider.body_exited.connect(Callable(self, "on_portal_exit"))


func _process(_delta):
	# set the portal viewport size to match the main viewport size
	viewport.size = get_viewport().size

	# position offset between player camera and portal
	inbound_portal_position_offset = to_local(player_camera.global_transform.origin)

	var new_position = player_camera.global_transform.origin

	# take the offset between player and inbound portal, and rotate it based around the rotation difference between the portals
	new_position = rotate_position_around_anchor(new_position, global_transform.origin, portal_rotation_offset)

	# offset the camera position by the portal position offset
	new_position = new_position + portal_position_offset

	# rotate camera in place based on rotation difference between portals
	var new_basis = portal_rotation_offset * player_camera.global_transform.basis


	portal_camera.global_transform.origin = new_position
	portal_camera.global_transform.basis = new_basis


func rotate_position_around_anchor(tposition: Vector3, anchor: Vector3, rotation_basis: Basis) -> Vector3:
	# translate the position to the local space of the anchor
	var local_position = tposition - anchor

	# apply the rotation using the Transform
	var rotated_local_position = rotation_basis * local_position

	# translate the position back to global space
	var rotated_position = rotated_local_position + anchor

	return rotated_position


# function to get the global Y rotation of a node in radians
func get_global_rotY(node: Node3D) -> float:
	var v0 = node.global_transform.origin
	var v1 = node.to_global(Vector3.FORWARD)
	return atan2(v0.x - v1.x, v0.z - v1.z)


func check_warp(_delta = 0.01):
	if inbound_portal_position_offset.z > 0.01:
		return

	var d = Basis(Vector3.UP, get_global_rotY(outbound_portal)) * Vector3(0, 0.001, 0.01)
	# player.global_transform.origin += portal_position_offset + d
	player.global_transform.origin += portal_position_offset
	player.portal = null
	# player.update_portals(true)


func on_portal_enter(body):
	# if player entered portal, teleport them to the outbound portal
	if body == player:
		print("player entered portal")
		player.portal = self
		check_warp()
		# print("player position before teleport: ", player.global_transform.origin)
		# player.global_transform.origin = player.global_transform.origin + portal_position_offset
		# print("player position after teleport: ", player.global_transform.origin)
		# player.global_transform.basis = portal_rotation_offset * player.global_transform.basis


func on_portal_exit(body):
	if body == player:
		if player.portal == self:
			player.portal = null



# class_name Portal
# extends Node3D

# @export var portalPath: NodePath
# var otherPortal: Node3D = null
# @onready var mesh: MeshInstance3D = $PortalMesh
# @onready var meshMaterial: Material = mesh.mesh.surface_get_material(0)
# @onready var mesh2: MeshInstance3D = $PortalMesh2
# @onready var viewport: SubViewport = $Viewport
# @onready var viewCamera: Camera3D = $Viewport/Camera3D
# @onready var player: Node3D = get_node("/root/Level/Player")
# @onready var playerCamera: Camera3D = player.get_node("Camera3D")
# @onready var visibilityNotifier: VisibilityNotifier = $VisibilityNotifier
# @onready var playerDetector: Node3D = get_node_or_null("PlayerDetector")
# @onready var playerDetector2: Node3D = get_node_or_null("PlayerDetector2")

# func get_global_pos(spat: Node3D) -> Vector3:
# 	return spat.global_transform.origin

# func get_global_rotY(spat: Node3D) -> float:
# 	var v0 = spat.global_transform.origin
# 	var v1 = spat.to_global(Vector3.FORWARD)
# 	var r = atan2(v0.x - v1.x, v0.z - v1.z)
# 	return r

# func _ready():
# 	$Arrow.visible = false
# 	player.portals.append(self)
# 	if otherPortal == null:
# 		otherPortal = get_node(portalPath)
# 		if otherPortal != null:
# 			if otherPortal.otherPortal == null:
# 				otherPortal.otherPortal = self
# 		else:
# 			print(get_path(), ": Couldn't find portal! `", portalPath, "`")

#func update_camera(just_teleported):
	#if otherPortal == null:
		#return
#
	#var p1 = get_global_pos(self)
	#var p2 = get_global_pos(otherPortal)
	#var pp = get_global_pos(playerCamera)
	#
	#viewCamera.translation = pp - p1 + p2
	#viewCamera.rotation_degrees.y = player.rotation_degrees.y
	#viewCamera.rotation_degrees.x = playerCamera.rotation_degrees.x

#func is_visible() -> bool:
	#if playerDetector == null:
		#return false
	#return (playerDetector == null or playerDetector.has_player or (playerDetector2 != null and playerDetector2.has_player)) and visibilityNotifier.is_on_screen()

# func _process(_delta):
# 	viewport.size = get_viewport().size
# 	if otherPortal == null:
# 		return
# 	var p1 = get_global_pos(self)
# 	var p2 = get_global_pos(otherPortal)
# 	var pp = get_global_pos(playerCamera)
# 	visible = is_visible() or otherPortal.is_visible()

#func check_warp(_delta = 0.01):
	#var p1 = get_global_pos(self)
	#var p2 = get_global_pos(otherPortal)
	#var offset = to_local(get_global_pos(player))
	#if offset.z > 0.01:
		#return
#
	#var d = Vector3(0, 0.001, 0.01).rotated(Vector3.UP, get_global_rotY(otherPortal))
	#player.translation += p2 - p1 + d
	#player.portal = null
	#player.update_portals(true)

#func on_portal_area_enter(body):
	#if body.get_parent() == player:
		#player.portal = self
		#check_warp()

#func on_portal_area_exit(body):
	#if body.get_parent() == player:
		#if player.portal == self:
			#player.portal = null
