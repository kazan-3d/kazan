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
#include <cstring>
#include <iostream>
#include <list>
#include <utility>
#include <algorithm>
#include "util/optional.h"

namespace kazan
{
namespace vulkan_icd
{
struct Xcb_wsi::Implementation
{
    static std::uint32_t u32_from_bytes(std::uint8_t b0,
                                        std::uint8_t b1,
                                        std::uint8_t b2,
                                        std::uint8_t b3) noexcept
    {
        static_assert(sizeof(std::uint8_t) == 1 && sizeof(std::uint32_t) == 4, "");
        union
        {
            std::uint8_t bytes[4];
            std::uint32_t u32;
        };
        bytes[0] = b0;
        bytes[1] = b1;
        bytes[2] = b2;
        bytes[3] = b3;
        return u32;
    }
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
    static xcb_query_extension_cookie_t query_extension(xcb_connection_t *connection,
                                                        const char *extension_name) noexcept
    {
        return xcb_query_extension(connection, std::strlen(extension_name), extension_name);
    }
    enum class Surface_format_group
    {
        B8G8R8A8,
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
        Gc gc;
        bool shm_is_supported;
        unsigned window_depth;
        std::uint32_t image_width;
        std::uint32_t image_height;
        Surface_format_group surface_format_group;
        util::optional<std::vector<VkPresentModeKHR>> present_modes;
        VkSurfaceCapabilitiesKHR capabilities;
        std::size_t image_pixel_size;
        std::size_t scanline_alignment;
        xcb_shm_query_version_cookie_t shm_query_version_cookie;
        Start_setup_results(Gc gc,
                            bool shm_is_supported,
                            unsigned window_depth,
                            std::uint32_t image_width,
                            std::uint32_t image_height,
                            Surface_format_group surface_format_group,
                            std::vector<VkPresentModeKHR> present_modes,
                            const VkSurfaceCapabilitiesKHR &capabilities,
                            std::size_t image_pixel_size,
                            std::size_t scanline_alignment,
                            xcb_shm_query_version_cookie_t shm_query_version_cookie) noexcept
            : status(Status::Success),
              gc(std::move(gc)),
              shm_is_supported(shm_is_supported),
              window_depth(window_depth),
              image_width(image_width),
              image_height(image_height),
              surface_format_group(surface_format_group),
              present_modes(std::move(present_modes)),
              capabilities(capabilities),
              image_pixel_size(image_pixel_size),
              scanline_alignment(scanline_alignment),
              shm_query_version_cookie(shm_query_version_cookie)
        {
        }
        constexpr Start_setup_results(Status status) noexcept : status(status),
                                                                gc(),
                                                                shm_is_supported(),
                                                                window_depth(),
                                                                image_width(),
                                                                image_height(),
                                                                surface_format_group(),
                                                                present_modes(),
                                                                capabilities{},
                                                                image_pixel_size(),
                                                                scanline_alignment(),
                                                                shm_query_version_cookie()
        {
            assert(status != Status::Success);
        }
    };
    static Start_setup_results start_setup(xcb_connection_t *connection,
                                           xcb_window_t window,
                                           bool is_full_setup)
    {
        auto mit_shm_cookie = query_extension(connection, "MIT-SHM");
        auto get_geometry_cookie = xcb_get_geometry(connection, window);
        auto get_window_attributes_cookie = xcb_get_window_attributes(connection, window);
        auto query_tree_cookie = xcb_query_tree(connection, window);
        auto gc_id = xcb_generate_id(connection);
        const std::uint32_t gc_params[1] = {
            0, // value for XCB_GC_GRAPHICS_EXPOSURES
        };
        xcb_create_gc(connection, gc_id, window, XCB_GC_GRAPHICS_EXPOSURES, gc_params);
        auto gc = Gc(gc_id, connection);
        auto mit_shm_reply =
            Query_extension_reply(xcb_query_extension_reply(connection, mit_shm_cookie, nullptr));
        bool shm_is_supported = mit_shm_reply && mit_shm_reply->present;
        xcb_shm_query_version_cookie_t shm_query_version_cookie{};
        if(shm_is_supported && is_full_setup)
            shm_query_version_cookie = xcb_shm_query_version(connection);
        auto get_geometry_reply =
            Get_geometry_reply(xcb_get_geometry_reply(connection, get_geometry_cookie, nullptr));
        if(!get_geometry_reply)
            return Start_setup_results::Status::Bad_surface;
        std::uint32_t image_width = get_geometry_reply->width;
        std::uint32_t image_height = get_geometry_reply->height;
        auto get_window_attributes_reply = Get_window_attributes_reply(
            xcb_get_window_attributes_reply(connection, get_window_attributes_cookie, nullptr));
        if(!get_window_attributes_reply)
            return Start_setup_results::Status::Bad_surface;
        auto window_visual_id = get_window_attributes_reply->visual;
        auto query_tree_reply =
            Query_tree_reply(xcb_query_tree_reply(connection, query_tree_cookie, nullptr));
        if(!query_tree_reply)
            return Start_setup_results::Status::Bad_surface;
        auto root_window = query_tree_reply->root;
        xcb_screen_t *screen = nullptr;
        for(auto iter = xcb_setup_roots_iterator(xcb_get_setup(connection)); iter.rem;
            xcb_screen_next(&iter))
        {
            if(iter.data->root == root_window)
            {
                screen = iter.data;
                break;
            }
        }
        if(!screen)
            return Start_setup_results::Status::Bad_surface;
        xcb_visualtype_t *window_visual_type = nullptr;
        unsigned window_depth = 0;
        for(auto depth_iter = xcb_screen_allowed_depths_iterator(screen); depth_iter.rem;
            xcb_depth_next(&depth_iter))
        {
            for(auto visual_iter = xcb_depth_visuals_iterator(depth_iter.data); visual_iter.rem;
                xcb_visualtype_next(&visual_iter))
            {
                if(visual_iter.data->visual_id == window_visual_id)
                {
                    window_visual_type = visual_iter.data;
                    window_depth = depth_iter.data->depth;
                    break;
                }
            }
            if(window_visual_type)
                break;
        }
        if(!window_visual_type)
            return Start_setup_results::Status::Bad_surface;
        std::uint32_t red_mask = window_visual_type->red_mask;
        std::uint32_t green_mask = window_visual_type->green_mask;
        std::uint32_t blue_mask = window_visual_type->blue_mask;
        std::uint32_t alpha_mask;
        switch(window_depth)
        {
        case 24:
            alpha_mask = 0;
            break;
        case 32:
            alpha_mask = ~(red_mask | green_mask | blue_mask);
            break;
        default:
            return Start_setup_results::Status::No_support;
        }
        xcb_format_t *window_pixmap_format = nullptr;
        for(auto iter = xcb_setup_pixmap_formats_iterator(xcb_get_setup(connection)); iter.rem;
            xcb_format_next(&iter))
        {
            if(iter.data->depth == window_depth)
            {
                window_pixmap_format = iter.data;
                break;
            }
        }
        if(!window_pixmap_format)
            return Start_setup_results::Status::Bad_surface;
        std::size_t image_pixel_size;
        switch(window_pixmap_format->bits_per_pixel)
        {
        case 24:
            image_pixel_size = 3;
            break;
        case 32:
            image_pixel_size = 4;
            break;
        default:
            return Start_setup_results::Status::No_support;
        }
        Surface_format_group surface_format_group;
        if(red_mask == u32_from_bytes(0, 0, 0xFF, 0) && green_mask == u32_from_bytes(0, 0xFF, 0, 0)
           && blue_mask == u32_from_bytes(0xFF, 0, 0, 0)
           && (alpha_mask == 0 || alpha_mask == u32_from_bytes(0, 0, 0, 0xFF))
           && image_pixel_size == 4)
            surface_format_group = Surface_format_group::B8G8R8A8;
        else
            return Start_setup_results::Status::No_support;
        std::size_t scanline_alignment = 1;
        switch(window_pixmap_format->scanline_pad)
        {
        case 8:
            scanline_alignment = 1;
            break;
        case 16:
            scanline_alignment = 2;
            break;
        case 32:
            scanline_alignment = 4;
            break;
        default:
            assert(!"invalid pixmap format scanline-pad");
        }
        std::vector<VkPresentModeKHR> present_modes = {
#warning properly implement fifo present mode using X11 Present extension
            VK_PRESENT_MODE_FIFO_KHR, VK_PRESENT_MODE_IMMEDIATE_KHR,
        };
        VkSurfaceCapabilitiesKHR capabilities = {
            .minImageCount = 1,
            .maxImageCount = 0,
            .currentExtent =
                {
                    .width = image_width, .height = image_height,
                },
            .minImageExtent =
                {
                    .width = image_width, .height = image_height,
                },
            .maxImageExtent =
                {
                    .width = image_width, .height = image_height,
                },
            .maxImageArrayLayers = 1,
            .supportedTransforms = VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
            .currentTransform = VK_SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
            .supportedCompositeAlpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            .supportedUsageFlags = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT
                                   | VK_IMAGE_USAGE_INPUT_ATTACHMENT_BIT
                                   | VK_IMAGE_USAGE_SAMPLED_BIT
                                   | VK_IMAGE_USAGE_STORAGE_BIT
                                   | VK_IMAGE_USAGE_TRANSFER_DST_BIT
                                   | VK_IMAGE_USAGE_TRANSFER_SRC_BIT,
        };
        return Start_setup_results(std::move(gc),
                                   shm_is_supported,
                                   window_depth,
                                   image_width,
                                   image_height,
                                   surface_format_group,
                                   present_modes,
                                   capabilities,
                                   image_pixel_size,
                                   scanline_alignment,
                                   shm_query_version_cookie);
    }
    struct Swapchain final : public Vulkan_swapchain
    {
        struct Swapchain_image final : public vulkan::Vulkan_image
        {
            std::shared_ptr<void> pixels;
            std::size_t row_pitch;
            std::uint32_t width;
            std::uint32_t height;
            std::size_t pixel_size;
            Shared_memory_segment shared_memory_segment;
            Server_shm_seg server_shm_seg;
            Pixmap pixmap;
            std::list<Swapchain_image *>::iterator iter;
            Swapchain_image(std::shared_ptr<void> pixels,
                            std::size_t row_pitch,
                            std::uint32_t width,
                            std::uint32_t height,
                            std::size_t pixel_size,
                            Shared_memory_segment shared_memory_segment,
                            Server_shm_seg server_shm_seg,
                            Pixmap pixmap,
                            std::list<Swapchain_image *>::iterator iter) noexcept
                : pixels(std::move(pixels)),
                  row_pitch(row_pitch),
                  width(width),
                  height(height),
                  pixel_size(pixel_size),
                  shared_memory_segment(std::move(shared_memory_segment)),
                  server_shm_seg(std::move(server_shm_seg)),
                  pixmap(std::move(pixmap)),
                  iter(iter)
            {
            }
        };
        xcb_connection_t *connection;
        xcb_window_t window;
        bool shm_is_supported;
        std::list<Swapchain_image *> free_list;
        explicit Swapchain(Start_setup_results start_setup_results,
                           xcb_connection_t *connection,
                           xcb_window_t window,
                           const VkSwapchainCreateInfoKHR &create_info)
            : Vulkan_swapchain({}),
              connection(connection),
              window(window),
              shm_is_supported(start_setup_results.shm_is_supported)
        {
            std::size_t unpadded_scanline_size =
                start_setup_results.image_pixel_size * start_setup_results.image_width;
            std::size_t padded_scanline_size =
                (unpadded_scanline_size + start_setup_results.scanline_alignment - 1U)
                & ~(start_setup_results.scanline_alignment - 1U);
            std::size_t image_size = padded_scanline_size * start_setup_results.image_height;
            if(shm_is_supported)
            {
                auto shm_query_version_reply = Shm_query_version_reply(xcb_shm_query_version_reply(
                    connection, start_setup_results.shm_query_version_cookie, nullptr));
                if(!shm_query_version_reply || !shm_query_version_reply->shared_pixmaps
                   || shm_query_version_reply->pixmap_format != XCB_IMAGE_FORMAT_Z_PIXMAP)
                {
                    std::cerr
                        << "shared memory pixmaps are not supported, falling back to using core "
                           "X protocol"
                        << std::endl;
                    shm_is_supported = false;
                }
            }
            auto image_count = std::max<std::uint32_t>(create_info.minImageCount, 2);
            while(true)
            {
                bool shm_failed = false;
                for(std::uint32_t i = 0; i < image_count; i++)
                {
                    Shared_memory_segment shared_memory_segment;
                    std::shared_ptr<void> pixels;
                    Server_shm_seg server_shm_seg;
                    Pixmap pixmap;
                    if(shm_is_supported)
                    {
                        shared_memory_segment = Shared_memory_segment::create(image_size);
                        pixels = shared_memory_segment.map();
                        auto seg_id = xcb_generate_id(connection);
                        auto shm_attach_cookie = xcb_shm_attach_checked(
                            connection, seg_id, shared_memory_segment.get(), false);
                        auto error =
                            Generic_error(xcb_request_check(connection, shm_attach_cookie));
                        if(error)
                        {
                            shm_failed = true;
                            break;
                        }
                        server_shm_seg = Server_shm_seg(seg_id, connection);
                        auto pixmap_id = xcb_generate_id(connection);
                        error = Generic_error(xcb_request_check(
                            connection,
                            xcb_shm_create_pixmap_checked(connection,
                                                          pixmap_id,
                                                          window,
                                                          start_setup_results.image_width,
                                                          start_setup_results.image_height,
                                                          start_setup_results.window_depth,
                                                          server_shm_seg.get(),
                                                          0)));
                        if(error)
                        {
                            shm_failed = true;
                            break;
                        }
                        pixmap = Pixmap(pixmap_id, connection);
                    }
                    else
                    {
                        pixels = std::shared_ptr<unsigned char>(new unsigned char[image_size],
                                                                [](unsigned char *p) noexcept
                                                                {
                                                                    delete[] p;
                                                                });
                    }
                    auto image_iter = free_list.insert(free_list.end(), nullptr);
                    auto image =
                        std::make_unique<Swapchain_image>(std::move(pixels),
                                                          padded_scanline_size,
                                                          start_setup_results.image_width,
                                                          start_setup_results.image_height,
                                                          start_setup_results.image_pixel_size,
                                                          std::move(shared_memory_segment),
                                                          std::move(server_shm_seg),
                                                          std::move(pixmap),
                                                          image_iter);
                    *image_iter = image.get();
                    images.push_back(std::move(image));
                }
                if(shm_failed)
                {
                    std::cerr << "using shared memory failed, falling back to using core X protocol"
                              << std::endl;
                    shm_is_supported = false;
                    free_list.clear();
                    images.clear();
                    continue;
                }
                break;
            }
        }
    };
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

VkResult Xcb_wsi::get_surface_support(VkIcdSurfaceBase *surface_, bool &supported) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    switch(Implementation::start_setup(surface.connection, surface.window, false).status)
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

VkResult Xcb_wsi::get_surface_formats(VkIcdSurfaceBase *surface_,
                                      std::vector<VkSurfaceFormatKHR> &surface_formats) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    auto start_setup_result =
        Implementation::start_setup(surface.connection, surface.window, false);
    switch(start_setup_result.status)
    {
    case Implementation::Start_setup_results::Status::Bad_surface:
    case Implementation::Start_setup_results::Status::No_support:
        return VK_ERROR_SURFACE_LOST_KHR;
    case Implementation::Start_setup_results::Status::Success:
    {
        surface_formats.clear();
        switch(start_setup_result.surface_format_group)
        {
        case Implementation::Surface_format_group::B8G8R8A8:
            surface_formats = {
                {
                    .format = VK_FORMAT_B8G8R8A8_SRGB,
                    .colorSpace = VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                },
                {
                    .format = VK_FORMAT_B8G8R8A8_UNORM,
                    .colorSpace = VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
                },
            };
            break;
        }
        return VK_SUCCESS;
    }
    }
    assert(!"unreachable");
    return {};
}

VkResult Xcb_wsi::get_present_modes(VkIcdSurfaceBase *surface_,
                                    std::vector<VkPresentModeKHR> &present_modes) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    auto start_setup_result =
        Implementation::start_setup(surface.connection, surface.window, false);
    switch(start_setup_result.status)
    {
    case Implementation::Start_setup_results::Status::Bad_surface:
    case Implementation::Start_setup_results::Status::No_support:
        return VK_ERROR_SURFACE_LOST_KHR;
    case Implementation::Start_setup_results::Status::Success:
        present_modes = std::move(start_setup_result.present_modes.value());
        return VK_SUCCESS;
    }
    assert(!"unreachable");
    return {};
}

VkResult Xcb_wsi::get_surface_capabilities(VkIcdSurfaceBase *surface_,
                                           VkSurfaceCapabilitiesKHR &capabilities) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    auto start_setup_result =
        Implementation::start_setup(surface.connection, surface.window, false);
    switch(start_setup_result.status)
    {
    case Implementation::Start_setup_results::Status::Bad_surface:
    case Implementation::Start_setup_results::Status::No_support:
        return VK_ERROR_SURFACE_LOST_KHR;
    case Implementation::Start_setup_results::Status::Success:
        capabilities = start_setup_result.capabilities;
        return VK_SUCCESS;
    }
    assert(!"unreachable");
    return {};
}

