#version 300 es

in lowp vec2 texel;
out lowp vec4 color;

uniform sampler2D tex;

void main(void) {
	color = texelFetch(tex, ivec2(texel), 0);
}
