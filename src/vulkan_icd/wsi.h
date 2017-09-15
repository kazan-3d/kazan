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
#ifndef VULKAN_ICD_WSI_H_
#define VULKAN_ICD_WSI_H_

#include "vulkan/vulkan.h"
#include "vulkan/vk_icd.h"
#include "vulkan/remove_xlib_macros.h"
#include "vulkan/api_objects.h"
#include "util/variant.h"
#include <type_traits>
#include <cstdint>
#include <vector>

namespace kazan
{
namespace vulkan_icd
{
struct Wsi;

class Vulkan_swapchain
    : public vulkan::Vulkan_nondispatchable_object<Vulkan_swapchain, VkSwapchainKHR>
{
public:
    std::vector<std::unique_ptr<vulkan::Vulkan_image>> images;

public:
    explicit Vulkan_swapchain(std::vector<std::unique_ptr<vulkan::Vulkan_image>> images) noexcept
        : images(std::move(images))
    {
    }
    virtual ~Vulkan_swapchain() = default;
};

struct Wsi
{
    class Wsi_list
    {
    public:
        typedef const Wsi *value_type;
        typedef const value_type &reference;
        typedef const value_type &const_reference;
        typedef const value_type *pointer;
        typedef const value_type *const_pointer;
        typedef const value_type *iterator;
        typedef const value_type *const_iterator;

    private:
        pointer start;
        std::size_t count;

