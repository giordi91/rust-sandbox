#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec3 a_normal;

layout(location=0) out vec3 v_color;

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj; 
};
layout(set=1, binding=0)
uniform PerObject{
    mat4 transform; 
    mat4 pad1; 
    mat4 pad2; 
    mat4 pad3; 
};

void main() {
    v_color = a_normal;
    gl_Position = u_view_proj * transform* vec4(a_position, 1.0);
}