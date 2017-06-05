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
#include <iostream>
#include "../json/json.h"
#include "../json/parser.h"
#include "parser.h"
#include "../util/optional.h"

namespace vulkan_cpu
{
namespace generate_spirv_parser
{
int generate_spirv_parser_main(int argc, char **argv)
{
    std::string file_name;
    if(argc >= 2)
        file_name = argv[1];
    if(argc != 2 || (file_name.size() > 1 && file_name[0] == '-'))
    {
        std::cerr << "usage: " << argv[0] << " <input.json>" << std::endl;
        return 1;
    }
    try
    {
        auto source = file_name == "-" ? json::Source::load_stdin() :
                                         json::Source::load_file(std::move(file_name));
        try
        {
            auto ast = parser::parse(json::parse(&source));
        }
        catch(json::Parse_error &e)
        {
            std::cerr << "error: " << e.what() << std::endl;
            return 1;
        }
        catch(parser::Parse_error &e)
        {
            std::cerr << "error: " << e.what() << std::endl;
            return 1;
        }
    }
    catch(std::exception &e)
    {
        std::cerr << "error: " << e.what() << std::endl;
        return 1;
    }
    return 0;
}
}
}

int main(int argc, char **argv)
{
    return vulkan_cpu::generate_spirv_parser::generate_spirv_parser_main(argc, argv);
}
