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

#ifdef VK_USE_PLATFORM_XCB_KHR
#include <xcb/xcb.h>
#include <sys/types.h>
#include <sys/ipc.h>
#include <sys/shm.h>
#include <xcb/shm.h>
#include <xcb/present.h>
#include <cassert>
#include <cstdlib>
#include <memory>
#include "util/optional.h"

namespace kazan
{
namespace vulkan_icd
{
struct Xcb_wsi::Implementation
{
    template <typename T = void>
    struct Free_functor
    {
        void operator()(T *p) noexcept
        {
            std::free(p);
        }
    };
    typedef std::unique_ptr<xcb_query_extension_reply_t, Free_functor<xcb_query_extension_reply_t>>
        Query_extension_reply;
    typedef std::unique_ptr<xcb_get_geometry_reply_t, Free_functor<xcb_get_geometry_reply_t>>
        Get_geometry_reply;
    typedef std::unique_ptr<xcb_get_window_attributes_reply_t,
                            Free_functor<xcb_get_window_attributes_reply_t>>
        Get_window_attributes_reply;
    typedef std::unique_ptr<xcb_query_tree_reply_t, Free_functor<xcb_query_tree_reply_t>>
        Query_tree_reply;
    typedef std::unique_ptr<xcb_shm_query_version_reply_t,
                            Free_functor<xcb_shm_query_version_reply_t>> Shm_query_version_reply;
    typedef std::unique_ptr<xcb_generic_error_t, Free_functor<xcb_generic_error_t>> Generic_error;
    template <typename Id_type,
              xcb_void_cookie_t (*free_function)(xcb_connection_t *connection, Id_type id)>
    class Server_object
    {
    private:
        Id_type value;
        xcb_connection_t *connection;

    public:
        constexpr Server_object() noexcept : value(), connection()
        {
        }
        constexpr Server_object(std::nullptr_t) noexcept : value(), connection()
        {
        }
        constexpr Server_object(Id_type value, xcb_connection_t *connection) noexcept
            : value(value),
              connection(connection)
        {
            assert(connection);
        }
        void swap(Server_object &other) noexcept
        {
            using std::swap;
            swap(value, other.value);
            swap(connection, other.connection);
        }
        Server_object(Server_object &&rt) noexcept : value(), connection()
        {
            swap(rt);
        }
        Server_object &operator=(Server_object rt) noexcept
        {
            swap(rt);
            return *this;
        }
        ~Server_object() noexcept
        {
            if(connection)
                free_function(connection, value);
        }
        Id_type get() const noexcept
        {
            return value;
        }
    };
    typedef Server_object<xcb_gcontext_t, &xcb_free_gc> Gc;
    typedef Server_object<xcb_pixmap_t, &xcb_free_pixmap> Pixmap;
    typedef Server_object<xcb_shm_seg_t, &xcb_shm_detach> Server_shm_seg;
    class Shared_memory_segment
    {
    private:
        int value;