    public:
        constexpr explicit Wsi_list(pointer start, std::size_t count) noexcept : start(start),
                                                                                 count(count)
        {
        }
        constexpr std::size_t size() const noexcept
        {
            return count;
        }
        constexpr pointer data() const noexcept
        {
            return start;
        }
        constexpr const_iterator begin() const noexcept
        {
            return start;
        }
        constexpr const_iterator end() const noexcept
        {
            return start + count;
        }
    };
    static Wsi_list get_all() noexcept;
    static const Wsi *find(VkIcdWsiPlatform surface_platform) noexcept
    {
        for(auto *wsi : get_all())
            if(wsi->surface_platform == surface_platform)
                return wsi;
        return nullptr;
    }
    VkIcdWsiPlatform surface_platform;
    constexpr explicit Wsi(VkIcdWsiPlatform surface_platform) noexcept
        : surface_platform(surface_platform)
    {
    }
    virtual void destroy_surface(VkIcdSurfaceBase *surface) const noexcept = 0;
    virtual VkResult get_surface_support(VkIcdSurfaceBase *surface, bool &supported) const = 0;
    virtual VkResult get_surface_formats(
        VkIcdSurfaceBase *surface, std::vector<VkSurfaceFormatKHR> &surface_formats) const = 0;
    virtual VkResult get_present_modes(VkIcdSurfaceBase *surface,
                                       std::vector<VkPresentModeKHR> &present_modes) const = 0;
    virtual VkResult get_surface_capabilities(VkIcdSurfaceBase *surface,
                                              VkSurfaceCapabilitiesKHR &capabilities) const = 0;
    virtual util::variant<VkResult, std::unique_ptr<Vulkan_swapchain>> create_swapchain(
        vulkan::Vulkan_device &device, const VkSwapchainCreateInfoKHR &create_info) const = 0;
};

static_assert(std::is_trivially_destructible<Wsi>::value,
              "Wsi objects are statically allocated, so we want them to be trivially destructible");

#ifdef VK_USE_PLATFORM_XCB_KHR
struct Xcb_wsi final : public Wsi
{
    typedef VkIcdSurfaceXcb Surface_type;
    constexpr Xcb_wsi() noexcept : Wsi(VK_ICD_WSI_PLATFORM_XCB)
    {
    }
    struct Implementation;
    VkIcdSurfaceBase *create_surface(const VkXcbSurfaceCreateInfoKHR &create_info) const;
    virtual void destroy_surface(VkIcdSurfaceBase *surface) const noexcept override;
    virtual VkResult get_surface_support(VkIcdSurfaceBase *surface, bool &supported) const override;
    virtual VkResult get_surface_formats(
        VkIcdSurfaceBase *surface, std::vector<VkSurfaceFormatKHR> &surface_formats) const override;
    virtual VkResult get_present_modes(VkIcdSurfaceBase *surface,
                                       std::vector<VkPresentModeKHR> &present_modes) const override;
    virtual VkResult get_surface_capabilities(
        VkIcdSurfaceBase *surface, VkSurfaceCapabilitiesKHR &capabilities) const override;
    virtual util::variant<VkResult, std::unique_ptr<Vulkan_swapchain>> create_swapchain(
        vulkan::Vulkan_device &device, const VkSwapchainCreateInfoKHR &create_info) const override;
    static const Xcb_wsi &get() noexcept;
};
#endif

#ifdef VK_USE_PLATFORM_XLIB_KHR
struct Xlib_wsi final : public Wsi
{
    typedef VkIcdSurfaceXlib Surface_type;
    constexpr Xlib_wsi() noexcept : Wsi(VK_ICD_WSI_PLATFORM_XLIB)
    {
    }
    struct Implementation;
    VkIcdSurfaceBase *create_surface(const VkXlibSurfaceCreateInfoKHR &create_info) const;
    virtual void destroy_surface(VkIcdSurfaceBase *surface) const noexcept override;
    virtual VkResult get_surface_support(VkIcdSurfaceBase *surface, bool &supported) const override;
    virtual VkResult get_surface_formats(
        VkIcdSurfaceBase *surface, std::vector<VkSurfaceFormatKHR> &surface_formats) const override;
    virtual VkResult get_present_modes(VkIcdSurfaceBase *surface,
                                       std::vector<VkPresentModeKHR> &present_modes) const override;
    virtual VkResult get_surface_capabilities(
        VkIcdSurfaceBase *surface, VkSurfaceCapabilitiesKHR &capabilities) const override;
    virtual util::variant<VkResult, std::unique_ptr<Vulkan_swapchain>> create_swapchain(
        vulkan::Vulkan_device &device, const VkSwapchainCreateInfoKHR &create_info) const override;
    static const Xlib_wsi &get() noexcept;
};
#endif

#ifdef VK_USE_PLATFORM_WAYLAND_KHR
#error Wayland wsi is not implemented
struct Wayland_wsi final : public Wsi
{
    typedef VkIcdSurfaceWayland Surface_type;
    constexpr Wayland_wsi() noexcept : Wsi(VK_ICD_WSI_PLATFORM_WAYLAND)
    {
    }
    struct Implementation;
    VkIcdSurfaceBase *create_surface(const VkWaylandSurfaceCreateInfoKHR &create_info) const;
    virtual void destroy_surface(VkIcdSurfaceBase *surface) const noexcept override;
    virtual VkResult get_surface_support(VkIcdSurfaceBase *surface, bool &supported) const override;
    virtual VkResult get_surface_formats(
        VkIcdSurfaceBase *surface, std::vector<VkSurfaceFormatKHR> &surface_formats) const override;
    virtual VkResult get_present_modes(VkIcdSurfaceBase *surface,
                                       std::vector<VkPresentModeKHR> &present_modes) const override;
    virtual VkResult get_surface_capabilities(
        VkIcdSurfaceBase *surface, VkSurfaceCapabilitiesKHR &capabilities) const override;
    virtual util::variant<VkResult, std::unique_ptr<Vulkan_swapchain>> create_swapchain(
        vulkan::Vulkan_device &device, const VkSwapchainCreateInfoKHR &create_info) const override;
    static const Wayland_wsi &get() noexcept;
};
#endif

#ifdef VK_USE_PLATFORM_MIR_KHR
#error Mir wsi is not implemented
#endif

#ifdef VK_USE_PLATFORM_ANDROID_KHR
#error Android wsi is not implemented
#endif

#ifdef VK_USE_PLATFORM_WIN32_KHR
#error Win32 wsi is not implemented
struct Win32_wsi final : public Wsi
{
    typedef VkIcdSurfaceWin32 Surface_type;
    constexpr Win32_wsi() noexcept : Wsi(VK_ICD_WSI_PLATFORM_WIN32)
    {
    }
    struct Implementation;
    VkIcdSurfaceBase *create_surface(const VkWin32SurfaceCreateInfoKHR &create_info) const;
    virtual void destroy_surface(VkIcdSurfaceBase *surface) const noexcept override;
    virtual VkResult get_surface_support(VkIcdSurfaceBase *surface, bool &supported) const override;
    virtual VkResult get_surface_formats(
        VkIcdSurfaceBase *surface, std::vector<VkSurfaceFormatKHR> &surface_formats) const override;
    virtual VkResult get_present_modes(VkIcdSurfaceBase *surface,
                                       std::vector<VkPresentModeKHR> &present_modes) const override;
    virtual VkResult get_surface_capabilities(
        VkIcdSurfaceBase *surface, VkSurfaceCapabilitiesKHR &capabilities) const override;
    virtual util::variant<VkResult, std::unique_ptr<Vulkan_swapchain>> create_swapchain(
        vulkan::Vulkan_device &device, const VkSwapchainCreateInfoKHR &create_info) const override;
    static const Win32_wsi &get() noexcept;
};
#endif
}
}

#endif // VULKAN_ICD_WSI_H_
