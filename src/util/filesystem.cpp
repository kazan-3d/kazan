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
#ifdef __linux__
#ifndef _LARGEFILE_SOURCE
#define _LARGEFILE_SOURCE
#endif
#ifndef _LARGEFILE64_SOURCE
#define _LARGEFILE64_SOURCE
#endif
#endif
#include "filesystem.h"
#include <cstdlib>
#include <iostream>
#ifdef _WIN32
#define NOMINMAX
#include <windows.h>
#elif defined(__linux__)
#include <cerrno>
#include <time.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <dirent.h>
#else
#error filesystem is not implemented for your operating system
#endif

namespace vulkan_cpu
{
namespace util
{
namespace filesystem
{
namespace detail
{
constexpr bool Filesystem_clock::is_steady;

#ifdef __linux__
namespace
{
Filesystem_clock::time_point timespec_to_time_point(const timespec &ts) noexcept
{
    return Filesystem_clock::time_point(
        Filesystem_clock::duration(static_cast<std::int64_t>(ts.tv_sec) * 1'000'000'000L
                                   + static_cast<std::int64_t>(ts.tv_nsec)));
}
}
#elif defined(_WIN32)
namespace
{
Filesystem_clock::time_point filetime_to_time_point(const FILETIME &ft) noexcept
{
    ULARGE_INTEGER li{};
    li.u.LowPart = ft.dwLowDateTime;
    li.u.HighPart = ft.dwHighDateTime;
    return Filesystem_clock::time_point(Filesystem_clock::duration(li.QuadPart));
}
}
#endif

Filesystem_clock::time_point Filesystem_clock::now() noexcept
{
#ifdef _WIN32
    FILETIME ft{};
    ::GetSystemTimeAsFileTime(&ft);
    return filetime_to_time_point(ft);
#elif defined(__linux__)
    timespec ts{};
    ::clock_gettime(CLOCK_REALTIME, &ts);
    return timespec_to_time_point(ts);
#else
#error Filesystem_clock::now is not implemented for your operating system
#endif
}

struct Stat_results
{
    file_type type;
#ifdef _WIN32
#error Stat_results is not implemented on windows
#elif defined(__linux__)
    struct ::stat64 stat_results;
    constexpr Stat_results() noexcept : type(file_type::none), stat_results{}
    {
    }
    Stat_results(const path &p, bool follow_symlink, std::error_code &ec)
        : type(file_type::none), stat_results{}
    {
        ec.clear();
        int old_errno = errno;
        int stat_retval =
            follow_symlink ? stat64(p.c_str(), &stat_results) : lstat64(p.c_str(), &stat_results);
        int error = errno;
        errno = old_errno;
        if(stat_retval != 0)
        {
            switch(error)
            {
            case ENOENT:
                ec = make_error_code(std::errc::no_such_file_or_directory);
                type = file_type::not_found;
                break;
            default:
                ec = std::error_code(errno, std::generic_category());
                type = file_type::none;
                break;
            }
            return;
        }
        if(S_ISBLK(stat_results.st_mode))
            type = file_type::block;
        else if(S_ISCHR(stat_results.st_mode))
            type = file_type::character;
        else if(S_ISDIR(stat_results.st_mode))
            type = file_type::directory;
        else if(S_ISFIFO(stat_results.st_mode))
            type = file_type::fifo;
        else if(S_ISREG(stat_results.st_mode))
            type = file_type::regular;
        else if(S_ISLNK(stat_results.st_mode))
            type = file_type::symlink;
        else if(S_ISSOCK(stat_results.st_mode))
            type = file_type::symlink;
        else
            type = file_type::unknown;
    }
#else
#error Stat_results is not implemented for your operating system
#endif
};

std::uintmax_t file_size(const path &p, std::error_code *ec)
{
#ifdef _WIN32
#error file_size is not implemented on windows
#elif defined(__linux__)
    if(ec)
        ec->clear();
    std::error_code stat_error;
    Stat_results stat_results(p, true, stat_error);
    if(stat_error)
    {
        set_or_throw_error(ec, "stat failed", p, stat_error);
        return -1;
    }
    return stat_results.stat_results.st_size;
#else
#error file_size is not implemented for your operating system
#endif
}

std::uintmax_t hard_link_count(const path &p, std::error_code *ec)
{
#ifdef _WIN32
#error hard_link_count is not implemented on windows
#elif defined(__linux__)
    if(ec)
        ec->clear();
    std::error_code stat_error;
    Stat_results stat_results(p, true, stat_error);
    if(stat_error)
    {
        set_or_throw_error(ec, "stat failed", p, stat_error);
        return -1;
    }
    return stat_results.stat_results.st_nlink;
#else
#error hard_link_count is not implemented for your operating system
#endif
}

file_time_type last_write_time(const path &p, std::error_code *ec)
{
#ifdef _WIN32
#error hard_link_count is not implemented on windows
#elif defined(__linux__)
    if(ec)
        ec->clear();
    std::error_code stat_error;
    Stat_results stat_results(p, true, stat_error);
    if(stat_error)
    {
        set_or_throw_error(ec, "stat failed", p, stat_error);
        return file_time_type::min();
    }
    return timespec_to_time_point(stat_results.stat_results.st_mtim);
#else
#error hard_link_count is not implemented for your operating system
#endif
}

file_status status(const path &p, bool follow_symlink, std::error_code *ec)
{
#ifdef _WIN32
#error status is not implemented on windows
#elif defined(__linux__)
    if(ec)
        ec->clear();
    std::error_code stat_error;
    Stat_results stat_results(p, follow_symlink, stat_error);
    if(stat_error)
    {
        if(stat_results.type == file_type::none || ec)
            set_or_throw_error(ec, "stat failed", p, stat_error);
        return file_status(stat_results.type);
    }
    return file_status(stat_results.type,
                       static_cast<perms>(static_cast<std::uint32_t>(perms::mask)
                                          & stat_results.stat_results.st_mode));
#else
#error status is not implemented for your operating system
#endif
}
}

void directory_entry::refresh(std::error_code *ec)
{
#ifdef _WIN32
#error status is not implemented on windows
#elif defined(__linux__)
    if(ec)
        ec->clear();
    flags = Flags{};
    std::error_code stat_error;
    detail::Stat_results stat_results(path_value, false, stat_error);
    if(stat_error)
    {
        if(stat_results.type == file_type::none)
        {
            detail::set_or_throw_error(ec, "stat failed", path_value, stat_error);
            return;
        }
        flags.has_symlink_status_full_value = true;
        symlink_status_value = file_status(stat_results.type);
        return;
    }
    flags.has_symlink_status_full_value = true;
    symlink_status_value = file_status(stat_results.type,
                                       static_cast<perms>(static_cast<std::uint32_t>(perms::mask)
                                                          & stat_results.stat_results.st_mode));
    flags.has_file_size_value = true;
    file_size_value = stat_results.stat_results.st_size;
    flags.has_hard_link_count_value = true;
    hard_link_count_value = stat_results.stat_results.st_nlink;
    flags.has_last_write_time_value = true;
    last_write_time_value = detail::timespec_to_time_point(stat_results.stat_results.st_mtim)
                                .time_since_epoch()
                                .count();
#else
#error status is not implemented for your operating system
#endif
}

#ifdef _WIN32
#error directory_iterator is not implemented on windows
#elif defined(__linux__)
struct directory_iterator::Implementation
{
    ::DIR *dir = nullptr;
    const directory_options options;
    Implementation(const Implementation &) = delete;
    Implementation &operator=(const Implementation &) = delete;
    Implementation(directory_entry &current_entry,
                   const path &p,
                   directory_options options_in,
                   std::error_code *ec,
                   bool &failed)
        : options(options_in)
    {
        failed = false;
        if(ec)
            ec->clear();
        auto old_errno = errno;
        dir = ::opendir(p.c_str());
        auto error = errno;
        errno = old_errno;
        if(!dir)
        {
            if(error != EACCES
               || (options & directory_options::skip_permission_denied) == directory_options::none)
                detail::set_or_throw_error(
                    ec, "opendir failed", p, std::error_code(error, std::generic_category()));
            failed = true;
            return;
        }
        try
        {
            current_entry.path_value = p / path(); // add trailing slash
            if(!read(current_entry, ec))
                failed = true;
        }
        catch(...)
        {
            close();
            throw;
        }
    }
    bool read(directory_entry &current_entry, std::error_code *ec)
    {
        if(ec)
            ec->clear();
        ::dirent64 *entry;
        while(true)
        {
            auto old_errno = errno;
            errno = 0;
            // using readdir64 instead of readdir64_r: see
            // https://www.gnu.org/software/libc/manual/html_node/Reading_002fClosing-Directory.html
            entry = ::readdir64(dir);
            auto error = errno;
            errno = old_errno;
            if(!entry)
            {
                if(error != 0)
                    detail::set_or_throw_error(
                        ec, "readdir failed", std::error_code(error, std::generic_category()));
                return false;
            }
            if(entry->d_name == string_view(".") || entry->d_name == string_view(".."))
                continue;
            break;
        }
        current_entry.flags = {};
        current_entry.path_value.replace_filename(entry->d_name);
        current_entry.flags.has_symlink_status_type_value = true;
        switch(entry->d_type)
        {
        case DT_FIFO:
            current_entry.symlink_status_value.type(file_type::fifo);
            break;
        case DT_CHR:
            current_entry.symlink_status_value.type(file_type::character);
            break;
        case DT_DIR:
            current_entry.symlink_status_value.type(file_type::directory);
            break;
        case DT_BLK:
            current_entry.symlink_status_value.type(file_type::block);
            break;
        case DT_LNK:
            current_entry.symlink_status_value.type(file_type::symlink);
            break;
        case DT_REG:
            current_entry.symlink_status_value.type(file_type::regular);
            break;
        case DT_SOCK:
            current_entry.symlink_status_value.type(file_type::socket);
            break;
        case DT_UNKNOWN:
        default:
            current_entry.flags.has_symlink_status_type_value = false;
            break;
        }
        return true;
    }
    void close() noexcept
    {
        if(!dir)
            return;
        auto old_errno = errno;
        ::closedir(dir);
        dir = nullptr;
        // ignore any errors
        errno = old_errno;
    }
    ~Implementation()
    {
        close();
    }
};
#else
#error directory_iterator is not implemented for your operating system
#endif

std::shared_ptr<directory_iterator::Implementation> directory_iterator::create(
    directory_entry &current_entry, const path &p, directory_options options, std::error_code *ec)
{
    try
    {
        bool failed;
        auto retval = std::make_shared<Implementation>(current_entry, p, options, ec, failed);
        if(failed)
            return nullptr;
        return retval;
    }
    catch(std::bad_alloc &)
    {
        if(!ec)
            throw;
        *ec = std::make_error_code(std::errc::not_enough_memory);
        return nullptr;
    }
}

void directory_iterator::increment(std::shared_ptr<Implementation> &implementation,
                                   directory_entry &current_entry,
                                   std::error_code *ec)
{
    try
    {
        if(!implementation->read(current_entry, ec))
            implementation = nullptr;
    }
    catch(...)
    {
        implementation = nullptr;
        throw;
    }
}
}
}
}

#if 0 // change to 1 to test filesystem::path
namespace vulkan_cpu
{
namespace util
{
namespace filesystem
{
namespace detail
{
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
}
}
}
}
#endif