    public:
        constexpr Shared_memory_segment() noexcept : value(-1)
        {
        }
        constexpr Shared_memory_segment(std::nullptr_t) noexcept : Shared_memory_segment()
        {
        }
        explicit Shared_memory_segment(int value) noexcept : value(value)
        {
        }
        static Shared_memory_segment create(std::size_t size, int flags = IPC_CREAT | 0777)
        {
            Shared_memory_segment retval(shmget(IPC_PRIVATE, size, flags));
            if(!retval)
                throw std::runtime_error("shmget failed");
            return retval;
        }
        void swap(Shared_memory_segment &other) noexcept
        {
            using std::swap;
            swap(value, other.value);
        }
        Shared_memory_segment(Shared_memory_segment &&rt) noexcept : Shared_memory_segment()
        {
            swap(rt);
        }
        Shared_memory_segment &operator=(Shared_memory_segment rt) noexcept
        {
            swap(rt);
            return *this;
        }
        ~Shared_memory_segment() noexcept
        {
            if(*this)
                shmctl(value, IPC_RMID, nullptr);
        }
        explicit operator bool() const noexcept
        {
            return value != -1;
        }
        std::shared_ptr<void> map()
        {
            assert(*this);
            void *memory = shmat(value, nullptr, 0);
            if(memory == reinterpret_cast<void *>(-1))
                throw std::runtime_error("shmat failed");
            return std::shared_ptr<void>(memory,
                                         [](void *memory) noexcept
                                         {
                                             shmdt(memory);
                                         });
        }
        int get() const noexcept
        {
            return value;
        }
    };
    struct Start_setup_results
    {
        enum class Status
        {
            Bad_surface,
            No_support,
            Success,
        };
        Status status;
    };
    static Start_setup_results start_setup(xcb_connection_t *connection, xcb_window_t window)
    {
#warning implement start_setup
        assert(!"implement start_setup");
        return {
            .status = Start_setup_results::Status::No_support,
        };
    }
};

VkIcdSurfaceBase *Xcb_wsi::create_surface(const VkXcbSurfaceCreateInfoKHR &create_info) const
{
    assert(create_info.sType == VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR);
    assert(create_info.flags == 0);
    return reinterpret_cast<VkIcdSurfaceBase *>(new Surface_type{
        .base =
            {
                .platform = VK_ICD_WSI_PLATFORM_XCB,
            },
        .connection = create_info.connection,
        .window = create_info.window,
    });
}

void Xcb_wsi::destroy_surface(VkIcdSurfaceBase *surface) const noexcept
{
    delete reinterpret_cast<Surface_type *>(surface);
}

VkResult Xcb_wsi::get_surface_support(VkIcdSurfaceBase *surface_, VkBool32 &supported) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    switch(Implementation::start_setup(surface.connection, surface.window).status)
    {
    case Implementation::Start_setup_results::Status::Bad_surface:
        return VK_ERROR_SURFACE_LOST_KHR;
    case Implementation::Start_setup_results::Status::No_support:
        supported = false;
        return VK_SUCCESS;
    case Implementation::Start_setup_results::Status::Success:
        supported = true;
        return VK_SUCCESS;
    }
    assert(!"unreachable");
    return {};
}

const Xcb_wsi &Xcb_wsi::get() noexcept
{
    static const Xcb_wsi retval{};
    return retval;
}
}
}
#endif

#ifdef VK_USE_PLATFORM_XLIB_KHR
#ifndef VK_USE_PLATFORM_XCB_KHR
#error can't Xlib WSI interface depends on XCB WSI interface for the implementation
#endif
#include <X11/Xlib-xcb.h>

namespace kazan
{
namespace vulkan_icd
{
struct Xlib_wsi::Implementation : public Xcb_wsi::Implementation
{
};

VkIcdSurfaceBase *Xlib_wsi::create_surface(const VkXlibSurfaceCreateInfoKHR &create_info) const
{
    assert(create_info.sType == VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR);
    assert(create_info.flags == 0);
    return reinterpret_cast<VkIcdSurfaceBase *>(new Surface_type{
        .base =
            {
                .platform = VK_ICD_WSI_PLATFORM_XLIB,
            },
        .dpy = create_info.dpy,
        .window = create_info.window,
    });
}

void Xlib_wsi::destroy_surface(VkIcdSurfaceBase *surface) const noexcept
{
    delete reinterpret_cast<Surface_type *>(surface);
}

VkResult Xlib_wsi::get_surface_support(VkIcdSurfaceBase *surface_, VkBool32 &supported) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    switch(Implementation::start_setup(XGetXCBConnection(surface.dpy),
                                       static_cast<xcb_window_t>(surface.window))
               .status)
    {
    case Implementation::Start_setup_results::Status::Bad_surface:
        return VK_ERROR_SURFACE_LOST_KHR;
    case Implementation::Start_setup_results::Status::No_support:
        supported = false;
        return VK_SUCCESS;
    case Implementation::Start_setup_results::Status::Success:
        supported = true;
        return VK_SUCCESS;
    }
    assert(!"unreachable");
    return {};
}

const Xlib_wsi &Xlib_wsi::get() noexcept
{
    static const Xlib_wsi retval{};
    return retval;
}
}
}
#endif
