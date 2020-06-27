#version 450

layout(location=0) in vec2 i_uv;
layout(location=0) out vec4 f_color;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj; 
    mat4 u_view_proj_inverse; 
};



layout(set = 1, binding = 0) uniform texture2D  depth_texture;
layout(set = 1, binding = 1) uniform sampler depth_sampler;


vec3 reconstructPosition(in vec2  uv, in float z, in mat4 InvVP)
{
  float x = uv.x * 2.0f - 1.0f;
  float y = (1.0 - uv.y) * 2.0f - 1.0f;
  vec4 position_s = vec4(x, y, z, 1.0f);
  vec4 position_v = InvVP* position_s;
  return position_v.xyz / position_v.w;
}

void main() {

    float depth = texture(sampler2D(depth_texture, depth_sampler), vec2(i_uv.x,i_uv.y)).r;
    vec3 pos = reconstructPosition(i_uv,depth, u_view_proj_inverse); 
    vec3 normal = normalize(cross(dFdx(pos), dFdy(pos)));
    f_color = vec4(normal*0.5 + 0.5, 1.0);
    //f_color = vec4(vec3(depth*10.0f,0,0), 1.0);

}
