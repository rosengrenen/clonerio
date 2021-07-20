#version 430
layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_tex_coords;

layout(location = 0) out vec2 out_tex_coords;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main() {
		out_tex_coords = in_tex_coords;
    gl_Position = projection * view * model * vec4(in_position, 0.0, 1.0);
}
