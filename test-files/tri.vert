#version 450

void main()
{
    gl_Position = vec4(gl_VertexIndex * 0.25, gl_VertexIndex % 2 * 0.5, 0.0, 1.0);
}
