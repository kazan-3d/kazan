# Documentation for the indentation-control language implemented by `generate_spirv_parser::generate::detail::Generated_output_stream`

Indentation is controlled by several different values:
- `start_indent_depth`: indentation depth that is changed to at the beginning of the next line.
- `start_indent_depth_stack`: stack that is used to save and restore the value of `start_indent_depth`.
- `indent_depth`: the indentation depth of the current line.
- `output_indent_width`: the standard indentation step size, defaults to 4

Each line is indented based on indentation change commands at the beginning of the line. After indenting the current line, but before interpreting the indentation change commands in the next line, `indent_depth` is set to the value of `start_indent_depth`.  
Indentation commands:
- <code style="white-space: pre">"    "</code> (4 spaces): Increase `indent_depth` by `output_indent_width`.
- <code>"\`"</code> (single back-tick): Increase `indent_depth` by 1. Used for manual indentation.
- `"@+"`: increase both `start_indent_depth` and `indent_depth` by `output_indent_width`.
- `"@-"`: decrease both `start_indent_depth` and `indent_depth` by `output_indent_width`.
- `"@_"`: decrease `start_indent_depth` by `output_indent_width`, while retaining `indent_depth` at it's current value.
- `guard_macro`: write the name of the include-guard macro. The macro name is generated from the name of the file.
- `literal("<text>")`: Write `<text>` without interpreting it as indentation-commands.
- `push_start`: push the value of `start_indent_depth` onto `start_indent_depth_stack`.
- `pop_start`: pop a value off of `start_indent_depth_stack` and write it into `start_indent_depth`.
- `restart_indent`: sets `indent_depth` to the value of `start_indent_depth`.
- `add_start_offset(offset)`: changes `start_indent_depth` by `offset`.
