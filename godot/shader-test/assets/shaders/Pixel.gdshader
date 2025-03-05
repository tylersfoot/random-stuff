shader_type spatial;
render_mode unshaded, fog_disabled;

uniform sampler2D DEPTH_TEXTURE : hint_depth_texture, filter_linear_mipmap;
uniform sampler2D SCREEN_TEXTURE : hint_screen_texture, filter_linear_mipmap;
uniform sampler2D NORMAL_TEXTURE : hint_normal_roughness_texture, filter_nearest;

uniform bool shadows_enabled = true;
uniform bool highlights_enabled = true;
uniform float shadow_strength : hint_range(0.0, 1.0, 0.01) = 0.4;
uniform float highlight_strength : hint_range(0.0, 1.0, 0.01) = 0.1;
uniform vec3 highlight_color : source_color = vec3(1.);
uniform vec3 shadow_color : source_color = vec3(0.0);

varying mat4 model_view_matrix;

// generates a pseudo random value between 0.0 and 1.0
float random(vec2 uv) {
	return fract(sin(dot(uv.xy, vec2(12.9898,78.233))) * 43758.5453123);
}

float getDepth(vec2 screen_uv, sampler2D depth_texture, mat4 inv_projection_matrix) {
//	Credit: https://godotshaders.com/shader/depth-modulated-pixel-outline-in-screen-space/
	float raw_depth = texture(depth_texture, screen_uv)[0];
	vec3 normalized_device_coordinates = vec3(screen_uv * 2.0 - 1.0, raw_depth);
    vec4 view_space = inv_projection_matrix * vec4(normalized_device_coordinates, 1.0);	
	view_space.xyz /= view_space.w;	
	return -view_space.z;
}

vec3 getPos(float depth, mat4 mvm, mat4 ipm, vec2 suv, mat4 wm, mat4 icm){
  vec4 pos = inverse(mvm) * ipm * vec4((suv * 2.0 - 1.0), depth * 2.0 - 1.0, 1.0);
  pos.xyz /= (pos.w+0.0001*(1.-abs(sign(pos.w))));
  return (pos*icm).xyz+wm[3].xyz;
}

float normalIndicator(vec3 normalEdgeBias, vec3 baseNormal, vec3 newNormal, float depth_diff){
	// Credit: https://threejs.org/examples/webgl_postprocessing_pixel.html
	float normalDiff = dot(baseNormal - newNormal, normalEdgeBias);
	float normalIndicator = clamp(smoothstep(-.01, .01, normalDiff), 0.0, 1.0);
	float depthIndicator = clamp(sign(depth_diff * .25 + .0025), 0.0, 1.0);
	return (1.0 - dot(baseNormal, newNormal)) * depthIndicator * normalIndicator;
}


void vertex() {
	POSITION = vec4(VERTEX.xy, 1.0, 1.0);
	model_view_matrix = VIEW_MATRIX * mat4(INV_VIEW_MATRIX[0], INV_VIEW_MATRIX[1], INV_VIEW_MATRIX[2], MODEL_MATRIX[3]);

}

void fragment() {
	vec2 e = vec2(1./VIEWPORT_SIZE.xy);
	
//	Shadows
	float depth_diff = 0.0;
	float neg_depth_diff = .5;
	if (shadows_enabled) {
		float depth = getDepth(SCREEN_UV, DEPTH_TEXTURE, INV_PROJECTION_MATRIX);
		float du = getDepth(SCREEN_UV+vec2(0., -1.)*e, DEPTH_TEXTURE, INV_PROJECTION_MATRIX);
		float dr = getDepth(SCREEN_UV+vec2(1., 0.)*e, DEPTH_TEXTURE, INV_PROJECTION_MATRIX);
		float dd = getDepth(SCREEN_UV+vec2(0., 1.)*e, DEPTH_TEXTURE, INV_PROJECTION_MATRIX);
		float dl = getDepth(SCREEN_UV+vec2(-1., 0.)*e, DEPTH_TEXTURE, INV_PROJECTION_MATRIX);
		depth_diff += clamp(du - depth, 0., 1.);
		depth_diff += clamp(dd - depth, 0., 1.);
		depth_diff += clamp(dr - depth, 0., 1.);
		depth_diff += clamp(dl - depth, 0., 1.);
		neg_depth_diff += depth - du;
		neg_depth_diff += depth - dd;
		neg_depth_diff += depth - dr;
		neg_depth_diff += depth - dl;
		neg_depth_diff = clamp(neg_depth_diff, 0., 1.);
		neg_depth_diff = clamp(smoothstep(0.5, 0.5, neg_depth_diff)*10., 0., 1.);
		depth_diff = smoothstep(0.2, 0.3, depth_diff);
		//ALBEDO = vec3(neg_depth_diff);
	}
	
//	Highlights
	float normal_diff = 0.;
	if (highlights_enabled) {
		vec3 normal = texture(NORMAL_TEXTURE, SCREEN_UV).rgb * 2.0 - 1.0;
		vec3 nu = texture(NORMAL_TEXTURE, SCREEN_UV+vec2(0., -1.)*e).rgb * 2.0 - 1.0;
		vec3 nr = texture(NORMAL_TEXTURE, SCREEN_UV+vec2(1., 0.)*e).rgb * 2.0 - 1.0;
		vec3 nd = texture(NORMAL_TEXTURE, SCREEN_UV+vec2(0., 1.)*e).rgb * 2.0 - 1.0;
		vec3 nl = texture(NORMAL_TEXTURE, SCREEN_UV+vec2(-1., 0.)*e).rgb * 2.0 - 1.0;
		vec3 normal_edge_bias = (vec3(1., 1., 1.));
		normal_diff += normalIndicator(normal_edge_bias, normal, nu, depth_diff);
		normal_diff += normalIndicator(normal_edge_bias, normal, nr, depth_diff);
		normal_diff += normalIndicator(normal_edge_bias, normal, nd, depth_diff);
		normal_diff += normalIndicator(normal_edge_bias, normal, nl, depth_diff);
		normal_diff = smoothstep(0.2, 0.8, normal_diff);
		normal_diff = clamp(normal_diff-neg_depth_diff, 0., 1.);
//		ALBEDO = vec3(normal_diff);
	}
	

	vec3 original_color = texture(SCREEN_TEXTURE, SCREEN_UV).rgb;
	vec3 final_highlight_color = mix(original_color, highlight_color, highlight_strength);
	vec3 final_shadow_color = mix(original_color, shadow_color, shadow_strength);
	vec3 final = original_color;
	if (highlights_enabled) {
		final = mix(final, final_highlight_color, normal_diff);
	}
	if (shadows_enabled) {
		final = mix(final, final_shadow_color, depth_diff);
	}
	ALBEDO = final;

	float alpha_mask = depth_diff * float(shadows_enabled) + normal_diff * float(highlights_enabled);
	ALPHA = clamp((alpha_mask) * 5., 0., 1.);
}

