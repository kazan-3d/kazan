# json library
JSON writing and parsing library.

## `json/json.h`

JSON AST along with number writing functions.

### `json::Write_options`
Options for writing JSON.  
Members:
- `composite_value_elements_on_seperate_lines`: true if object and array values should be split over multiple lines.
- `sort_object_values`: true if the values in an object should be sorted based on the field name when writing. If false, writing takes less time, but the written order can change between compilers and even between different executions of the same program.
- `indent_text`: the string written to indent a line by one unit. Defaults to the empty string. `Write_options::pretty` defaults to setting this to 4 spaces.
- `defaults`: create a `Write_options` that uses the default values.
- `pretty`: create a `Write_options` that uses values optimized for human reading. Sets `indent_text` to 4 spaces by default.

### `json::Write_state`
Current JSON-writing state.

### `json::ast::Value_kind`
`enum` of all the JSON value kinds.

### `json::ast::Null_value`
Type representing the `null` value for JSON.

### `json::ast::Boolean_value`
Type representing a `Boolean` value for JSON.

### `json::ast::String_value`
Type representing a `String` value for JSON.

### `json::ast::Number_value`
Type representing a `Number` value for JSON. Can exactly represent all integers with magnitude less than or equal to 2^53.  
Also contains the number-to-text conversion functions.  
Members:
- `max_base = 36`: the maximum base for conversion to text.
- `min_base = 2`: the minimum base for conversion to text.
- `default_base = 10`: the default base for conversion to text. The JSON specification only supports numbers in base 10.
- `operator std::string()`: convert to a `std::string`. Equivalent to `to_string`.
- `to_string`: convert to a `std::string`. Uses the memory from the passed-in `std::string` if it's big enough.
- `to_buffer`: convert to text in the passed-in memory buffer.
- `append_to_string`: appends the textual form of `this` to the passed-in string. Like `return buffer + to_string()`, but more efficient.
- `append_unsigned_integer_to_string`: similar to `append_to_string` but uses the passed-in `std::uint64_t` instead of `this`.
- `unsigned_integer_to_string`: similar to `to_string` but uses the passed-in `std::uint64_t` instead of `this`.
- `unsigned_integer_to_buffer`: similar to `to_buffer` but uses the passed-in `std::uint64_t` instead of `this`.
- `append_signed_integer_to_string`: similar to `append_to_string` but uses the passed-in `std::int64_t` instead of `this`.
- `signed_integer_to_string`: similar to `to_string` but uses the passed-in `std::int64_t` instead of `this`.
- `signed_integer_to_buffer`: similar to `to_buffer` but uses the passed-in `std::int64_t` instead of `this`.
- `append_double_to_string`: similar to `append_to_string` but uses the passed-in `double` instead of `this`.
- `double_to_string`: similar to `to_string` but uses the passed-in `double` instead of `this`.
- `double_to_buffer`: similar to `to_buffer` but uses the passed-in `double` instead of `this`.

### `json::ast::Value`
Type representing any JSON value along with a `Location`.

### `json::ast::Object`
Type representing a JSON Object.

### `json::ast::Array`
Type representing a JSON Array.

### `json::write`
Writes the passed-in JSON to the passed-in `std::ostream`.

### `json::Difference`
Represents a difference between two JSON ASTs.  
Members:
- `element_selectors`: the indexes and/or fields needed to get from the top-level JSON value to the point where the difference is.
- `find_difference`: finds the first difference between two JSON ASTs. Returns `util::nullopt` if the two JSON ASTs are equal.

## `json/location.h`

### `json::Location`
type representing a source-code location. Designed for speed, so it holds a pointer to the `Source` and the byte-index of the represented location.

## `json/parser.h`

### `json::Parse_error`
Type representing a JSON parse error. Holds a `json::Location`, which holds a pointer to the `Source`, so the `Source` needs to be still valid when this is caught, unless you don't need the `location` field.  
Converts the `Location` to text on construction, so, if the `Source` object is destroyed before this `Location` is, it will still work.

### `json::Parse_options`
Options for parsing JSON.  
Members:
- `allow_infinity_and_nan`: if the parser will parse +/-`Infinity` and `NaN`. The JSON specification doesn't allow `Infinity` or `NaN`.  
Ex: Allow parsing of `Infinity`, `-Infinity`, and `NaN`.
- `allow_explicit_plus_sign_in_mantissa`: if the parser will parse a leading `+` on a number. The JSON specification doesn't allow a leading `+` on a number.  
Ex: Allow parsing `+123.45` in addition to `123.45`.
- `allow_single_quote_strings`: if the parser will parse a string delimited by `'` instead of `"`. The JSON specification requires strings to be delimited by `"`.  
Ex: Allow parsing `'abc'` in addition to `"abc"`
- `allow_number_to_start_with_dot`: if the parser will parse a leading `.` on a number. The JSON specification doesn't allow a leading `.` on a number, instead requiring a `0` before the `.`.  
Ex: Allow parsing `.25` in addition to `0.25`
- `default_options`: create a `Parse_options` that holds the defaults.
- `relaxed_options`: create a `Parse_options` that allows parsing all of the different options in `Parse_options`.

### `json::parse`
Parse the passed-in source into the JSON AST. Throws `json::Parse_error` on parse error.

## `json/source.h`

### `json::Source`
type that holds the in-memory representation of a JSON source file.  
Members:
- `file_name`: the name of the JSON source file.
- `contents`: the contents of the JSON source file. Points to a buffer of size `contents_size`. Implemented as a `std::shared_ptr` so it's possible to point to a memory-mapped version of the source file for additional speed.
- `contents_size`: the size of the memory buffer pointed to by `contents`.
- `load_file`: loads the file indicated by the passed-in file name.
- `load_stdin`: reads `stdin` until end-of-file, returning the source-code that was read.
