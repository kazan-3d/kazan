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
#include "wsi.h"
#include <initializer_list>

namespace kazan
{
namespace vulkan_icd
{
Wsi::Wsi_list Wsi::get_all() noexcept
{
    static const std::initializer_list<const Wsi *> wsi_list = {
#ifdef VK_USE_PLATFORM_XCB_KHR
        &Xcb_wsi::get(),
#endif
#ifdef VK_USE_PLATFORM_XLIB_KHR
        &Xlib_wsi::get(),
#endif
#ifdef VK_USE_PLATFORM_WAYLAND_KHR
        &Wayland_wsi::get(),
#endif
#ifdef VK_USE_PLATFORM_MIR_KHR
        &Mir_wsi::get(),
#endif
#ifdef VK_USE_PLATFORM_ANDROID_KHR
        &Android_wsi::get(),
#endif
#ifdef VK_USE_PLATFORM_WIN32_KHR
        &Win32_wsi::get(),
#endif
    };
    return Wsi_list(wsi_list.begin(), wsi_list.size());
}
}
}
