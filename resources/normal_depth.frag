#version 450

layout(location=0) in vec2 i_uv;
layout(location=0) out vec4 f_color;

layout(set = 1, binding = 0) uniform texture2D  in_tex;
layout(set = 1, binding = 1) uniform sampler in_tex_sampler;

vec3 DecodeOctNormal(vec2 f) {
  f = f * 2.0f - 1.0f;

  // https://twitter.com/Stubbesaurus/status/937994790553227264
  vec3 n = vec3(f.x, f.y, 1.0f - abs(f.x) - abs(f.y));
  float t = clamp(-n.z,0.0,1.0);
  //n.xy += n.xy >= 0.0f ? -t : t;
  n.x += n.x >= 0.0f ? -t : t;
  n.y += n.y >= 0.0f ? -t : t;
  return normalize(n);
}
void main() {

    vec2 normal = texture(sampler2D(in_tex, in_tex_sampler), vec2(i_uv.x,1-i_uv.y)).rg;
    //reconstruct z
    f_color = vec4(DecodeOctNormal(normal),1.0);
}
