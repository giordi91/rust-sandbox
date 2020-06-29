#version 450

layout(location=0) in vec2 i_uv;
layout(location=0) out vec4 f_color;

layout(set = 1, binding = 0) uniform texture2D  in_tex;
layout(set = 1, binding = 1) uniform sampler in_tex_sampler;

void main() {

    vec3 normal = texture(sampler2D(in_tex, in_tex_sampler), vec2(i_uv.x,1-i_uv.y)).rgb;
    f_color = vec4(normal, 1.0);
}
