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