util::variant<VkResult, std::unique_ptr<Vulkan_swapchain>> Xcb_wsi::create_swapchain(
    vulkan::Vulkan_device &device, const VkSwapchainCreateInfoKHR &create_info) const
{
#warning finish implementing Xcb_wsi::create_swapchain
    auto &surface = *reinterpret_cast<Surface_type *>(create_info.surface);
    return std::make_unique<Implementation::Swapchain>(
        Implementation::start_setup(surface.connection, surface.window, true),
        surface.connection,
        surface.window,
        create_info);
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
    static VkIcdSurfaceXcb get_xcb_surface(const VkIcdSurfaceXlib &surface) noexcept
    {
        return VkIcdSurfaceXcb{
            .base = {.platform = VK_ICD_WSI_PLATFORM_XCB},
            .connection = XGetXCBConnection(surface.dpy),
            .window = static_cast<xcb_window_t>(surface.window),
        };
    }
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

VkResult Xlib_wsi::get_surface_support(VkIcdSurfaceBase *surface_, bool &supported) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    auto xcb_surface = Implementation::get_xcb_surface(surface);
    return Xcb_wsi::get().get_surface_support(reinterpret_cast<VkIcdSurfaceBase *>(&xcb_surface),
                                              supported);
}

VkResult Xlib_wsi::get_surface_formats(VkIcdSurfaceBase *surface_,
                                       std::vector<VkSurfaceFormatKHR> &surface_formats) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    auto xcb_surface = Implementation::get_xcb_surface(surface);
    return Xcb_wsi::get().get_surface_formats(reinterpret_cast<VkIcdSurfaceBase *>(&xcb_surface),
                                              surface_formats);
}

