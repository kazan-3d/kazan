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
#ifndef VULKAN_ICD_VULKAN_ICD_H_
#define VULKAN_ICD_VULKAN_ICD_H_

#include "vulkan/vulkan.h"
#include "vulkan/vk_icd.h"
#include "vulkan/remove_xlib_macros.h"
#include "vulkan/api_objects.h"
#include <type_traits>
#include <cstdint>
#include <cassert>
#include <initializer_list>
#include <memory>
#include <stdexcept>
#include <new>

extern "C" VKAPI_ATTR PFN_vkVoidFunction VKAPI_CALL vk_icdGetInstanceProcAddr(VkInstance instance,
                                                                              const char *pName);

typedef PFN_vkVoidFunction (*VKAPI_PTR PFN_vk_icdGetInstanceProcAddr)(VkInstance instance,
                                                                      const char *pName);

namespace kazan
{
namespace vulkan_icd
{
class Vulkan_loader_interface
{
public:
    enum class Version : std::uint32_t
    {
        Version_0 = 0,
        Version_1 = 1,
        Version_2 = 2,
        Version_3 = 3,
        Version_4 = 4,
        Version_5 = 5,
    };

public:
    static Vulkan_loader_interface *get() noexcept;

private:
    constexpr Vulkan_loader_interface() noexcept
    {
    }

public:
    constexpr Version get_negotiated_version() const noexcept
    {
        return negotiated_version;
    }

public:
    enum class Procedure_address_scope
    {
        Library,
        Instance,
        Device
    };
    PFN_vkVoidFunction get_procedure_address(const char *name,
                                             Procedure_address_scope scope,
                                             vulkan::Vulkan_instance *instance,
                                             vulkan::Vulkan_device *device) noexcept;
    PFN_vkVoidFunction get_instance_proc_addr(VkInstance instance, const char *name) noexcept;
    VkResult create_instance(const VkInstanceCreateInfo *create_info,
                             const VkAllocationCallbacks *allocator,
                             VkInstance *instance) noexcept;
    VkResult enumerate_instance_extension_properties(const char *layer_name,
                                                     uint32_t *property_count,
                                                     VkExtensionProperties *properties) noexcept;
    VkResult enumerate_device_extension_properties(VkPhysicalDevice physical_device,
                                                   const char *layer_name,
                                                   uint32_t *property_count,
                                                   VkExtensionProperties *properties) noexcept;

private:
    Version negotiated_version = Version::Version_0;
};

static_assert(std::is_trivially_destructible<Vulkan_loader_interface>::value, "");
static_assert(std::is_literal_type<Vulkan_loader_interface>::value, "");

template <typename T>
VkResult vulkan_enumerate_list_helper(std::uint32_t *api_value_count,
                                      T *api_values,
                                      const T *generated_values,
                                      std::size_t generated_value_count) noexcept
{
    static_assert(std::is_trivially_copyable<T>::value, "");
    assert(api_value_count);
    assert(static_cast<std::uint32_t>(generated_value_count) == generated_value_count);
    if(!api_values)
    {
        *api_value_count = generated_value_count;
        return VK_SUCCESS;
    }
    auto api_values_length = *api_value_count;
    auto copy_length = api_values_length;
    if(copy_length > generated_value_count)
        copy_length = generated_value_count;
    for(std::size_t i = 0; i < copy_length; i++)
        api_values[i] = generated_values[i];
    *api_value_count = copy_length;
    if(copy_length < generated_value_count)
        return VK_INCOMPLETE;
    return VK_SUCCESS;
}

template <typename T>
VkResult vulkan_enumerate_list_helper(std::uint32_t *api_value_count,
                                      T *api_values,
                                      std::initializer_list<T> generated_values) noexcept
{
    return vulkan_enumerate_list_helper(
        api_value_count, api_values, generated_values.begin(), generated_values.size());
}

void print_exception(std::exception &e) noexcept;

template <typename Fn>
VkResult catch_exceptions_and_return_result(Fn &&fn) noexcept
{
    try
    {
        return std::forward<Fn>(fn)();
    }
    catch(std::bad_alloc &)
    {
        return VK_ERROR_OUT_OF_HOST_MEMORY;
    }
    catch(std::exception &e)
    {
        print_exception(e);
        throw; // send to std::terminate
    }
    // explicitly don't catch other exceptions and let std::terminate handle them
}
}
}

#endif // VULKAN_ICD_VULKAN_ICD_H_
