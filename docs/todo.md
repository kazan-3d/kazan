# TODO list

This is a partial TODO list, mostly because I can't remember the full list of things left to do.

Partially implemented:
- SPIR-V to LLVM IR translation code. Need to implement all required instructions. `spirv_to_llvm/`
- Graphics pipeline

Temporary implementation that needs to be rewritten:
- Rasterization code, need to change to use a tiled rasterizer with binning. `pipeline/pipeline.cpp`
- Shader input/output variable layout. Need to implement layout to a array of 128-bit chunks, and matching variables based on those chunks. `spirv_to_llvm/`
- Image memory handling -- needs to be changed to use `VkDeviceMemory`

Implementation that needs to be improved:
- LLVM optimization pass ordering. `pipeline/pipeline.cpp`

Not implemented:
- control-barrier lowering pass for LLVM.
- whole-function vectorization pass for LLVM.
- depth buffering
- non-builtin shader input variables
- multithreading
- multisampling
- command buffers
- actual Vulkan ICD interface -- can just wrap the internal functions that already implement the guts for part of the interface
- `VkBuffer`
- SPIR-V image variables, samplers, and other SPIR-V opaque types.
- indexed drawing -- should implement a vertex cache of some sort
- shader compile cache
- Vulkan WSI
