#version 330 core

layout (location = 0) in vec4 vertex;
layout (location = 1) in uvec2 char;

out vec2 texel;

uniform uint line_cells;
uniform uvec2 char_res;
uniform mat4 projection;

void main(void) {
	vec2 pos = uvec2(uint(gl_InstanceID) % line_cells, uint(gl_InstanceID) / line_cells);
	gl_Position = projection * vec4(vertex.xy + vec2(pos * char_res), 0.0f, 1.0f);
	texel = vec2(char.yx * char_res) + (vertex.zw * vec2(char_res));
}
