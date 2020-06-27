#version 450

layout(location=0) in vec2 i_uv;
layout(location=0) out vec4 f_color;

layout(set = 1, binding = 0) uniform texture2D  depth_texture;
layout(set = 1, binding = 1) uniform sampler depth_sampler;
void main() {

    f_color = texture(sampler2D(depth_texture, depth_sampler), vec2(i_uv.x,1.0 -i_uv.y));
    //f_color = vec4(i_uv, 0.0, 1.0);
}
