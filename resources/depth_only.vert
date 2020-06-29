#version 450

#extension GL_GOOGLE_include_directive: require

#include "resources/structures.glsl"

layout(location=0) in vec3 a_position;


layout (set=0,binding=0) uniform InputData 
{
	FrameData frameData;
}; 

/*
layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj; 
};
*/

layout(set=1, binding=0)
uniform PerObject{
    mat4 transform; 
    mat4 pad1; 
    mat4 pad2; 
    mat4 pad3; 
};

void main() {
    gl_Position = frameData.u_view_proj * transform* vec4(a_position, 1.0);
}