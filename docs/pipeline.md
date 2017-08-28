# pipeline library

## `pipeline/pipeline.h`

### `pipeline::Api_object_deleter`
helper type trait for `pipeline::Api_object_handle`

### `pipeline::Api_object_handle`
pointer-wrapper type that implements conversion to/from Vulkan handles.

### `pipeline::Pipeline_cache`
Implementation for `VkPipelineCache`

### `pipeline::Render_pass`
Implementation for `VkRenderPass`

### `pipeline::Shader_module`
Implementation for `VkShaderModule`  
Members:
- `bytes`: pointer to shader source.
- `byte_count`: number of bytes the shader source takes.
- `words`: get the shader source as a `spirv::Word` pointer.
- `word_count`: get the number of `spirv::Word`s the shader takes.

### `pipeline::Pipeline`
Generic `VkPipeline`.  
Members:
- `optimize_module`: function that optimizes the passed-in LLVM IR module.

### `pipeline::Graphics_pipeline`
`VkPipeline` for graphics pipelines.  
Members:
- `Vertex_shader_function`: type for JIT compiled vertex shaders.
- `Fragment_shader_function`: type for JIT compiled fragment shaders.
- `run_vertex_shader`: run the vertex shader.
- `get_vertex_shader_output_struct_size`: return the size used by each invocation of the vertex shader.
- `dump_vertex_shader_output_struct`: decodes and dumps the values in the vertex shader output struct.
- `run_fragment_shader`: runs the fragment shader.
- `run`: run the pipeline for a single draw command.
- `make`: create a new `Graphics_pipeline`.

## `pipeline/pipeline.cpp`

### `pipeline::Pipeline::optimize_module`
function that runs LLVM optimizations on the passed-in LLVM IR module.

### `pipeline::Graphics_pipeline::Implementation`
implementation for Graphics_pipeline.  
Members:
- `llvm_context`: the LLVM context for the JIT compiled code in this pipeline.
- `jit_symbol_resolver`: the `Jit_symbol_resolver` for the JIT compiled code in this pipeline.
- `jit_stack`: the JIT compiler stack for this pipeline.
- `data_layout`: the LLVM data layout for this pipeline.
- `compiled_shaders`: the list of compiled shaders in this pipeline.
- `vertex_shader_output_struct`: the type of the vertex shader's output struct.
- `append_value_to_string`: decodes the value of the passed-in type from the passed-in memory buffer, returning the string representation of it. Used only for debugging.

### `pipeline::Graphics_pipeline::run`
Function that runs the graphics pipeline for a single draw call. This function's implementation will be replaced when the original rasterizer plan is implemented.  
Member types and lambdas:
- `Vec4`: type for glsl's `vec4` type
- `Ivec4`: type for glsl's `ivec4` type
- `interpolate_float`: interpolate a `float` value, returns `v0` when `t == 0` and `v1` when `t == 1`.
- `interpolate_vec4`: interpolate a `Vec4` value, returns `v0` when `t == 0` and `v1` when `t == 1`.
- `Triangle`: type for a triangle.
- `solve_for_t`: solves the formula `interpolate_float(t, v0, v1) == 0` for `t`.
- `clip_edge`: processes a single edge of a triangle to be clipped.
- `clip_triangles`: clips a list of triangles
- `Edge_equation`: equation for the line at one of the edges of the triangle currently being rasterized.  
Members:
 - `inside`: checks if the passed-in pixel coordinate is inside the triangle, according to `this`.

### `pipeline::Graphics_pipeline::make`
Creates a new `Graphics_pipeline` by compiling all the passed-in shaders. Throws `std::runtime_error` or a child class on error.