VkResult Xlib_wsi::get_present_modes(VkIcdSurfaceBase *surface_,
                                     std::vector<VkPresentModeKHR> &present_modes) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    auto xcb_surface = Implementation::get_xcb_surface(surface);
    return Xcb_wsi::get().get_present_modes(reinterpret_cast<VkIcdSurfaceBase *>(&xcb_surface),
                                            present_modes);
}

VkResult Xlib_wsi::get_surface_capabilities(VkIcdSurfaceBase *surface_,
                                            VkSurfaceCapabilitiesKHR &capabilities) const
{
    auto &surface = *reinterpret_cast<Surface_type *>(surface_);
    auto xcb_surface = Implementation::get_xcb_surface(surface);
    return Xcb_wsi::get().get_surface_capabilities(
        reinterpret_cast<VkIcdSurfaceBase *>(&xcb_surface), capabilities);
}

util::variant<VkResult, std::unique_ptr<Vulkan_swapchain>> Xlib_wsi::create_swapchain(
    vulkan::Vulkan_device &device, const VkSwapchainCreateInfoKHR &create_info) const
{
    assert(create_info.surface);
    auto &surface = *reinterpret_cast<Surface_type *>(create_info.surface);
    auto xcb_surface = Implementation::get_xcb_surface(surface);
    VkSwapchainCreateInfoKHR xcb_create_info = create_info;
    xcb_create_info.surface = reinterpret_cast<VkSurfaceKHR>(&xcb_surface);
    return Xcb_wsi::get().create_swapchain(device, xcb_create_info);
}

const Xlib_wsi &Xlib_wsi::get() noexcept
{
    static const Xlib_wsi retval{};
    return retval;
}
}
}
#endif
