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
#include "system_memory_info.h"

#ifdef __linux__
#include <sys/sysinfo.h>
namespace vulkan_cpu
{
namespace util
{
System_memory_info System_memory_info::get()
{
    struct ::sysinfo info
    {
    };
    ::sysinfo(&info);
    return System_memory_info{
        .total_usable_ram = static_cast<std::uintmax_t>(info.totalram) * info.mem_unit,
    };
}
}
}
#elif defined(_WIN32)
#include <windows.h>

namespace vulkan_cpu
{
namespace util
{
System_memory_info System_memory_info::get()
{
    ::MEMORYSTATUSEX memory_status;
    ::GlobalMemoryStatusEx(&memory_status);
    std::uintmax_t retval = memory_status.ullTotalPageFile;
    if(retval > memory_status.ullTotalPhys)
        retval = ullTotalPhys;
    return System_memory_info{
        .total_usable_ram = retval,
    };
}
}
}
#else
#error System_memory_info::get() is not implemented for platform
#endif
