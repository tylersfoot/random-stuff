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
@onready var reverser = outbound_portal.get_node("Reverser") # object that is facing the back of the portal

var inbound_portal_position_offset_camera: Vector3
var inbound_portal_position_offset_player: Vector3
var portal_position_offset: Vector3
var portal_rotation_offset: Vector3
var y_rotation_offset: Basis


func _ready():
	# just in case
	portal_camera.fov = player_camera.fov

	collider.monitoring = true
	# collider.body_shape_entered.connect(Callable(self, "on_portal_enter"))
	collider.body_entered.connect(Callable(self, "on_portal_enter"))
	collider.body_exited.connect(Callable(self, "on_portal_exit"))


func _process(_delta):
	# get the position and rotation offsets between the two portals
	portal_position_offset = outbound_portal.global_transform.origin - global_transform.origin
	# portal_rotation_offset = (global_transform.basis.inverse() * outbound_portal.global_transform.basis).inverse()
	# portal_rotation_offset = outbound_portal.global_transform.basis.get_euler() - global_transform.basis.get_euler()
	# Calculate the rotation offset between the portals
	portal_rotation_offset = reverser.global_transform.basis.get_euler() - global_transform.basis.get_euler()

	# set the portal viewport size to match the main viewport size
	viewport.size = get_viewport().size

	# position offset between player camera and portal, and player and portal
	inbound_portal_position_offset_camera = to_local(player_camera.global_transform.origin)
	inbound_portal_position_offset_player = to_local(player.global_transform.origin)

	var new_position = player_camera.global_transform.origin

	# take the offset between player and inbound portal, and rotate it based around the rotation difference between the portals
	new_position = rotate_position_around_anchor(new_position, global_transform.origin, portal_rotation_offset)

	# offset the camera position by the portal position offset
	new_position = new_position + portal_position_offset

	# rotate camera in place based on rotation difference between portals
	var new_basis = Basis.from_euler(portal_rotation_offset) * player_camera.global_transform.basis


	portal_camera.global_transform.origin = new_position
	portal_camera.global_transform.basis = new_basis

	check_warp()


# func rotate_position_around_anchor(tposition: Vector3, anchor: Vector3, rotation_basis: Basis) -> Vector3:
# 	# translate the position to the local space of the anchor
# 	var local_position = tposition - anchor

# 	# apply the rotation using the Transform
# 	var rotated_local_position = rotation_basis * local_position

# 	# translate the position back to global space
# 	var rotated_position = rotated_local_position + anchor

# 	return rotated_position

func rotate_position_around_anchor(tposition: Vector3, anchor: Vector3, euler_angles: Vector3) -> Vector3:
	# translate the position to the local space of the anchor
	var local_position = tposition - anchor

	# convert euler angles to a basis and then to a quaternion and apply rotation
	var rotated_local_position = Quaternion(Basis.from_euler(euler_angles)) * local_position

	# translate the position back to global space
	var rotated_position = rotated_local_position + anchor

	return rotated_position


# function to get the global Y rotation of a node in radians
func get_global_rotY(node: Node3D) -> float:
	var v0 = node.global_transform.origin
	var v1 = node.to_global(Vector3.FORWARD)
	return atan2(v0.x - v1.x, v0.z - v1.z)

func check_warp():
	# if the player is touching the portal
	if self in player.portals:
		# if the player is past the portal line
		print(self, "  ", inbound_portal_position_offset_player.z)
		if (inbound_portal_position_offset_player.z > 0):
			# teleport the player - offset the player's position and rotation by the portal's position and rotation offsets, with position accounting for portal rotation difference
			player.global_transform.origin = rotate_position_around_anchor(player.global_transform.origin, global_transform.origin, portal_rotation_offset) + portal_position_offset
			player.global_transform.basis = Basis.from_euler(portal_rotation_offset) * player.global_transform.basis
			player.portals.erase(self)


func on_portal_enter(body):
	# checks if player is touching portal hitbox
	if body == player:
		if self in player.portals:
			print("player is touching portal... before touching it... what?")
		print("player is touching ", self)
		player.portals.append(self)


func on_portal_exit(body):
	# if player is no longer touching portal hitbox, remove it from the list (aka right after teleport)
	if body == player:
		if self in player.portals:
			print("player is no longer touching ", self)
			player.portals.erase(self)


# func on_portal_enter(_body_id: RID, body: Node, _body_shape_index: int, area_shape_index: int):
# 	# checks if player's portal collider entered portal
# 	print("body: ", body)
# 	if body == player:
# 		# check if the specific shape is the portal detection shape
# 		var shape_owner_id = body.shape_find_owner(_body_shape_index)
# 		var shape_owner = body.shape_owner_get_owner(shape_owner_id)
# 		# var shape_owner = collider.shape_owner_get_owner(area_shape_index)
# 		# print("shape_owner_id: ", shape_owner_id)
# 		# print("shape_owner: ", shape_owner)
# 		# print("shape_owner.name: ", shape_owner.name)
# 		if shape_owner.name == "PortalCollisionShape":
# 			print("player entered portal")
# 			player.portal = self
# 			check_warp()
