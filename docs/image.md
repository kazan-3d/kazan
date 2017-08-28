# image library

## `image/image.h`

### `image::Image_descriptor`
`struct` holding all an Image's state, except for the actual memory block.  
Members:
- `supported_flags`: the `VkImageCreateFlags` that are supported by this implementation.
- `flags`
- `type`
- `format`
- `extent`
- `mip_levels`
- `array_layers`
- `supported_samples`: the samples-per-pixel values that are supported by this implementation.
- `samples`
- `tiling`
- `get_memory_size`: returns the size of the memory block for this image.
- `get_memory_stride`: for linear tiling, returns the size of a row of pixels.
- `get_memory_pixel_size`: for linear tiling, returns the size of a pixel.

### `image::Image`
A Vulkan image.  
Members:
- `descriptor`: the `Image_descriptor` for this image.
- `memory`: the memory block for this image.
- `clear`: clear the image to the specified color. Slow implementation of `vkCmdClearColorImage`.
