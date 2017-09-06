# util library

## `util/bit_intrinsics.h`

Implements bit manipulation functions using whatever compiler built-ins are available, otherwise falling back to reasonably efficient C++ implementations.

Implements population-count (`popcount`), count-trailing-zeros (`ctz`), and count-leading-zeros (`clz`).

## `util/bitset.h`

Implements a `constexpr` version of [`std::bitset`](http://en.cppreference.com/w/cpp/utility/bitset). Additionally implements `find_first` and `find_last` for fast set/clear bit searching. The interface is similar to [`std::string::find_*_of(char, std::size_t)`](http://en.cppreference.com/w/cpp/string/basic_string/find_first_of).

## `util/constexpr_array.h`

Implements a `constexpr` version of [`std::array`](http://en.cppreference.com/w/cpp/container/array).

## `util/copy_cv_ref.h`

Utility type traits to copy the `const`-`volatile` and reference qualifiers from one type to another. The source qualifiers combine with the destination qualifiers rather than replacing them.

## `util/endian.h`

Get machine [endianness](https://en.wikipedia.org/wiki/Endianness).

## `util/enum.h`

Utility functions and types for `enum`s.

To use, you need to call the `kazan_util_generate_enum_traits` macro at namespace scope after the definition of the `enum`:

    enum class My_enum // class keyword is optional
    {
        Value_1 = 3, // allows non-zero starting point
        Value_2 = 47, // allows non-successive values
        Value_3 = Value_1, // allows duplicate values
    };

    kazan_util_generate_enum_traits(My_enum,
        My_enum::Value_1,
        My_enum::Value_2,
        My_enum::Value_3);

### `util::Enum_traits<My_enum>`
Has the following `static` `constexpr` members:
- `std::size_t value_count`: the number of values in the `enum`
- `Constexpr_array<My_enum, value_count> values`: the values in the `enum`
- `typedef underlying_type`: the [underlying type](http://en.cppreference.com/w/cpp/types/underlying_type) of the `enum`
- `bool is_compact`: `true` if the list of `enum` values are successive integers.
- `struct Value_and_index`: a holder for a `enum` value and the index into `values` for that value.  
Members:
  - `My_enum value`
  - `std::size_t index`
- `Constexpr_array<Value_and_index, value_count> sorted_value_index_map`: a list of `Value_and_index`, sorted into ascending order based on `value`, leaving duplicates in the original order.
- `std::size_t npos = -1`: constant returned from `find_value` when it can't find the value.
- `std::size_t find_value(My_enum value)`: finds the index of the first occurrence of `value` in values, otherwise returns `npos`.  
If `is_compact`, then casts and subtracts.  
Otherwise, selects between a linear and binary search depending on the number of `enum` values.

### `util::Enum_set<My_enum>`

Similar to [`std::set<My_enum>`](http://en.cppreference.com/w/cpp/container/set) except it is implemented using `util::bitset` to save space.

### `util::Enum_map<My_enum, V>`

Similar to [`std::map<My_enum, V>`](http://en.cppreference.com/w/cpp/container/map) except it is implemented using `util::bitset` and an array. Values are stored only when they are entered into the map, similar to `util::optional`.

## `util/filesystem.h`

Partial implementation of [`std::filesystem`](http://en.cppreference.com/w/cpp/filesystem).

Works on Linux, needs implementations for a few functions on Win32.

Contains declarations for the whole `filesystem` library for easy autocompletion, with `deprecated` annotations for the unimplemented functions.

Contains the `util::filesystem::basic_path` template for easy usage of path manipulation for non-native platforms.

Implemented functions and types:
- `class path` -- completely implemented, except for locales, for Linux and Win32.  
Note: Implemented as a `typedef` of `basic_path`
- `hash_value()` -- note that the C++17 standard specifies that `std::hash` is not specialized for `path`.
- `u8path()`
- `enum class file_type`
- `enum class perms`
- `enum class perm_options`
- `enum class copy_options`
- `enum class directory_options`
- `typedef file_time_type` -- implemented using a custom [Clock](http://en.cppreference.com/w/cpp/concept/TrivialClock), implemented on top of [`GetSystemTimeAsFileTime`](https://msdn.microsoft.com/en-us/library/windows/desktop/ms724397%28v=vs.85%29.aspx) or [`clock_gettime(CLOCK_REALTIME, ...)`](http://man7.org/linux/man-pages/man2/clock_gettime.2.html).
- `class file_status`
- `status_known()`
- `exists()`
- `is_block_file()`
- `is_character_file()`
- `is_directory()`
- `is_fifo()`
- `is_regular_file()`
- `is_socket()`
- `is_symlink()`
- `is_other()`
- `struct space_info`
- `class filesystem_error`
- `class directory_entry`
- `class directory_iterator`
- `begin(directory_iterator iter)`
- `end(const directory_iterator &)`

## `util/in_place.h`

`in_place_t` and friends for `optional` and `variant`

## `util/invoke.h`

Type traits for C++17's `invoke`:
- [`invoke_result`](http://en.cppreference.com/w/cpp/types/result_of)
- `invoke_result_t`
- [`is_invocable`](http://en.cppreference.com/w/cpp/types/is_invocable)
- `is_invocable_v`
- `is_invocable_r`
- `is_invocable_r_v`
- `is_nothrow_invocable`
- `is_nothrow_invocable_v`
- `is_nothrow_invocable_r`
- `is_nothrow_invocable_r_v`

## `util/is_referenceable.h`

Type trait for determining if a reference can be made.

## `util/is_swappable.h`

Type traits for [`swap`-ability](http://en.cppreference.com/w/cpp/types/is_swappable)

## `util/optional.h`

Implementation of [`std::optional`](http://en.cppreference.com/w/cpp/utility/optional)

## `util/soft_float.h`
Software floating-point library, used for conversion to/from text because `long double` doesn't have more precision than `double` on all platforms (ex. ARM).

### `util::soft_float::ExtendedFloat`
- Design is based on the IEEE754 specification.
- Has a 16-bit exponent and a 64-bit mantissa.
- Almost the same as the 80-bit floating-point format on x86 (x86 has one less exponent bit, so has a reduced exponent range).
- Completely `constexpr`, except for conversion to/from `double`/`long double` (because `<cmath>` functions are not `constexpr`).
- Supports +/- Infinity, NaN, and denormal numbers.

Ported from C++11 `constexpr`, hence the ugly implementation.

Implemented operations:
- Add, Subtrace, Multiply, and Divide -- all correctly rounded in round-to-nearest mode
- Conversion to/from `std::int64_t` and `std::uint64_t` -- conversion to integers truncates
- Comparison, implements same semantics for comparison of NaN as ECMAScript.
- Conversion to/from `double` and `long double` -- only rounds once
- `floor`, `ceil`, `trunc`, and `round`
- `pow(ExtendedFloat, std::int64_t)` and `pow(ExtendedFloat, std::uint64_t)`
- `ilogb` and `scalbn`
- `log2` -- implemented by repeated squaring, probably correctly rounded (everything I tested was correctly rounded, but I have no proof)
- `log10` and `log` -- implemented in terms of `log2`

### `util::soft_float::UInt128`

Helper class for `ExtendedFloat` implementing 128-bit unsigned integers.  
Division algorithm is from algorithm D, section 4.3.1 in Art of Computer Programming volume 2 by Knuth.

## `util/string_view.h`

Implementation of C++17's [`std::string_view`](http://en.cppreference.com/w/cpp/string/basic_string_view). Not all functionality works because I can't modify `std::string`.

## `util/text.h`

Utility functions for encoding/decoding UTF-8, UTF-16, UTF-32, and `wchar_t` strings (assuming that `wchar_t` is either UTF-16 or UTF-32).

### `string_cast()`
Converts a text string by decoding to UTF-32 then encoding to the destination string. type.

## `util/variant.h`

Implementation of [`std::variant`](http://en.cppreference.com/w/cpp/utility/variant)

## `util/void_t.h`

Implementation of [`std::void_t`](http://en.cppreference.com/w/cpp/types/void_t)
