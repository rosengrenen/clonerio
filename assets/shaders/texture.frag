#version 430
layout(location = 0) in vec2 in_tex_coords;

layout(location = 0) out vec4 out_color;

uniform vec4 color;

layout(binding = 0) uniform sampler2D atlas;
uniform int atlas_size;
uniform int atlas_index;
uniform mat2 tex_rot;

void main() {
		int row = atlas_index / atlas_size;
		int col = atlas_index % atlas_size;
		vec2 rot_tex_coords = (tex_rot * (in_tex_coords - vec2(0.5))) + vec2(0.5);
		vec2 tex_coords = rot_tex_coords / atlas_size + col * vec2(1.0 / atlas_size, 0.0) + row * vec2(0.0, 1.0 / atlas_size);

    out_color = color * texture(atlas, tex_coords);
}