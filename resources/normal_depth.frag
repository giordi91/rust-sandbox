#version 450

layout(location=0) in vec2 i_uv;
layout(location=0) out vec4 f_color;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj; 
    mat4 u_view_proj_inverse; 
};



layout(set = 1, binding = 0) uniform texture2D  in_tex;
layout(set = 1, binding = 1) uniform sampler in_tex_sampler;


vec3 reconstructPosition(in vec2  uv, in float z, in mat4 InvVP)
{
  float x = uv.x * 2.0f - 1.0f;
  float y = (1.0 - uv.y) * 2.0f - 1.0f;
  vec4 position_s = vec4(x, y, z, 1.0f);
  vec4 position_v = InvVP* position_s;
  return position_v.xyz / position_v.w;
}

void main() {

    vec3 normal = texture(sampler2D(in_tex, in_tex_sampler), vec2(i_uv.x,1-i_uv.y)).rgb;
    f_color = vec4(normal, 1.0);
}
