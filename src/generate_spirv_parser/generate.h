/*
 * Copyright 2017 Jacob Lifshay
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 */

#ifndef GENERATE_SPIRV_PARSER_GENERATE_H_
#define GENERATE_SPIRV_PARSER_GENERATE_H_

#include "ast.h"
#include "util/string_view.h"
#include <fstream>
#include <memory>
#include <string>
#include <cassert>
#include <type_traits>
#include <cstdint>
#include <unordered_set>
#include <unordered_map>
#include <vector>
#include <stdexcept>

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
namespace generate
{
struct Generate_error : public std::runtime_error
{
    using runtime_error::runtime_error;
};

class Generator
{
private:
    struct Tester;

public:
    struct Generator_args
    {
        std::string output_directory;
        explicit Generator_args(std::string output_directory) noexcept
            : output_directory(std::move(output_directory))
        {
        }
        Generator_args(Generator_args &&) = default;
        Generator_args &operator=(Generator_args &&) = default;
        Generator_args(const Generator_args &) = delete;
        Generator_args &operator=(const Generator_args &) = delete;
    };

protected:
    class Push_indent;
    struct Generator_state
    {
        Generator_args &generator_args;
        std::size_t indent_level;
        std::string full_output_file_name;
        std::string guard_macro_name;
        std::ofstream os;
        const ast::Top_level &top_level;
        std::unordered_map<std::string, const ast::Operand_kinds::Operand_kind *> operand_kind_map;
        std::unordered_map<const ast::Operand_kinds::Operand_kind *, bool>
            operand_has_any_parameters_map;
        explicit Generator_state(const Generator *generator,
                                 Generator_args &generator_args,
                                 const ast::Top_level &top_level);
        void open_output_file();
        template <typename T, typename = decltype(os << std::declval<T>())>
        Generator_state &operator<<(T &&v)
        {
            os << std::forward<T>(v);
            return *this;
        }
        Generator_state &operator<<(const ast::Capabilities &v)
        {
            write_capabilities_set(*this, v);
            return *this;
        }
        Generator_state &operator<<(const ast::Extensions &v)
        {
            write_extensions_set(*this, v);
            return *this;
        }
        Push_indent pushed_indent(std::ptrdiff_t amount = 1) noexcept;
    };
    class Push_indent final
    {
        Push_indent(const Push_indent &) = delete;
        Push_indent &operator=(const Push_indent &) = delete;

    private:
        Generator_state *state;
        std::ptrdiff_t amount;

