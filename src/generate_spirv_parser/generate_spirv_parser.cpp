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
#include "json/json.h"
#include "json/parser.h"
#include "parser.h"
#include "util/optional.h"
#include "generate.h"
#include "patch.h"

namespace kazan
{
namespace generate_spirv_parser
{
int generate_spirv_parser_main(int argc, char **argv)
{
    std::string input_directory;
    std::string output_directory;
    if(argc >= 2)
        input_directory = argv[1];
    if(argc >= 3)
        output_directory = argv[2];
    if(argc != 3 || input_directory.empty() || input_directory[0] == '-' || output_directory.empty()
       || output_directory[0] == '-')
    {
        std::cerr << "usage: " << argv[0] << " <input-directory> <output-directory>" << std::endl;
        return 1;
    }
    try
    {
        std::shared_ptr<std::vector<ast::Json_file>> required_files; // outside of try so
        try
        {
            required_files = parser::read_required_files(std::move(input_directory));
            auto ast = parser::parse(std::move(*required_files));
            for(auto *patch : Ast_patches::get_patches())
                patch->run(ast, &std::cout);
            for(auto &generator : generate::Generators::make_all_generators())
            {
                generator->run(generate::Generator::Generator_args(output_directory), ast);
            }
            return 0;
        }
        catch(parser::Parse_error &e)
        {
            std::cerr << e.what() << std::endl;
            return 1;
        }
        catch(json::Parse_error &e)
        {
            std::cerr << e.what() << std::endl;
            return 1;
        }
    }
    catch(std::runtime_error &e)
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
    return kazan::generate_spirv_parser::generate_spirv_parser_main(argc, argv);
}
