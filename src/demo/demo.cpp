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
#include <fstream>
#include <iostream>
#include <sstream>
#include <vector>
#include <array>
#include <type_traits>
#include <string>
#include "spirv/spirv.h"
#include "spirv/parser.h"
#include "util/optional.h"
#include "util/string_view.h"
#include "spirv_to_llvm/spirv_to_llvm.h"
#include "llvm_wrapper/llvm_wrapper.h"

namespace vulkan_cpu
{
namespace test
{
util::optional<std::vector<spirv::Word>> load_file(const char *filename)
{
    using spirv::Word;
    constexpr int eof = std::char_traits<char>::eof();
    std::ifstream is;
    is.open(filename, std::ios::in | std::ios::binary);
    if(!is)
        return {};
    constexpr std::size_t block_size = 0x1000;
    std::vector<std::array<Word, block_size>> blocks;
    std::array<unsigned char, sizeof(Word)> word_bytes{};
    static_assert(sizeof(Word) == 4, "");
    static_assert(std::is_same<std::uint8_t, unsigned char>::value, "");
    auto read_little_endian = [](const unsigned char *bytes) -> Word
    {
        Word retval = bytes[3];
        retval <<= 8;
        retval |= bytes[2];
        retval <<= 8;
        retval |= bytes[1];
        retval <<= 8;
        retval |= bytes[0];
        return retval;
    };
    auto read_big_endian = [](const unsigned char *bytes) -> Word
    {
        Word retval = bytes[0];
        retval <<= 8;
        retval |= bytes[1];
        retval <<= 8;
        retval |= bytes[2];
        retval <<= 8;
        retval |= bytes[3];
        return retval;
    };
    for(unsigned char &byte : word_bytes)
    {
        auto v = is.get();
        if(v == eof)
            return {};
        byte = v;
    }
    Word (*read_word_fn)(const unsigned char *) = nullptr;
    if(read_little_endian(word_bytes.data()) == spirv::magic_number)
        read_word_fn = read_little_endian;
    else if(read_big_endian(word_bytes.data()) == spirv::magic_number)
        read_word_fn = read_big_endian;
    else
        return {};
    std::size_t word_count = 1;
    blocks.emplace_back();
    blocks[0][0] = read_word_fn(word_bytes.data());
    std::size_t word_in_block_index = 1;
    while(is.peek() != eof)
    {
        for(unsigned char &byte : word_bytes)
        {
            auto v = is.get();
            if(v == eof)
                return {};
            byte = v;
        }
        blocks.back()[word_in_block_index++] = read_word_fn(word_bytes.data());
        word_count++;
        if(word_in_block_index >= block_size)
        {
            word_in_block_index = 0;
            blocks.emplace_back();
        }
    }
    std::vector<Word> retval;
    retval.reserve(word_count);
    word_in_block_index = 0;
    for(std::size_t word_index = 0, block_index = 0; word_index < word_count; word_index++)
    {
        retval.push_back(blocks[block_index][word_in_block_index++]);
        if(word_in_block_index >= block_size)
        {
            word_in_block_index = 0;
            block_index++;
        }
    }
    return std::move(retval);
}

void dump_words(const spirv::Word *words, std::size_t word_count)
{
    constexpr std::size_t max_words_per_line = 4;
    auto old_fill = std::cerr.fill('0');
    auto old_flags = std::cerr.flags(std::ios::uppercase | std::ios::hex | std::ios::right);
    auto old_width = std::cerr.width();
    bool wrote_line_beginning = false;
    bool wrote_line_ending = true;
    std::cerr << "Words:\n";
    std::size_t current_words_per_line = 0;
    std::size_t index;
    auto seperator = "";
    auto write_line_beginning = [&]()
    {
        std::cerr.width(8);
        std::cerr << index << ":";
        seperator = " ";
        wrote_line_beginning = true;
        wrote_line_ending = false;
    };
    std::string chars = "";
    auto write_line_ending = [&]()
    {
        while(current_words_per_line < max_words_per_line)
        {
            std::cerr << seperator;
            seperator = " ";
            std::cerr.width(8);
            std::cerr.fill(' ');
            std::cerr << "";
            std::cerr.fill('0');
            current_words_per_line++;
        }
        std::cerr << seperator << " |" << chars << "|\n";
        seperator = "";
        wrote_line_ending = true;
        wrote_line_beginning = false;
        current_words_per_line = 0;
        chars.clear();
    };
    auto append_char = [&](unsigned ch)
    {
        if(ch >= 0x20U && ch < 0x7FU)
            chars += static_cast<char>(ch);
        else
            chars += '.';
    };
    auto write_word = [&](spirv::Word w)
    {
        std::cerr << seperator;
        seperator = " ";
        std::cerr.width(8);
        std::cerr << w;
        current_words_per_line++;
        append_char(w & 0xFFU);
        append_char((w >> 8) & 0xFFU);
        append_char((w >> 16) & 0xFFU);
        append_char((w >> 24) & 0xFFU);
    };
    for(index = 0; index < word_count; index++)
    {
        if(current_words_per_line >= max_words_per_line)
            write_line_ending();
        if(!wrote_line_beginning)
            write_line_beginning();
        write_word(words[index]);
    }
    if(!wrote_line_ending)
        write_line_ending();
    std::cerr.flush();
    std::cerr.width(old_width);
    std::cerr.fill(old_fill);
    std::cerr.flags(old_flags);
}

void dump_words(const std::vector<spirv::Word> &words)
{
    dump_words(words.data(), words.size());
}

int test_main(int argc, char **argv)
{
    const char *filename = "test-files/test.spv";
    if(argc > 1)
    {
        if(argv[1][0] == '-')
        {
            std::cerr << "usage: demo [<file.spv>]\n";
            return 1;
        }
        filename = argv[1];
    }
    std::cerr << "loading " << filename << std::endl;
    auto file = load_file(filename);
    if(file)
    {
        {
            dump_words(*file);
            std::cerr << std::endl;
            spirv::Dump_callbacks dump_callbacks;
            try
            {
                spirv::parse(dump_callbacks, file->data(), file->size());
            }
            catch(spirv::Parser_error &e)
            {
                std::cerr << dump_callbacks.ss.str() << std::endl;
                std::cerr << "error: " << e.what();
                return 1;
            }
            std::cerr << dump_callbacks.ss.str() << std::endl;
        }
        auto llvm_target_machine = llvm_wrapper::Target_machine::create_native_target_machine();
        auto llvm_context = llvm_wrapper::Context::create();
        std::uint64_t next_module_id = 1;
        spirv_to_llvm::Converted_module converted_module;
        try
        {
            converted_module = spirv_to_llvm::spirv_to_llvm(llvm_context.get(),
                                                            llvm_target_machine.get(),
                                                            file->data(),
                                                            file->size(),
                                                            next_module_id++);
        }
        catch(spirv::Parser_error &e)
        {
            std::cerr << "error: " << e.what();
            return 1;
        }
        std::cerr << "Translation to LLVM succeeded." << std::endl;
        ::LLVMDumpModule(converted_module.module.get());
        bool failed =
            ::LLVMVerifyModule(converted_module.module.get(), ::LLVMPrintMessageAction, nullptr);
        if(failed)
            return 1;
        auto orc_jit_stack = llvm_wrapper::Orc_jit_stack::create(std::move(llvm_target_machine));
        orc_jit_stack.add_eagerly_compiled_ir(
            std::move(converted_module.module),
            [](const char *symbol_name, [[gnu::unused]] void *user_data) noexcept->std::uint64_t
            {
                std::cerr << "resolving symbol: " << symbol_name << std::endl;
                void *symbol = nullptr;
                return reinterpret_cast<std::uintptr_t>(symbol);
            },
            nullptr);
        for(auto &entry_point : converted_module.entry_points)
        {
            auto function = reinterpret_cast<void *>(
                orc_jit_stack.get_symbol_address(entry_point.entry_function_name.c_str()));
            std::cerr << "entry point \"" << entry_point.name << "\": &"
                      << entry_point.entry_function_name << " == " << function << std::endl;
        }
    }
    else
    {
        std::cerr << "error: can't load file" << std::endl;
        return 1;
    }
    return 0;
}
}
}

int main(int argc, char **argv)
{
    return vulkan_cpu::test::test_main(argc, argv);
}
