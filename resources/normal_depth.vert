#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

layout(location=0) out vec2  o_uv;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj; 
    mat4 u_view_proj_inverse; 
};

const vec4 positions[6] = vec4[6](
    vec4(-1.0f, 1.0f, 0.0f, 0.0f), vec4(1.0f, 1.0f, 1.0f, 0.0f),
    vec4(-1.0f, -1.0f, 0.0f, 1.0f), vec4(1.0f, -1.0f, 1.0f, 1.0f),
	vec4(-1.0f, -1.0f, 0.0f, 1.0f),vec4(1.0f, 1.0f, 1.0f, 0.0f)
);

void main() {
    vec4 p = positions[gl_VertexIndex];
    o_uv = p.zw;
    gl_Position = u_view_proj * vec4(p.xy, 0.0, 1.0);
}

/*
#include "../common/vertexDefinitions.hlsl"
#include "../common/structures.hlsl"

ConstantBuffer<CameraBuffer> g_camera: register(b0,space0);

static const float4 arrBasePos[6] = {
    float4(-1.0f, 1.0f, 0.0f, 0.0f), float4(1.0f, 1.0f, 1.0f, 0.0f),
    float4(-1.0f, -1.0f, 0.0f, 1.0f), float4(1.0f, -1.0f, 1.0f, 1.0f),
	float4(-1.0f, -1.0f, 0.0f, 1.0f),float4(1.0f, 1.0f, 1.0f, 0.0f)};

FullScreenVertexOut VS( uint vid : SV_VertexID) {
  FullScreenVertexOut vout;
  float4 p = arrBasePos[vid];

  vout.pos.xy = p.xy;
  //vertex positioned slightly before the end plane, could be perfectly zero if we used a 
  //greater equal depth function
  vout.pos.z = 0.00001f;
  vout.pos.w = 1.0f;
  vout.clipPos = p.xy; 
  vout.uv = p.zw;
  return vout;
}
*/