//void fragment() {
	//float pixel_size = 16.0;
	//
	//vec4 screen = texture(SCREEN_TEXTURE, SCREEN_UV);
	//float depth_raw = texture(DEPTH_TEXTURE, SCREEN_UV).x;
	//float depth_scaled = clamp(depth_raw*(200.0), 0.0, 1.0);
	////ALBEDO.rgb = vec3(depth_scaled);
	//
	//vec2 grid_uv = round(UV * float(pixel_size)) / float(pixel_size);
	//vec4 text = texture(SCREEN_TEXTURE, grid_uv);
	//ALBEDO.rgb = text.rgb;
	//
	////vec2 new_uv = floor(SCREEN_UV * pixel_size) / pixel_size;
	////ALBEDO.rgb = vec3(random(new_uv*TIME), random(new_uv*TIME*2.0), random(new_uv*TIME*3.0));
//
	////pixel_size = pow(2, floor(log((depth_scaled)*64.0) / log(2)));
	//
	////float x = float(int(FRAGCOORD.x) % int(pixel_size));
	////float y = float(int(FRAGCOORD.y) % int(pixel_size));
	////x = FRAGCOORD.x + floor(float(pixel_size) / 2.0) - x;
	////y = FRAGCOORD.y + floor(float(pixel_size) / 2.0) - y;
	////vec4 pixelated = texture(screen_texture, vec2(x, y) / (VIEWPORT_SIZE));
	////if (pixelated.a==0.0 && screen.a==0.0) {
		////discard;
	////}
	////ALBEDO = pixelated.rgb;
	//
//
	//vec3 ndc = vec3(SCREEN_UV * 2.0 - 1.0, depth_raw);
	//vec4 position_view = INV_PROJECTION_MATRIX * vec4(ndc, 1.0);
	//position_view.xyz /= position_view.w;
	//float linear_depth = -position_view.z;
	//vec4 world = INV_VIEW_MATRIX * INV_PROJECTION_MATRIX * vec4(ndc, 1.0);
	//vec3 position_world = world.xyz / world.w;
	//vec4 normal_rough = texture(normal_rough_texture, SCREEN_UV);
	//vec3 normals_view_raw = normal_rough.xyz; // Normals in view space, in [0.0, 1.0] range
	//vec3 normals_view_remapped = normals_view_raw.xyz * 2.0 - 1.0;  // Normals in view space, in [-1.0, 1.0] range
	//vec3 normals_world = (INV_VIEW_MATRIX * vec4(normals_view_remapped, 0.0)).xyz;
	//float roughness = normal_rough.w;
	//
	////float depth_scaled = 1.0 - (depth_raw / (depth_raw + 0.1));
//
	//
	////ALBEDO.rgb = screen.rgb;
	//// Visualize the outputs
	//// Screen texture
	////ALBEDO.rgb = screen.rgb;
	//// Raw depth
	////ALBEDO.rgb = vec3(depth_raw);
	//// Linear depth
	////ALBEDO.rgb = vec3(fract(linear_depth));
	//// World position
	////ALBEDO.rgb = fract(position_world);
	//// Normals from the normal buffer, in view space
	////ALBEDO.rgb = normals_view_raw;
	//// Normals in world space, [-1.0,1.0] range
	////ALBEDO.rgb = normals_world;
	//// Roughness
	////ALBEDO.rgb = vec3(roughness);
//}