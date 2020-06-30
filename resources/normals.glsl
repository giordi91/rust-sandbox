
vec2 OctWrap(vec2 v) {
  vec2 rhs = vec2(
    v.x > 0.0 ? 1.0 : -1.0,
    v.y > 0.0 ? 1.0 : -1.0
    );
  return (vec2(1.0) - abs(v.yx)) * rhs;
}


vec2 EncodeOctNormal(vec3 n) {
  n /= (abs(n.x) + abs(n.y) + abs(n.z));
  n.xy = n.z >= 0.0f ? n.xy : OctWrap(n.xy);
  n.xy = n.xy * 0.5f + 0.5f;
  return n.xy;
}

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