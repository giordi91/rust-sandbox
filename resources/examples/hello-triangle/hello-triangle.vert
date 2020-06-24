#version 450

const vec2 positions[3] = vec2[3](
    vec2(0.0, 0.5),
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5)
);

layout(set=0, binding=0)
uniform Uniforms {
    mat4 u_view_proj; 
};

void main() {
    gl_Position = u_view_proj * vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