    public:
        explicit Push_indent(Generator_state &state, std::ptrdiff_t amount = 1) noexcept
            : state(&state),
              amount(amount)
        {
            state.indent_level += amount;
        }
        Push_indent(Push_indent &&rt) noexcept : state(rt.state), amount(rt.amount)
        {
            rt.state = nullptr;
        }
        void finish() noexcept
        {
            assert(state);
            state->indent_level -= amount;
            state = nullptr;
        }
        ~Push_indent()
        {
            if(state)
                state->indent_level -= amount;
        }
    };
    // translates initial '`' (backtick) characters to indentations
    struct Indent_interpreted_text
    {
        const char *text;
        std::ptrdiff_t indent_offset;
        bool start_indented;
        constexpr explicit Indent_interpreted_text(const char *text,
                                                   std::ptrdiff_t indent_offset,
                                                   bool start_indented) noexcept
            : text(text),
              indent_offset(indent_offset),
              start_indented(start_indented)
        {
        }
        friend Generator_state &operator<<(Generator_state &state, Indent_interpreted_text v)
        {
            write_indent_interpreted_text(state, v.text, v.indent_offset, v.start_indented);
            return state;
        }
    };
    struct Indent_t
    {
        std::ptrdiff_t offset;
        explicit Indent_t() = default;
        constexpr Indent_t operator()(std::ptrdiff_t additional_offset) const noexcept
        {
            return Indent_t{offset + additional_offset};
        }
        constexpr Indent_interpreted_text operator()(const char *text) const noexcept
        {
            return Indent_interpreted_text(text, offset, false);
        }
        constexpr Indent_interpreted_text operator()(bool start_indented, const char *text) const
            noexcept
        {
            return Indent_interpreted_text(text, offset, start_indented);
        }
        friend Generator_state &operator<<(Generator_state &state, Indent_t indent)
        {
            write_indent(state, indent.offset);
            return state;
        }
    };
    static constexpr auto indent = Indent_t{0};
    enum class Integer_literal_base
    {
        dec = 0,
        hex,
        oct
    };
    struct Unsigned_integer_literal
    {
        std::uint64_t value;
        Integer_literal_base base;
        std::size_t minimum_digit_count;
        constexpr Unsigned_integer_literal(std::uint64_t value,
                                           Integer_literal_base base,
                                           std::size_t minimum_digit_count = 1) noexcept
            : value(value),
              base(base),
              minimum_digit_count(minimum_digit_count)
        {
        }
        friend Generator_state &operator<<(Generator_state &state, Unsigned_integer_literal v)
        {
            write_unsigned_integer_literal(state, v.value, v.base, v.minimum_digit_count);
            return state;
        }
    };
    static constexpr Unsigned_integer_literal unsigned_dec_integer_literal(
        std::uint64_t value) noexcept
    {
        return Unsigned_integer_literal(value, Integer_literal_base::dec);
    }
    static constexpr Unsigned_integer_literal unsigned_hex_integer_literal(
        std::uint64_t value, std::size_t minimum_digit_count = 1) noexcept
    {
        return Unsigned_integer_literal(value, Integer_literal_base::hex, minimum_digit_count);
    }
    static constexpr Unsigned_integer_literal unsigned_oct_integer_literal(
        std::uint64_t value, std::size_t minimum_digit_count = 1) noexcept
    {
        return Unsigned_integer_literal(value, Integer_literal_base::oct, minimum_digit_count);
    }
    struct Signed_integer_literal
    {
        std::int64_t value;
        constexpr explicit Signed_integer_literal(std::int64_t value) noexcept : value(value)
        {
        }
        friend Generator_state &operator<<(Generator_state &state, Signed_integer_literal v)
        {
            write_signed_integer_literal(state, v.value);
            return state;
        }
    };
    static constexpr Signed_integer_literal signed_integer_literal(std::int64_t value) noexcept
    {
        return Signed_integer_literal(value);
    }

protected:
    const char *const output_base_file_name;

protected:
    static std::string get_guard_macro_name_from_file_name(std::string file_name);
    static std::string get_enumerant_name(util::string_view enumeration_name,
                                          std::string enumerant_name,
                                          bool input_name_should_have_prefix)
    {
        return get_enumerant_name(enumeration_name.data(),
                                  enumeration_name.size(),
                                  std::move(enumerant_name),
                                  input_name_should_have_prefix);
    }
    static std::string get_enumerant_name(const char *enumeration_name,
                                          std::size_t enumeration_name_size,
                                          std::string enumerant_name,
                                          bool input_name_should_have_prefix);
    static void write_indent_absolute(Generator_state &state, std::size_t amount);
    static void write_indent(Generator_state &state, std::ptrdiff_t offset)
    {
        write_indent_absolute(state, state.indent_level + offset);
    }
    static void write_indent_interpreted_text(Generator_state &state,
                                              const char *text,
                                              std::ptrdiff_t offset,
                                              bool start_indented);
    static void write_automatically_generated_file_warning(Generator_state &state);
    static void write_copyright_comment(Generator_state &state, const ast::Copyright &copyright);
    static void write_file_comments(Generator_state &state, const ast::Copyright &copyright)
    {
        write_automatically_generated_file_warning(state);
        write_copyright_comment(state, copyright);
    }
    static void write_file_guard_start(Generator_state &state);
    static void write_file_guard_end(Generator_state &state);
    static void write_namespace_start(Generator_state &state, const char *namespace_name);
    static void write_namespace_start(Generator_state &state, const std::string &namespace_name);

private:
    static void write_namespace_end(Generator_state &state);

protected:
    static void write_namespace_end(Generator_state &state, const char *namespace_name)
    {
        write_namespace_end(state);
    }
    static void write_namespace_end(Generator_state &state, const std::string &namespace_name)
    {
        write_namespace_end(state);
    }
    static void write_namespaces_start(Generator_state &state,
                                       const char *const *namespace_names,
                                       std::size_t namespace_name_count)
    {
        for(std::size_t i = 0; i < namespace_name_count; i++)
            write_namespace_start(state, namespace_names[i]);
    }
    static void write_namespaces_start(Generator_state &state,
                                       const std::string *namespace_names,
                                       std::size_t namespace_name_count)
    {
        for(std::size_t i = 0; i < namespace_name_count; i++)
            write_namespace_start(state, namespace_names[i]);
    }
    static void write_namespaces_end(Generator_state &state,
                                     const char *const *namespace_names,
                                     std::size_t namespace_name_count)
    {
        for(std::size_t i = 0; i < namespace_name_count; i++)
            write_namespace_end(state, namespace_names[namespace_name_count - i - 1]);
        state << '\n';
    }
    static void write_namespaces_end(Generator_state &state,
                                     const std::string *namespace_names,
                                     std::size_t namespace_name_count)
    {
        for(std::size_t i = 0; i < namespace_name_count; i++)
            write_namespace_end(state, namespace_names[namespace_name_count - i - 1]);
        state << '\n';
    }
    template <typename T, std::size_t N>
    static void write_namespaces_start(Generator_state &state, const T(&namespace_names)[N])
    {
        write_namespaces_start(state, namespace_names, N);
    }
    template <typename T, std::size_t N>
    static void write_namespaces_end(Generator_state &state, const T(&namespace_names)[N])
    {
        write_namespaces_end(state, namespace_names, N);
    }
    static void write_namespaces_start(Generator_state &state,
                                       std::initializer_list<std::string> namespace_names)
    {
        write_namespaces_start(state, namespace_names.begin(), namespace_names.size());
    }
    static void write_namespaces_start(Generator_state &state,
                                       std::initializer_list<const char *> namespace_names)
    {
        write_namespaces_start(state, namespace_names.begin(), namespace_names.size());
    }
    static void write_namespaces_end(Generator_state &state,
                                     std::initializer_list<std::string> namespace_names)
    {
        write_namespaces_end(state, namespace_names.begin(), namespace_names.size());
    }
    static void write_namespaces_end(Generator_state &state,
                                     std::initializer_list<const char *> namespace_names)
    {
        write_namespaces_end(state, namespace_names.begin(), namespace_names.size());
    }
    static void write_unsigned_integer_literal(Generator_state &state,
                                               std::uint64_t value,
                                               Integer_literal_base base,
                                               std::size_t minimum_digit_count);
    static void write_signed_integer_literal(Generator_state &state, std::int64_t value);

private:
    struct Get_extensions_visitor;

protected:
    static std::unordered_set<std::string> get_extensions(const ast::Top_level &top_level);
    static void write_capabilities_set(Generator_state &state,
                                       const ast::Capabilities &capabilities);
    static void write_extensions_set(Generator_state &state, const ast::Extensions &extensions);
    static std::string get_name_from_words(const std::string &words);
    static std::string get_member_name_from_operand(
        const ast::Instructions::Instruction::Operands::Operand &operand);
    static std::string get_member_name_from_parameter(
        const ast::Operand_kinds::Operand_kind::Enumerants::Enumerant::Parameters::Parameter
            &parameter);
    static std::string get_member_name_from_enumerant(
        const ast::Operand_kinds::Operand_kind::Enumerants::Enumerant
            &enumerant);
    static const ast::Operand_kinds::Operand_kind &get_operand_kind_from_string(
        Generator_state &state, const std::string &operand_kind_str)
    {
        auto *retval = state.operand_kind_map[operand_kind_str];
        if(!retval)
            throw Generate_error("operand kind not found: " + operand_kind_str);
        return *retval;
    }
    static bool get_operand_has_any_parameters(Generator_state &state,
                                               const ast::Operand_kinds::Operand_kind &operand_kind)
    {
        return state.operand_has_any_parameters_map[&operand_kind];
    }
    static std::string get_enumerant_parameters_struct_name(util::string_view enumeration_name,
                                                            std::string enumerant_name,
                                                            bool input_name_should_have_prefix)
    {
        auto retval = "_" + get_enumerant_name(
                                enumeration_name, enumerant_name, input_name_should_have_prefix)
                      + "_parameters";
        retval.insert(retval.begin(), enumeration_name.begin(), enumeration_name.end());
        return retval;
    }
    static std::string get_operand_with_parameters_name(
        Generator_state &state, const ast::Operand_kinds::Operand_kind &operand_kind);
    static std::string get_operand_with_parameters_name(Generator_state &state,
                                                        const std::string &operand_kind_str)
    {
        return get_operand_with_parameters_name(
            state, get_operand_kind_from_string(state, operand_kind_str));
    }
    static std::string get_operand_with_parameters_name(
        Generator_state &state, const ast::Instructions::Instruction::Operands::Operand &operand)
    {
        return get_operand_with_parameters_name(state,
                                                get_operand_kind_from_string(state, operand.kind));
    }
    static std::string get_enum_name(std::string operand_kind_str)
    {
        return operand_kind_str;
    }
    static std::string get_enum_name(const ast::Operand_kinds::Operand_kind &operand_kind)
    {
        return get_enum_name(operand_kind.kind);
    }
    static void write_struct_nonstatic_members_and_constructors(Generator_state &state,
                                                                const std::string &struct_name,
                                                                const std::string *member_types,
                                                                const std::string *member_names,
                                                                std::size_t member_count);
    static std::vector<ast::Operand_kinds::Operand_kind::Enumerants::Enumerant>
        get_unique_enumerants(
            std::vector<ast::Operand_kinds::Operand_kind::Enumerants::Enumerant> enumerants);

protected:
    static constexpr const char *vulkan_cpu_namespace_name = "vulkan_cpu";
    static constexpr const char *spirv_namespace_name = "spirv";
    static constexpr const char *spirv_namespace_names[] = {
        vulkan_cpu_namespace_name, spirv_namespace_name,
    };
    static constexpr const char *capability_enum_name = "Capability";
    static constexpr const char *extension_enum_name = "Extension";
    static constexpr const char *op_enum_name = "Op";

public:
    explicit Generator(const char *output_base_file_name) noexcept
        : output_base_file_name(output_base_file_name)
    {
    }
    virtual void run(Generator_args &generator_args, const ast::Top_level &top_level) const = 0;
    void run(Generator_args &&generator_args, const ast::Top_level &top_level) const
    {
        run(generator_args, top_level);
    }

public:
    virtual ~Generator() = default;
};

inline Generator::Push_indent Generator::Generator_state::pushed_indent(
    std::ptrdiff_t amount) noexcept
{
    return Push_indent(*this, amount);
}

struct Spirv_header_generator;
struct Spirv_source_generator;
struct Parser_header_generator;
struct Parser_source_generator;

struct Generators
{
    static std::unique_ptr<Generator> make_spirv_header_generator();
    static std::unique_ptr<Generator> make_spirv_source_generator();
    static std::unique_ptr<Generator> make_parser_header_generator();
    static std::unique_ptr<Generator> make_parser_source_generator();
    static std::vector<std::unique_ptr<Generator>> make_all_generators();
};
}
}
}

#endif /* GENERATE_SPIRV_PARSER_GENERATE_H_ */
