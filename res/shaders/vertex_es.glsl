#version 300 es

layout (location = 0) in vec4 vertex;
layout (location = 1) in uvec2 character;

out vec2 texel;

uniform uint line_cells;
uniform uvec2 character_res;
uniform mat4 projection;

void main(void) {
	vec2 pos = vec2(uvec2(uint(gl_InstanceID) % line_cells, uint(gl_InstanceID) / line_cells));
	gl_Position = projection * vec4(vertex.xy + vec2(pos * vec2(character_res)), 0.0f, 1.0f);
	texel = vec2(character.yx * character_res) + (vertex.zw * vec2(character_res));
}
