# generate_spirv_parser executable

Parses the JSON in `khronos-spirv/*.grammar.json` and generates `spirv/spirv.h`, `spirv/spirv.cpp`, `spirv/parser.h`, and `spirv/parser.cpp`.

## `generate_spirv_parser/ast.h`
AST for the SPIR-V grammars.

## `generate_spirv_parser/word_iterator.h`

### `generate_spirv_parser::generate::Word_iterator`
iterator that splits a string into words. Works for CamelCase word splitting as well.

### `generate_spirv_parser::generate::Chained_word_iterator`
iterator made from [chaining](https://docs.python.org/3/library/itertools.html#itertools.chain) several `Word_iterator`s together.

### `generate_spirv_parser::generate::make_chained_word_iterator`
helper function to construct `Chained_word_iterator`s.

## `generate_spirv_parser/generate_spirv_parser.cpp`
main driver code for `generate_spirv_parser`

## `generate_spirv_parser/generate.h`

### `generate_spirv_parser::generate::Generate_error`
type for error from output code generation

### `detail::keywords`
list of C++ keywords and words that may become C++ keywords.

### `detail::Generated_output_stream`
A generated section of output code.
Similar to a `std::ostringstream`.  
Implements a custom indentation-control language. See [docs/generate_spirv_parser_indentation_control.md](generate_spirv_parser_indentation_control.md)  
Members:
- `Name_from_words_holder`: holds the result of a `name_from_words` call. Holds references to the arguments of the `name_from_words` call, so users should call `to_string` if they want to save the value for use outside the current expression.  
Members:
  - `to_string`: converts to a `std::string`.

### `detail::name_from_words_*`
splits all the inputs strings into words using `Chained_word_iterator`, then concatenates all the words together separated by underlines, finally applies the word capitalization modifications indicated by the function name. If the result is a C++ keyword (is found in `detail::keywords`), then append an additional underline.
Returns a `Name_from_words_holder`.

## `generate_spirv_parser/generate.cpp`

### `detail::Generated_output_stream::write_to_file`
interprets the indentation-control commands then writes the resulting indented text to a file. If `do_reindent` is false, then it doesn't interpret the indentation-control commands or indents the text, writing the text to the output file without changes. This is useful for debugging.

### `Output_part`
`enum` for the part of the output. The output files are written in parts, this `enum` serves to determine the order in which the parts are written to the output.

### `Spirv_and_parser_generator::State`
Main implementation of the output file code generator.

#### `Spirv_and_parser_generator::State::Output_base`
base class that handles ordering the output parts and combining them together.  
Members:
- `register_output_part`: register a new output part along with the variable containing the output part or the function that generates the output part.
- `write_whole_output`: generates the output

#### `Spirv_and_parser_generator::State::Output_struct`
class representing the different output parts of a `struct`. Handles generating a default constructor and a initialize-everything constructor.  
Members:
- `add_nonstatic_member`: add a new non-static member.

#### `Spirv_and_parser_generator::State::Output_file_base`
class that contains the file-level output parts

#### `Spirv_and_parser_generator::State::Header_file_base`
class that contains the file-level output parts specific to header files.

#### `Spirv_and_parser_generator::State::Source_file_base`
class that contains the file-level output parts specific to source files.

#### `Spirv_and_parser_generator::State::Spirv_h`
class for generating `spirv/spirv.h`

#### `Spirv_and_parser_generator::State::Spirv_cpp`
class for generating `spirv/spirv.cpp`

#### `Spirv_and_parser_generator::State::Parser_h`
class for generating `spirv/parser.h`

#### `Spirv_and_parser_generator::State::Parser_cpp`
class for generating `spirv/parser.cpp`

## `generate_spirv_parser/instruction_properties.h`
structures for describing SPIR-V instruction properties that are not in the JSON format.

## `generate_spirv_parser/instruction_properties.cpp`
the list of SPIR-V instruction that have properties that are not in the JSON format.

## `generate_spirv_parser/parser.cpp`
the code to convert from the JSON ASTs to `generate_spirv_parser`'s AST. This is where all of the input validation should be.

## `generate_spirv_parser/patch.cpp`
the patches to apply to `generate_spirv_parser`'s AST to correct things like unnamed variables producing duplicate names in the output code.
