#version 450

out gl_PerVertex
{
    vec4 gl_Position;
};

layout(location = 0) in vec4 position; // location must match demo.cpp

void main()
{
    gl_Position = position;
}
