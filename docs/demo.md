# demo executable

## `demo/demo.cpp`

### `test::load_file`
loads a SPIR-V shader into memory, swapping the endian if needed.

### `test::dump_words`
dumps a SPIR-V shader's binary form to `stderr`. Displays the individual bytes in little-endian format to promote readability of SPIR-V strings.

### `test::load_shader`
loads a SPIR-V shader, creating a `pipeline::Shader_module`.

### `test::make_pipeline_layout`
creates the `pipeline::Pipeline_layout` object.

### `test::parse_unsigned_integer`
utility function to parse an unsigned integer.

### `test::test_main`
the main function. Builds a `pipeline::Graphics_pipeline`, draws to an image, then saves the image to `output.bmp`
