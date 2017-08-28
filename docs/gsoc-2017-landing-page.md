# GSOC 2017 Landing Page

The code produced as part of GSOC is available here: [gsoc-2017 tag](https://github.com/programmerjake/vulkan-cpu/tree/gsoc-2017)  
I've probably done more work on the code since GSOC, available here: [master branch](https://github.com/programmerjake/vulkan-cpu/tree/master)

## State of code at end of GSOC

A more detailed to-do list is available: [todo.md](todo.md)  
Documentation is available: [docs](../docs)

Completed:
- Generation of SPIR-V parser from Khronos's JSON grammar descriptions.
- Using LLVM as JIT compiler back-end.
- Support for Linux

In-progress:
- Translation from SPIR-V to LLVM IR
- Generation of Graphics Pipelines
- Image support
- Rasterization
- Support for Win32

Not yet started:
- Vectorization
- Vulkan ICD interface
- Vulkan WSI
- Support for other platforms.

## What I learned
- Vulkan doesn't actually specify using the top-left fill rule. (I couldn't find it in the [rasterization section](https://www.khronos.org/registry/vulkan/specs/1.0-wsi_extensions/html/vkspec.html#primsrast-polygons-basic) of the Vulkan spec.)
- SPIR-V is actually surprisingly complicated to parse for an IR that's supposed to be simple to parse. You have to determine the types of values before you can parse OpSwitch instructions properly as the number of words taken by each case depends on the bit-width of the value being switched on.
- I probably should have used the SPIR-V parser that was already written by Khronos.
