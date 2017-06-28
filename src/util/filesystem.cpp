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
#include "filesystem.h"
#include <cstdlib>
#include <iostream>

namespace vulkan_cpu
{
namespace util
{
namespace filesystem
{
namespace detail
{
#if 0
#warning testing util::filesystem::path
struct Path_tester
{
    template <typename Path, bool Show_parts = true>
    static void write_path(const Path &path)
    {
        std::cout << path << ": kind=";
        switch(path.kind)
        {
        case Path_part_kind::file_name:
            std::cout << "file_name";
            break;
        case Path_part_kind::multiple_parts:
            std::cout << "multiple_parts";
            break;
        case Path_part_kind::root_dir:
            std::cout << "root_dir";
            break;
        case Path_part_kind::relative_root_name:
            std::cout << "relative_root_name";
            break;
        case Path_part_kind::absolute_root_name:
            std::cout << "absolute_root_name";
            break;
        case Path_part_kind::path_separator:
            std::cout << "path_separator";
            break;
        }
        if(Show_parts)
        {
            std::cout << " parts=[";
            auto separator = "";
            for(auto &part : path.parts)
            {
                std::cout << separator;
                separator = ", ";
                write_path<Path, false>(part);
            }
            std::cout << "]";
        }
    }
    template <Path_traits_kind Traits_kind>
    static void test_path(const char *traits_kind_name)
    {
#if 0
        typedef basic_path<> Path;
#else
        typedef basic_path<Traits_kind> Path;
#endif
        std::cout << "testing basic_path<" << traits_kind_name << ">" << std::endl;
        for(auto *test_path_string : {
                "",
                ".",
                "..",
                "C:",
                "C:\\",
                "C:/",
                "/",
                "//",
                "//a",
                "//a/",
                "\\",
                "\\\\",
                "\\\\a",
                "\\\\a\\",
                "a/",
                "a/.",
                "a/..",
                "a/...",
                "a/a",
                "a/.a",
                "a/a.",
                "a/a.a",
                "a/.a.",
                "a/.a.a",
                "a/b/c/d/../.././e/../../f",
                "C:../.",
                "/../.././",
            })
        {
            Path p(test_path_string);
            std::cout << "'" << test_path_string << "' -> ";
            write_path(p);
            std::cout << std::endl;
            std::cout << "make_preferred -> " << Path(p).make_preferred() << std::endl;
            std::cout << "remove_filename -> " << Path(p).remove_filename() << std::endl;
            std::cout << "lexically_normal -> " << p.lexically_normal() << std::endl;
            std::cout << "root_name -> " << p.root_name() << std::endl;
            std::cout << "root_directory -> " << p.root_directory() << std::endl;
            std::cout << "root_path -> " << p.root_path() << std::endl;
            std::cout << "relative_path -> " << p.relative_path() << std::endl;
            std::cout << "parent_path -> " << p.parent_path() << std::endl;
            std::cout << "filename -> " << p.filename() << std::endl;
            std::cout << "stem -> " << p.stem() << std::endl;
            std::cout << "extension -> " << p.extension() << std::endl;
            std::cout << "operator/:";
            for(auto *appended_path : {
                    "", "/abc", "C:abc", "//a/abc", "C:/abc", "abc",
                })
            {
                std::cout << " \"" << appended_path << "\"->" << p / appended_path;
            }
            std::cout << std::endl;
            std::cout << "lexically_proximate:";
            for(auto *base_path : {
                    "", "/abc", "C:abc", "//a/abc", "C:/abc", "abc",
                })
            {
                std::cout << " \"" << base_path << "\"->" << p.lexically_proximate(base_path);
            }
            std::cout << std::endl;
        }
    }
    static void test()
    {
        test_path<Path_traits_kind::posix>("posix");
        test_path<Path_traits_kind::windows>("windows");
        std::string().assign("", 0);
    }
    Path_tester()
    {
        test();
        std::exit(1);
    }
};

namespace
{
Path_tester path_tester;
}
#endif
}
}
}
}
