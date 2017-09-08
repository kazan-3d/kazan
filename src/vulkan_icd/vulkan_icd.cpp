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
#include "vulkan_icd.h"
#include "util/string_view.h"
#include <initializer_list>
#include <iostream>

using namespace kazan;

#ifdef __ELF__
#define DLLEXPORT_ATTR(original_attributes) \
    __attribute__((visibility("default"))) original_attributes
#define DLLEXPORT_CALL(original_attributes) original_attributes
#elif defined(_WIN32)
#define DLLEXPORT_ATTR(original_attributes) __declspec(dllexport) original_attributes
#define DLLEXPORT_CALL(original_attributes) original_attributes
#else
#error DLLEXPORT_* macros not implemented for platform
#endif

extern "C" DLLEXPORT_ATTR(VKAPI_ATTR) PFN_vkVoidFunction DLLEXPORT_CALL(VKAPI_CALL)
    vk_icdGetInstanceProcAddr(VkInstance instance, const char *name)
{
    return vulkan_icd::Vulkan_loader_interface::get()->get_instance_proc_addr(instance, name);
}

static_assert(static_cast<PFN_vk_icdGetInstanceProcAddr>(&vk_icdGetInstanceProcAddr), "");

static constexpr void validate_allocator(const VkAllocationCallbacks *allocator) noexcept
{
    assert(allocator == nullptr && "Vulkan allocation callbacks are not implemented");
}

extern "C" VKAPI_ATTR PFN_vkVoidFunction VKAPI_CALL vkGetInstanceProcAddr(VkInstance instance,
                                                                          const char *name)
{
    return vk_icdGetInstanceProcAddr(instance, name);
}

static_assert(static_cast<PFN_vkGetInstanceProcAddr>(&vkGetInstanceProcAddr), "");

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkCreateInstance(const VkInstanceCreateInfo *create_info,
                                                           const VkAllocationCallbacks *allocator,
                                                           VkInstance *instance)
{
    return vulkan_icd::Vulkan_loader_interface::get()->create_instance(
        create_info, allocator, instance);
}

static_assert(static_cast<PFN_vkCreateInstance>(&vkCreateInstance), "");

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkEnumerateInstanceExtensionProperties(
    const char *layer_name, uint32_t *property_count, VkExtensionProperties *properties)

{
    return vulkan_icd::Vulkan_loader_interface::get()->enumerate_instance_extension_properties(
        layer_name, property_count, properties);
}

static_assert(static_cast<PFN_vkEnumerateInstanceExtensionProperties>(
                  &vkEnumerateInstanceExtensionProperties),
              "");

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyInstance(VkInstance instance,
                                                        const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
    vulkan::Vulkan_instance::move_from_handle(instance).reset();
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkEnumeratePhysicalDevices(
    VkInstance instance, uint32_t *physical_device_count, VkPhysicalDevice *physical_devices)
{
    assert(instance);
    return vulkan_icd::catch_exceptions_and_return_result(
        [&]()
        {
            auto *instance_pointer = vulkan::Vulkan_instance::from_handle(instance);
            return vulkan_icd::vulkan_enumerate_list_helper(
                physical_device_count,
                physical_devices,
                {
                    to_handle(&instance_pointer->physical_device),
                });
        });
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetPhysicalDeviceFeatures(
    VkPhysicalDevice physical_device, VkPhysicalDeviceFeatures *features)
{
    assert(physical_device);
    assert(features);
    auto *physical_device_pointer = vulkan::Vulkan_physical_device::from_handle(physical_device);
    *features = physical_device_pointer->features;
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetPhysicalDeviceFormatProperties(
    VkPhysicalDevice physical_device, VkFormat format, VkFormatProperties *format_properties)
{
    assert(physical_device);
    assert(format_properties);
    *format_properties = vulkan::get_format_properties(format);
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkGetPhysicalDeviceImageFormatProperties(VkPhysicalDevice physicalDevice,
                                             VkFormat format,
                                             VkImageType type,
                                             VkImageTiling tiling,
                                             VkImageUsageFlags usage,
                                             VkImageCreateFlags flags,
                                             VkImageFormatProperties *pImageFormatProperties)
{
#warning finish implementing vkGetPhysicalDeviceImageFormatProperties
    assert(!"vkGetPhysicalDeviceImageFormatProperties is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetPhysicalDeviceProperties(
    VkPhysicalDevice physical_device, VkPhysicalDeviceProperties *properties)
{
    assert(physical_device);
    assert(properties);
    auto *physical_device_pointer = vulkan::Vulkan_physical_device::from_handle(physical_device);
    *properties = physical_device_pointer->properties;
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkGetPhysicalDeviceQueueFamilyProperties(VkPhysicalDevice physical_device,
                                             uint32_t *queue_family_property_count,
                                             VkQueueFamilyProperties *queue_family_properties)
{
    assert(physical_device);
    auto *physical_device_pointer = vulkan::Vulkan_physical_device::from_handle(physical_device);
    vulkan_icd::vulkan_enumerate_list_helper(
        queue_family_property_count,
        queue_family_properties,
        physical_device_pointer->queue_family_properties,
        vulkan::Vulkan_physical_device::queue_family_property_count);
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetPhysicalDeviceMemoryProperties(
    VkPhysicalDevice physical_device, VkPhysicalDeviceMemoryProperties *memory_properties)
{
    assert(physical_device);
    assert(memory_properties);
    auto *physical_device_pointer = vulkan::Vulkan_physical_device::from_handle(physical_device);
    *memory_properties = physical_device_pointer->memory_properties;
}

extern "C" VKAPI_ATTR PFN_vkVoidFunction VKAPI_CALL vkGetDeviceProcAddr(VkDevice device,
                                                                        const char *name)
{
    return vulkan_icd::Vulkan_loader_interface::get()->get_procedure_address(
        name, vulkan_icd::Vulkan_loader_interface::Procedure_address_scope::Device);
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkCreateDevice(VkPhysicalDevice physical_device,
                                                         const VkDeviceCreateInfo *create_info,
                                                         const VkAllocationCallbacks *allocator,
                                                         VkDevice *device)
{
    validate_allocator(allocator);
    assert(create_info);
    assert(physical_device);
    return vulkan_icd::catch_exceptions_and_return_result(
        [&]()
        {
            auto create_result = vulkan::Vulkan_device::create(
                *vulkan::Vulkan_physical_device::from_handle(physical_device), *create_info);
            if(util::holds_alternative<VkResult>(create_result))
                return util::get<VkResult>(create_result);
            *device = move_to_handle(
                util::get<std::unique_ptr<vulkan::Vulkan_device>>(std::move(create_result)));
            return VK_SUCCESS;
        });
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyDevice(VkDevice device,
                                                      const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
    vulkan::Vulkan_device::move_from_handle(device).reset();
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkEnumerateDeviceExtensionProperties(VkPhysicalDevice physical_device,
                                         const char *layer_name,
                                         uint32_t *property_count,
                                         VkExtensionProperties *properties)
{
    return vulkan_icd::Vulkan_loader_interface::get()->enumerate_device_extension_properties(
        physical_device, layer_name, property_count, properties);
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetDeviceQueue(VkDevice device,
                                                       uint32_t queueFamilyIndex,
                                                       uint32_t queueIndex,
                                                       VkQueue *pQueue)
{
#warning finish implementing vkGetDeviceQueue
    assert(!"vkGetDeviceQueue is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkQueueSubmit(VkQueue queue,
                                                        uint32_t submitCount,
                                                        const VkSubmitInfo *pSubmits,
                                                        VkFence fence)
{
#warning finish implementing vkQueueSubmit
    assert(!"vkQueueSubmit is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkQueueWaitIdle(VkQueue queue)
{
#warning finish implementing vkQueueWaitIdle
    assert(!"vkQueueWaitIdle is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkDeviceWaitIdle(VkDevice device)
{
    return vulkan_icd::catch_exceptions_and_return_result(
        [&]()
        {
            auto device_pointer = vulkan::Vulkan_device::from_handle(device);
            device_pointer->wait_idle();
            return VK_SUCCESS;
        });
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkAllocateMemory(VkDevice device,
                     const VkMemoryAllocateInfo *pAllocateInfo,
                     const VkAllocationCallbacks *allocator,
                     VkDeviceMemory *pMemory)
{
    validate_allocator(allocator);
#warning finish implementing vkAllocateMemory
    assert(!"vkAllocateMemory is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkFreeMemory(VkDevice device,
                                                   VkDeviceMemory memory,
                                                   const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkFreeMemory
    assert(!"vkFreeMemory is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkMapMemory(VkDevice device,
                                                      VkDeviceMemory memory,
                                                      VkDeviceSize offset,
                                                      VkDeviceSize size,
                                                      VkMemoryMapFlags flags,
                                                      void **ppData)
{
#warning finish implementing vkMapMemory
    assert(!"vkMapMemory is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkUnmapMemory(VkDevice device, VkDeviceMemory memory)
{
#warning finish implementing vkUnmapMemory
    assert(!"vkUnmapMemory is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkFlushMappedMemoryRanges(
    VkDevice device, uint32_t memoryRangeCount, const VkMappedMemoryRange *pMemoryRanges)
{
#warning finish implementing vkFlushMappedMemoryRanges
    assert(!"vkFlushMappedMemoryRanges is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkInvalidateMappedMemoryRanges(
    VkDevice device, uint32_t memoryRangeCount, const VkMappedMemoryRange *pMemoryRanges)
{
#warning finish implementing vkInvalidateMappedMemoryRanges
    assert(!"vkInvalidateMappedMemoryRanges is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetDeviceMemoryCommitment(
    VkDevice device, VkDeviceMemory memory, VkDeviceSize *pCommittedMemoryInBytes)
{
#warning finish implementing vkGetDeviceMemoryCommitment
    assert(!"vkGetDeviceMemoryCommitment is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkBindBufferMemory(VkDevice device,
                                                             VkBuffer buffer,
                                                             VkDeviceMemory memory,
                                                             VkDeviceSize memoryOffset)
{
#warning finish implementing vkBindBufferMemory
    assert(!"vkBindBufferMemory is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkBindImageMemory(VkDevice device,
                                                            VkImage image,
                                                            VkDeviceMemory memory,
                                                            VkDeviceSize memoryOffset)
{
#warning finish implementing vkBindImageMemory
    assert(!"vkBindImageMemory is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetBufferMemoryRequirements(
    VkDevice device, VkBuffer buffer, VkMemoryRequirements *pMemoryRequirements)
{
#warning finish implementing vkGetBufferMemoryRequirements
    assert(!"vkGetBufferMemoryRequirements is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetImageMemoryRequirements(
    VkDevice device, VkImage image, VkMemoryRequirements *pMemoryRequirements)
{
#warning finish implementing vkGetImageMemoryRequirements
    assert(!"vkGetImageMemoryRequirements is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkGetImageSparseMemoryRequirements(VkDevice device,
                                       VkImage image,
                                       uint32_t *pSparseMemoryRequirementCount,
                                       VkSparseImageMemoryRequirements *pSparseMemoryRequirements)
{
#warning finish implementing vkGetImageSparseMemoryRequirements
    assert(!"vkGetImageSparseMemoryRequirements is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkGetPhysicalDeviceSparseImageFormatProperties(VkPhysicalDevice physicalDevice,
                                                   VkFormat format,
                                                   VkImageType type,
                                                   VkSampleCountFlagBits samples,
                                                   VkImageUsageFlags usage,
                                                   VkImageTiling tiling,
                                                   uint32_t *pPropertyCount,
                                                   VkSparseImageFormatProperties *pProperties)
{
#warning finish implementing vkGetPhysicalDeviceSparseImageFormatProperties
    assert(!"vkGetPhysicalDeviceSparseImageFormatProperties is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkQueueBindSparse(VkQueue queue,
                                                            uint32_t bindInfoCount,
                                                            const VkBindSparseInfo *pBindInfo,
                                                            VkFence fence)
{
#warning finish implementing vkQueueBindSparse
    assert(!"vkQueueBindSparse is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkCreateFence(VkDevice device,
                                                        const VkFenceCreateInfo *pCreateInfo,
                                                        const VkAllocationCallbacks *allocator,
                                                        VkFence *pFence)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateFence
    assert(!"vkCreateFence is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyFence(VkDevice device,
                                                     VkFence fence,
                                                     const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyFence
    assert(!"vkDestroyFence is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkResetFences(VkDevice device,
                                                        uint32_t fenceCount,
                                                        const VkFence *pFences)
{
#warning finish implementing vkResetFences
    assert(!"vkResetFences is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkGetFenceStatus(VkDevice device, VkFence fence)
{
#warning finish implementing vkGetFenceStatus
    assert(!"vkGetFenceStatus is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkWaitForFences(VkDevice device,
                                                          uint32_t fenceCount,
                                                          const VkFence *pFences,
                                                          VkBool32 waitAll,
                                                          uint64_t timeout)
{
#warning finish implementing vkWaitForFences
    assert(!"vkWaitForFences is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateSemaphore(VkDevice device,
                      const VkSemaphoreCreateInfo *pCreateInfo,
                      const VkAllocationCallbacks *allocator,
                      VkSemaphore *pSemaphore)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateSemaphore
    assert(!"vkCreateSemaphore is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroySemaphore(VkDevice device,
                                                         VkSemaphore semaphore,
                                                         const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroySemaphore
    assert(!"vkDestroySemaphore is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkCreateEvent(VkDevice device,
                                                        const VkEventCreateInfo *pCreateInfo,
                                                        const VkAllocationCallbacks *allocator,
                                                        VkEvent *pEvent)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateEvent
    assert(!"vkCreateEvent is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyEvent(VkDevice device,
                                                     VkEvent event,
                                                     const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyEvent
    assert(!"vkDestroyEvent is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkGetEventStatus(VkDevice device, VkEvent event)
{
#warning finish implementing vkGetEventStatus
    assert(!"vkGetEventStatus is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkSetEvent(VkDevice device, VkEvent event)
{
#warning finish implementing vkSetEvent
    assert(!"vkSetEvent is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkResetEvent(VkDevice device, VkEvent event)
{
#warning finish implementing vkResetEvent
    assert(!"vkResetEvent is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateQueryPool(VkDevice device,
                      const VkQueryPoolCreateInfo *pCreateInfo,
                      const VkAllocationCallbacks *allocator,
                      VkQueryPool *pQueryPool)
{
#warning finish implementing vkCreateQueryPool
    assert(!"vkCreateQueryPool is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyQueryPool(VkDevice device,
                                                         VkQueryPool queryPool,
                                                         const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyQueryPool
    assert(!"vkDestroyQueryPool is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkGetQueryPoolResults(VkDevice device,
                                                                VkQueryPool queryPool,
                                                                uint32_t firstQuery,
                                                                uint32_t queryCount,
                                                                size_t dataSize,
                                                                void *pData,
                                                                VkDeviceSize stride,
                                                                VkQueryResultFlags flags)
{
#warning finish implementing vkGetQueryPoolResults
    assert(!"vkGetQueryPoolResults is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkCreateBuffer(VkDevice device,
                                                         const VkBufferCreateInfo *pCreateInfo,
                                                         const VkAllocationCallbacks *allocator,
                                                         VkBuffer *pBuffer)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateBuffer
    assert(!"vkCreateBuffer is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyBuffer(VkDevice device,
                                                      VkBuffer buffer,
                                                      const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyBuffer
    assert(!"vkDestroyBuffer is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateBufferView(VkDevice device,
                       const VkBufferViewCreateInfo *pCreateInfo,
                       const VkAllocationCallbacks *allocator,
                       VkBufferView *pView)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateBufferView
    assert(!"vkCreateBufferView is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyBufferView(VkDevice device,
                                                          VkBufferView bufferView,
                                                          const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyBufferView
    assert(!"vkDestroyBufferView is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkCreateImage(VkDevice device,
                                                        const VkImageCreateInfo *pCreateInfo,
                                                        const VkAllocationCallbacks *allocator,
                                                        VkImage *pImage)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateImage
    assert(!"vkCreateImage is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyImage(VkDevice device,
                                                     VkImage image,
                                                     const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyImage
    assert(!"vkDestroyImage is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkGetImageSubresourceLayout(VkDevice device,
                                VkImage image,
                                const VkImageSubresource *pSubresource,
                                VkSubresourceLayout *pLayout)
{
#warning finish implementing vkGetImageSubresourceLayout
    assert(!"vkGetImageSubresourceLayout is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateImageView(VkDevice device,
                      const VkImageViewCreateInfo *pCreateInfo,
                      const VkAllocationCallbacks *allocator,
                      VkImageView *pView)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateImageView
    assert(!"vkCreateImageView is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyImageView(VkDevice device,
                                                         VkImageView imageView,
                                                         const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyImageView
    assert(!"vkDestroyImageView is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateShaderModule(VkDevice device,
                         const VkShaderModuleCreateInfo *pCreateInfo,
                         const VkAllocationCallbacks *allocator,
                         VkShaderModule *pShaderModule)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateShaderModule
    assert(!"vkCreateShaderModule is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyShaderModule(VkDevice device,
                                                            VkShaderModule shaderModule,
                                                            const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyShaderModule
    assert(!"vkDestroyShaderModule is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreatePipelineCache(VkDevice device,
                          const VkPipelineCacheCreateInfo *pCreateInfo,
                          const VkAllocationCallbacks *allocator,
                          VkPipelineCache *pPipelineCache)
{
    validate_allocator(allocator);
#warning finish implementing vkCreatePipelineCache
    assert(!"vkCreatePipelineCache is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyPipelineCache(VkDevice device,
                                                             VkPipelineCache pipelineCache,
                                                             const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyPipelineCache
    assert(!"vkDestroyPipelineCache is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkGetPipelineCacheData(VkDevice device,
                                                                 VkPipelineCache pipelineCache,
                                                                 size_t *pDataSize,
                                                                 void *pData)
{
#warning finish implementing vkGetPipelineCacheData
    assert(!"vkGetPipelineCacheData is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkMergePipelineCaches(VkDevice device,
                                                                VkPipelineCache dstCache,
                                                                uint32_t srcCacheCount,
                                                                const VkPipelineCache *pSrcCaches)
{
#warning finish implementing vkMergePipelineCaches
    assert(!"vkMergePipelineCaches is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateGraphicsPipelines(VkDevice device,
                              VkPipelineCache pipelineCache,
                              uint32_t createInfoCount,
                              const VkGraphicsPipelineCreateInfo *pCreateInfos,
                              const VkAllocationCallbacks *allocator,
                              VkPipeline *pPipelines)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateGraphicsPipelines
    assert(!"vkCreateGraphicsPipelines is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateComputePipelines(VkDevice device,
                             VkPipelineCache pipelineCache,
                             uint32_t createInfoCount,
                             const VkComputePipelineCreateInfo *pCreateInfos,
                             const VkAllocationCallbacks *allocator,
                             VkPipeline *pPipelines)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateComputePipelines
    assert(!"vkCreateComputePipelines is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyPipeline(VkDevice device,
                                                        VkPipeline pipeline,
                                                        const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyPipeline
    assert(!"vkDestroyPipeline is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreatePipelineLayout(VkDevice device,
                           const VkPipelineLayoutCreateInfo *pCreateInfo,
                           const VkAllocationCallbacks *allocator,
                           VkPipelineLayout *pPipelineLayout)
{
    validate_allocator(allocator);
#warning finish implementing vkCreatePipelineLayout
    assert(!"vkCreatePipelineLayout is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyPipelineLayout(
    VkDevice device, VkPipelineLayout pipelineLayout, const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyPipelineLayout
    assert(!"vkDestroyPipelineLayout is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkCreateSampler(VkDevice device,
                                                          const VkSamplerCreateInfo *pCreateInfo,
                                                          const VkAllocationCallbacks *allocator,
                                                          VkSampler *pSampler)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateSampler
    assert(!"vkCreateSampler is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroySampler(VkDevice device,
                                                       VkSampler sampler,
                                                       const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroySampler
    assert(!"vkDestroySampler is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateDescriptorSetLayout(VkDevice device,
                                const VkDescriptorSetLayoutCreateInfo *pCreateInfo,
                                const VkAllocationCallbacks *allocator,
                                VkDescriptorSetLayout *pSetLayout)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateDescriptorSetLayout
    assert(!"vkCreateDescriptorSetLayout is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkDestroyDescriptorSetLayout(VkDevice device,
                                 VkDescriptorSetLayout descriptorSetLayout,
                                 const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyDescriptorSetLayout
    assert(!"vkDestroyDescriptorSetLayout is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateDescriptorPool(VkDevice device,
                           const VkDescriptorPoolCreateInfo *pCreateInfo,
                           const VkAllocationCallbacks *allocator,
                           VkDescriptorPool *pDescriptorPool)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateDescriptorPool
    assert(!"vkCreateDescriptorPool is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyDescriptorPool(
    VkDevice device, VkDescriptorPool descriptorPool, const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyDescriptorPool
    assert(!"vkDestroyDescriptorPool is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkResetDescriptorPool(VkDevice device,
                                                                VkDescriptorPool descriptorPool,
                                                                VkDescriptorPoolResetFlags flags)
{
#warning finish implementing vkResetDescriptorPool
    assert(!"vkResetDescriptorPool is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkAllocateDescriptorSets(VkDevice device,
                             const VkDescriptorSetAllocateInfo *pAllocateInfo,
                             VkDescriptorSet *pDescriptorSets)
{
#warning finish implementing vkAllocateDescriptorSets
    assert(!"vkAllocateDescriptorSets is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkFreeDescriptorSets(VkDevice device,
                         VkDescriptorPool descriptorPool,
                         uint32_t descriptorSetCount,
                         const VkDescriptorSet *pDescriptorSets)
{
#warning finish implementing vkFreeDescriptorSets
    assert(!"vkFreeDescriptorSets is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkUpdateDescriptorSets(VkDevice device,
                           uint32_t descriptorWriteCount,
                           const VkWriteDescriptorSet *pDescriptorWrites,
                           uint32_t descriptorCopyCount,
                           const VkCopyDescriptorSet *pDescriptorCopies)
{
#warning finish implementing vkUpdateDescriptorSets
    assert(!"vkUpdateDescriptorSets is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateFramebuffer(VkDevice device,
                        const VkFramebufferCreateInfo *pCreateInfo,
                        const VkAllocationCallbacks *allocator,
                        VkFramebuffer *pFramebuffer)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateFramebuffer
    assert(!"vkCreateFramebuffer is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyFramebuffer(VkDevice device,
                                                           VkFramebuffer framebuffer,
                                                           const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyFramebuffer
    assert(!"vkDestroyFramebuffer is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateRenderPass(VkDevice device,
                       const VkRenderPassCreateInfo *pCreateInfo,
                       const VkAllocationCallbacks *allocator,
                       VkRenderPass *pRenderPass)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateRenderPass
    assert(!"vkCreateRenderPass is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyRenderPass(VkDevice device,
                                                          VkRenderPass renderPass,
                                                          const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyRenderPass
    assert(!"vkDestroyRenderPass is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkGetRenderAreaGranularity(VkDevice device,
                                                                 VkRenderPass renderPass,
                                                                 VkExtent2D *pGranularity)
{
#warning finish implementing vkGetRenderAreaGranularity
    assert(!"vkGetRenderAreaGranularity is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkCreateCommandPool(VkDevice device,
                        const VkCommandPoolCreateInfo *pCreateInfo,
                        const VkAllocationCallbacks *allocator,
                        VkCommandPool *pCommandPool)
{
    validate_allocator(allocator);
#warning finish implementing vkCreateCommandPool
    assert(!"vkCreateCommandPool is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkDestroyCommandPool(VkDevice device,
                                                           VkCommandPool commandPool,
                                                           const VkAllocationCallbacks *allocator)
{
    validate_allocator(allocator);
#warning finish implementing vkDestroyCommandPool
    assert(!"vkDestroyCommandPool is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkResetCommandPool(VkDevice device,
                                                             VkCommandPool commandPool,
                                                             VkCommandPoolResetFlags flags)
{
#warning finish implementing vkResetCommandPool
    assert(!"vkResetCommandPool is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkAllocateCommandBuffers(VkDevice device,
                             const VkCommandBufferAllocateInfo *pAllocateInfo,
                             VkCommandBuffer *pCommandBuffers)
{
#warning finish implementing vkAllocateCommandBuffers
    assert(!"vkAllocateCommandBuffers is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkFreeCommandBuffers(VkDevice device,
                                                           VkCommandPool commandPool,
                                                           uint32_t commandBufferCount,
                                                           const VkCommandBuffer *pCommandBuffers)
{
#warning finish implementing vkFreeCommandBuffers
    assert(!"vkFreeCommandBuffers is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL
    vkBeginCommandBuffer(VkCommandBuffer commandBuffer, const VkCommandBufferBeginInfo *pBeginInfo)
{
#warning finish implementing vkBeginCommandBuffer
    assert(!"vkBeginCommandBuffer is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkEndCommandBuffer(VkCommandBuffer commandBuffer)
{
#warning finish implementing vkEndCommandBuffer
    assert(!"vkEndCommandBuffer is not implemented");
}

extern "C" VKAPI_ATTR VkResult VKAPI_CALL vkResetCommandBuffer(VkCommandBuffer commandBuffer,
                                                               VkCommandBufferResetFlags flags)
{
#warning finish implementing vkResetCommandBuffer
    assert(!"vkResetCommandBuffer is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdBindPipeline(VkCommandBuffer commandBuffer,
                                                        VkPipelineBindPoint pipelineBindPoint,
                                                        VkPipeline pipeline)
{
#warning finish implementing vkCmdBindPipeline
    assert(!"vkCmdBindPipeline is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetViewport(VkCommandBuffer commandBuffer,
                                                       uint32_t firstViewport,
                                                       uint32_t viewportCount,
                                                       const VkViewport *pViewports)
{
#warning finish implementing vkCmdSetViewport
    assert(!"vkCmdSetViewport is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetScissor(VkCommandBuffer commandBuffer,
                                                      uint32_t firstScissor,
                                                      uint32_t scissorCount,
                                                      const VkRect2D *pScissors)
{
#warning finish implementing vkCmdSetScissor
    assert(!"vkCmdSetScissor is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetLineWidth(VkCommandBuffer commandBuffer,
                                                        float lineWidth)
{
#warning finish implementing vkCmdSetLineWidth
    assert(!"vkCmdSetLineWidth is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetDepthBias(VkCommandBuffer commandBuffer,
                                                        float depthBiasConstantFactor,
                                                        float depthBiasClamp,
                                                        float depthBiasSlopeFactor)
{
#warning finish implementing vkCmdSetDepthBias
    assert(!"vkCmdSetDepthBias is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetBlendConstants(VkCommandBuffer commandBuffer,
                                                             const float blendConstants[4])
{
#warning finish implementing vkCmdSetBlendConstants
    assert(!"vkCmdSetBlendConstants is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetDepthBounds(VkCommandBuffer commandBuffer,
                                                          float minDepthBounds,
                                                          float maxDepthBounds)
{
#warning finish implementing vkCmdSetDepthBounds
    assert(!"vkCmdSetDepthBounds is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetStencilCompareMask(VkCommandBuffer commandBuffer,
                                                                 VkStencilFaceFlags faceMask,
                                                                 uint32_t compareMask)
{
#warning finish implementing vkCmdSetStencilCompareMask
    assert(!"vkCmdSetStencilCompareMask is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetStencilWriteMask(VkCommandBuffer commandBuffer,
                                                               VkStencilFaceFlags faceMask,
                                                               uint32_t writeMask)
{
#warning finish implementing vkCmdSetStencilWriteMask
    assert(!"vkCmdSetStencilWriteMask is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetStencilReference(VkCommandBuffer commandBuffer,
                                                               VkStencilFaceFlags faceMask,
                                                               uint32_t reference)
{
#warning finish implementing vkCmdSetStencilReference
    assert(!"vkCmdSetStencilReference is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkCmdBindDescriptorSets(VkCommandBuffer commandBuffer,
                            VkPipelineBindPoint pipelineBindPoint,
                            VkPipelineLayout layout,
                            uint32_t firstSet,
                            uint32_t descriptorSetCount,
                            const VkDescriptorSet *pDescriptorSets,
                            uint32_t dynamicOffsetCount,
                            const uint32_t *pDynamicOffsets)
{
#warning finish implementing vkCmdBindDescriptorSets
    assert(!"vkCmdBindDescriptorSets is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdBindIndexBuffer(VkCommandBuffer commandBuffer,
                                                           VkBuffer buffer,
                                                           VkDeviceSize offset,
                                                           VkIndexType indexType)
{
#warning finish implementing vkCmdBindIndexBuffer
    assert(!"vkCmdBindIndexBuffer is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdBindVertexBuffers(VkCommandBuffer commandBuffer,
                                                             uint32_t firstBinding,
                                                             uint32_t bindingCount,
                                                             const VkBuffer *pBuffers,
                                                             const VkDeviceSize *pOffsets)
{
#warning finish implementing vkCmdBindVertexBuffers
    assert(!"vkCmdBindVertexBuffers is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdDraw(VkCommandBuffer commandBuffer,
                                                uint32_t vertexCount,
                                                uint32_t instanceCount,
                                                uint32_t firstVertex,
                                                uint32_t firstInstance)
{
#warning finish implementing vkCmdDraw
    assert(!"vkCmdDraw is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdDrawIndexed(VkCommandBuffer commandBuffer,
                                                       uint32_t indexCount,
                                                       uint32_t instanceCount,
                                                       uint32_t firstIndex,
                                                       int32_t vertexOffset,
                                                       uint32_t firstInstance)
{
#warning finish implementing vkCmdDrawIndexed
    assert(!"vkCmdDrawIndexed is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdDrawIndirect(VkCommandBuffer commandBuffer,
                                                        VkBuffer buffer,
                                                        VkDeviceSize offset,
                                                        uint32_t drawCount,
                                                        uint32_t stride)
{
#warning finish implementing vkCmdDrawIndirect
    assert(!"vkCmdDrawIndirect is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdDrawIndexedIndirect(VkCommandBuffer commandBuffer,
                                                               VkBuffer buffer,
                                                               VkDeviceSize offset,
                                                               uint32_t drawCount,
                                                               uint32_t stride)
{
#warning finish implementing vkCmdDrawIndexedIndirect
    assert(!"vkCmdDrawIndexedIndirect is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdDispatch(VkCommandBuffer commandBuffer,
                                                    uint32_t groupCountX,
                                                    uint32_t groupCountY,
                                                    uint32_t groupCountZ)
{
#warning finish implementing vkCmdDispatch
    assert(!"vkCmdDispatch is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdDispatchIndirect(VkCommandBuffer commandBuffer,
                                                            VkBuffer buffer,
                                                            VkDeviceSize offset)
{
#warning finish implementing vkCmdDispatchIndirect
    assert(!"vkCmdDispatchIndirect is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdCopyBuffer(VkCommandBuffer commandBuffer,
                                                      VkBuffer srcBuffer,
                                                      VkBuffer dstBuffer,
                                                      uint32_t regionCount,
                                                      const VkBufferCopy *pRegions)
{
#warning finish implementing vkCmdCopyBuffer
    assert(!"vkCmdCopyBuffer is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdCopyImage(VkCommandBuffer commandBuffer,
                                                     VkImage srcImage,
                                                     VkImageLayout srcImageLayout,
                                                     VkImage dstImage,
                                                     VkImageLayout dstImageLayout,
                                                     uint32_t regionCount,
                                                     const VkImageCopy *pRegions)
{
#warning finish implementing vkCmdCopyImage
    assert(!"vkCmdCopyImage is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdBlitImage(VkCommandBuffer commandBuffer,
                                                     VkImage srcImage,
                                                     VkImageLayout srcImageLayout,
                                                     VkImage dstImage,
                                                     VkImageLayout dstImageLayout,
                                                     uint32_t regionCount,
                                                     const VkImageBlit *pRegions,
                                                     VkFilter filter)
{
#warning finish implementing vkCmdBlitImage
    assert(!"vkCmdBlitImage is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdCopyBufferToImage(VkCommandBuffer commandBuffer,
                                                             VkBuffer srcBuffer,
                                                             VkImage dstImage,
                                                             VkImageLayout dstImageLayout,
                                                             uint32_t regionCount,
                                                             const VkBufferImageCopy *pRegions)
{
#warning finish implementing vkCmdCopyBufferToImage
    assert(!"vkCmdCopyBufferToImage is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdCopyImageToBuffer(VkCommandBuffer commandBuffer,
                                                             VkImage srcImage,
                                                             VkImageLayout srcImageLayout,
                                                             VkBuffer dstBuffer,
                                                             uint32_t regionCount,
                                                             const VkBufferImageCopy *pRegions)
{
#warning finish implementing vkCmdCopyImageToBuffer
    assert(!"vkCmdCopyImageToBuffer is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdUpdateBuffer(VkCommandBuffer commandBuffer,
                                                        VkBuffer dstBuffer,
                                                        VkDeviceSize dstOffset,
                                                        VkDeviceSize dataSize,
                                                        const void *pData)
{
#warning finish implementing vkCmdUpdateBuffer
    assert(!"vkCmdUpdateBuffer is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdFillBuffer(VkCommandBuffer commandBuffer,
                                                      VkBuffer dstBuffer,
                                                      VkDeviceSize dstOffset,
                                                      VkDeviceSize size,
                                                      uint32_t data)
{
#warning finish implementing vkCmdFillBuffer
    assert(!"vkCmdFillBuffer is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdClearColorImage(VkCommandBuffer commandBuffer,
                                                           VkImage image,
                                                           VkImageLayout imageLayout,
                                                           const VkClearColorValue *pColor,
                                                           uint32_t rangeCount,
                                                           const VkImageSubresourceRange *pRanges)
{
#warning finish implementing vkCmdClearColorImage
    assert(!"vkCmdClearColorImage is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkCmdClearDepthStencilImage(VkCommandBuffer commandBuffer,
                                VkImage image,
                                VkImageLayout imageLayout,
                                const VkClearDepthStencilValue *pDepthStencil,
                                uint32_t rangeCount,
                                const VkImageSubresourceRange *pRanges)
{
#warning finish implementing vkCmdClearDepthStencilImage
    assert(!"vkCmdClearDepthStencilImage is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdClearAttachments(VkCommandBuffer commandBuffer,
                                                            uint32_t attachmentCount,
                                                            const VkClearAttachment *pAttachments,
                                                            uint32_t rectCount,
                                                            const VkClearRect *pRects)
{
#warning finish implementing vkCmdClearAttachments
    assert(!"vkCmdClearAttachments is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdResolveImage(VkCommandBuffer commandBuffer,
                                                        VkImage srcImage,
                                                        VkImageLayout srcImageLayout,
                                                        VkImage dstImage,
                                                        VkImageLayout dstImageLayout,
                                                        uint32_t regionCount,
                                                        const VkImageResolve *pRegions)
{
#warning finish implementing vkCmdResolveImage
    assert(!"vkCmdResolveImage is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdSetEvent(VkCommandBuffer commandBuffer,
                                                    VkEvent event,
                                                    VkPipelineStageFlags stageMask)
{
#warning finish implementing vkCmdSetEvent
    assert(!"vkCmdSetEvent is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdResetEvent(VkCommandBuffer commandBuffer,
                                                      VkEvent event,
                                                      VkPipelineStageFlags stageMask)
{
#warning finish implementing vkCmdResetEvent
    assert(!"vkCmdResetEvent is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkCmdWaitEvents(VkCommandBuffer commandBuffer,
                    uint32_t eventCount,
                    const VkEvent *pEvents,
                    VkPipelineStageFlags srcStageMask,
                    VkPipelineStageFlags dstStageMask,
                    uint32_t memoryBarrierCount,
                    const VkMemoryBarrier *pMemoryBarriers,
                    uint32_t bufferMemoryBarrierCount,
                    const VkBufferMemoryBarrier *pBufferMemoryBarriers,
                    uint32_t imageMemoryBarrierCount,
                    const VkImageMemoryBarrier *pImageMemoryBarriers)
{
#warning finish implementing vkCmdWaitEvents
    assert(!"vkCmdWaitEvents is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkCmdPipelineBarrier(VkCommandBuffer commandBuffer,
                         VkPipelineStageFlags srcStageMask,
                         VkPipelineStageFlags dstStageMask,
                         VkDependencyFlags dependencyFlags,
                         uint32_t memoryBarrierCount,
                         const VkMemoryBarrier *pMemoryBarriers,
                         uint32_t bufferMemoryBarrierCount,
                         const VkBufferMemoryBarrier *pBufferMemoryBarriers,
                         uint32_t imageMemoryBarrierCount,
                         const VkImageMemoryBarrier *pImageMemoryBarriers)
{
#warning finish implementing vkCmdPipelineBarrier
    assert(!"vkCmdPipelineBarrier is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdBeginQuery(VkCommandBuffer commandBuffer,
                                                      VkQueryPool queryPool,
                                                      uint32_t query,
                                                      VkQueryControlFlags flags)
{
#warning finish implementing vkCmdBeginQuery
    assert(!"vkCmdBeginQuery is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdEndQuery(VkCommandBuffer commandBuffer,
                                                    VkQueryPool queryPool,
                                                    uint32_t query)
{
#warning finish implementing vkCmdEndQuery
    assert(!"vkCmdEndQuery is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdResetQueryPool(VkCommandBuffer commandBuffer,
                                                          VkQueryPool queryPool,
                                                          uint32_t firstQuery,
                                                          uint32_t queryCount)
{
#warning finish implementing vkCmdResetQueryPool
    assert(!"vkCmdResetQueryPool is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdWriteTimestamp(VkCommandBuffer commandBuffer,
                                                          VkPipelineStageFlagBits pipelineStage,
                                                          VkQueryPool queryPool,
                                                          uint32_t query)
{
#warning finish implementing vkCmdWriteTimestamp
    assert(!"vkCmdWriteTimestamp is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdCopyQueryPoolResults(VkCommandBuffer commandBuffer,
                                                                VkQueryPool queryPool,
                                                                uint32_t firstQuery,
                                                                uint32_t queryCount,
                                                                VkBuffer dstBuffer,
                                                                VkDeviceSize dstOffset,
                                                                VkDeviceSize stride,
                                                                VkQueryResultFlags flags)
{
#warning finish implementing vkCmdCopyQueryPoolResults
    assert(!"vkCmdCopyQueryPoolResults is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdPushConstants(VkCommandBuffer commandBuffer,
                                                         VkPipelineLayout layout,
                                                         VkShaderStageFlags stageFlags,
                                                         uint32_t offset,
                                                         uint32_t size,
                                                         const void *pValues)
{
#warning finish implementing vkCmdPushConstants
    assert(!"vkCmdPushConstants is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL
    vkCmdBeginRenderPass(VkCommandBuffer commandBuffer,
                         const VkRenderPassBeginInfo *pRenderPassBegin,
                         VkSubpassContents contents)
{
#warning finish implementing vkCmdBeginRenderPass
    assert(!"vkCmdBeginRenderPass is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdNextSubpass(VkCommandBuffer commandBuffer,
                                                       VkSubpassContents contents)
{
#warning finish implementing vkCmdNextSubpass
    assert(!"vkCmdNextSubpass is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdEndRenderPass(VkCommandBuffer commandBuffer)
{
#warning finish implementing vkCmdEndRenderPass
    assert(!"vkCmdEndRenderPass is not implemented");
}

extern "C" VKAPI_ATTR void VKAPI_CALL vkCmdExecuteCommands(VkCommandBuffer commandBuffer,
                                                           uint32_t commandBufferCount,
                                                           const VkCommandBuffer *pCommandBuffers)
{
#warning finish implementing vkCmdExecuteCommands
    assert(!"vkCmdExecuteCommands is not implemented");
}
namespace kazan
{
namespace vulkan_icd
{
Vulkan_loader_interface *Vulkan_loader_interface::get() noexcept
{
    static Vulkan_loader_interface vulkan_loader_interface{};
    return &vulkan_loader_interface;
}

PFN_vkVoidFunction Vulkan_loader_interface::get_procedure_address(
    const char *name, Procedure_address_scope scope) noexcept
{
    using namespace util::string_view_literals;
    assert(name != "vkEnumerateInstanceLayerProperties"_sv
           && "shouldn't be called, implemented by the vulkan loader");
    assert(name != "vkEnumerateDeviceLayerProperties"_sv
           && "shouldn't be called, implemented by the vulkan loader");

#define LIBRARY_SCOPE_FUNCTION(function_name)                     \
    do                                                            \
    {                                                             \
        if(name == #function_name##_sv)                           \
            return reinterpret_cast<PFN_vkVoidFunction>(          \
                static_cast<PFN_##function_name>(function_name)); \
    } while(0)
#define INSTANCE_SCOPE_FUNCTION(function_name)                                       \
    do                                                                               \
    {                                                                                \
        if(scope != Procedure_address_scope::Library && name == #function_name##_sv) \
            return reinterpret_cast<PFN_vkVoidFunction>(                             \
                static_cast<PFN_##function_name>(function_name));                    \
    } while(0)

    LIBRARY_SCOPE_FUNCTION(vkEnumerateInstanceExtensionProperties);
    LIBRARY_SCOPE_FUNCTION(vkCreateInstance);
    INSTANCE_SCOPE_FUNCTION(vkDestroyInstance);
    INSTANCE_SCOPE_FUNCTION(vkEnumeratePhysicalDevices);
    INSTANCE_SCOPE_FUNCTION(vkGetPhysicalDeviceFeatures);
    INSTANCE_SCOPE_FUNCTION(vkGetPhysicalDeviceFormatProperties);
    INSTANCE_SCOPE_FUNCTION(vkGetPhysicalDeviceImageFormatProperties);
    INSTANCE_SCOPE_FUNCTION(vkGetPhysicalDeviceProperties);
    INSTANCE_SCOPE_FUNCTION(vkGetPhysicalDeviceQueueFamilyProperties);
    INSTANCE_SCOPE_FUNCTION(vkGetPhysicalDeviceMemoryProperties);
    INSTANCE_SCOPE_FUNCTION(vkGetInstanceProcAddr);
    INSTANCE_SCOPE_FUNCTION(vkGetDeviceProcAddr);
    INSTANCE_SCOPE_FUNCTION(vkCreateDevice);
    INSTANCE_SCOPE_FUNCTION(vkDestroyDevice);
    INSTANCE_SCOPE_FUNCTION(vkEnumerateDeviceExtensionProperties);
    INSTANCE_SCOPE_FUNCTION(vkGetDeviceQueue);
    INSTANCE_SCOPE_FUNCTION(vkQueueSubmit);
    INSTANCE_SCOPE_FUNCTION(vkQueueWaitIdle);
    INSTANCE_SCOPE_FUNCTION(vkDeviceWaitIdle);
    INSTANCE_SCOPE_FUNCTION(vkAllocateMemory);
    INSTANCE_SCOPE_FUNCTION(vkFreeMemory);
    INSTANCE_SCOPE_FUNCTION(vkMapMemory);
    INSTANCE_SCOPE_FUNCTION(vkUnmapMemory);
    INSTANCE_SCOPE_FUNCTION(vkFlushMappedMemoryRanges);
    INSTANCE_SCOPE_FUNCTION(vkInvalidateMappedMemoryRanges);
    INSTANCE_SCOPE_FUNCTION(vkGetDeviceMemoryCommitment);
    INSTANCE_SCOPE_FUNCTION(vkBindBufferMemory);
    INSTANCE_SCOPE_FUNCTION(vkBindImageMemory);
    INSTANCE_SCOPE_FUNCTION(vkGetBufferMemoryRequirements);
    INSTANCE_SCOPE_FUNCTION(vkGetImageMemoryRequirements);
    INSTANCE_SCOPE_FUNCTION(vkGetImageSparseMemoryRequirements);
    INSTANCE_SCOPE_FUNCTION(vkGetPhysicalDeviceSparseImageFormatProperties);
    INSTANCE_SCOPE_FUNCTION(vkQueueBindSparse);
    INSTANCE_SCOPE_FUNCTION(vkCreateFence);
    INSTANCE_SCOPE_FUNCTION(vkDestroyFence);
    INSTANCE_SCOPE_FUNCTION(vkResetFences);
    INSTANCE_SCOPE_FUNCTION(vkGetFenceStatus);
    INSTANCE_SCOPE_FUNCTION(vkWaitForFences);
    INSTANCE_SCOPE_FUNCTION(vkCreateSemaphore);
    INSTANCE_SCOPE_FUNCTION(vkDestroySemaphore);
    INSTANCE_SCOPE_FUNCTION(vkCreateEvent);
    INSTANCE_SCOPE_FUNCTION(vkDestroyEvent);
    INSTANCE_SCOPE_FUNCTION(vkGetEventStatus);
    INSTANCE_SCOPE_FUNCTION(vkSetEvent);
    INSTANCE_SCOPE_FUNCTION(vkResetEvent);
    INSTANCE_SCOPE_FUNCTION(vkCreateQueryPool);
    INSTANCE_SCOPE_FUNCTION(vkDestroyQueryPool);
    INSTANCE_SCOPE_FUNCTION(vkGetQueryPoolResults);
    INSTANCE_SCOPE_FUNCTION(vkCreateBuffer);
    INSTANCE_SCOPE_FUNCTION(vkDestroyBuffer);
    INSTANCE_SCOPE_FUNCTION(vkCreateBufferView);
    INSTANCE_SCOPE_FUNCTION(vkDestroyBufferView);
    INSTANCE_SCOPE_FUNCTION(vkCreateImage);
    INSTANCE_SCOPE_FUNCTION(vkDestroyImage);
    INSTANCE_SCOPE_FUNCTION(vkGetImageSubresourceLayout);
    INSTANCE_SCOPE_FUNCTION(vkCreateImageView);
    INSTANCE_SCOPE_FUNCTION(vkDestroyImageView);
    INSTANCE_SCOPE_FUNCTION(vkCreateShaderModule);
    INSTANCE_SCOPE_FUNCTION(vkDestroyShaderModule);
    INSTANCE_SCOPE_FUNCTION(vkCreatePipelineCache);
    INSTANCE_SCOPE_FUNCTION(vkDestroyPipelineCache);
    INSTANCE_SCOPE_FUNCTION(vkGetPipelineCacheData);
    INSTANCE_SCOPE_FUNCTION(vkMergePipelineCaches);
    INSTANCE_SCOPE_FUNCTION(vkCreateGraphicsPipelines);
    INSTANCE_SCOPE_FUNCTION(vkCreateComputePipelines);
    INSTANCE_SCOPE_FUNCTION(vkDestroyPipeline);
    INSTANCE_SCOPE_FUNCTION(vkCreatePipelineLayout);
    INSTANCE_SCOPE_FUNCTION(vkDestroyPipelineLayout);
    INSTANCE_SCOPE_FUNCTION(vkCreateSampler);
    INSTANCE_SCOPE_FUNCTION(vkDestroySampler);
    INSTANCE_SCOPE_FUNCTION(vkCreateDescriptorSetLayout);
    INSTANCE_SCOPE_FUNCTION(vkDestroyDescriptorSetLayout);
    INSTANCE_SCOPE_FUNCTION(vkCreateDescriptorPool);
    INSTANCE_SCOPE_FUNCTION(vkDestroyDescriptorPool);
    INSTANCE_SCOPE_FUNCTION(vkResetDescriptorPool);
    INSTANCE_SCOPE_FUNCTION(vkAllocateDescriptorSets);
    INSTANCE_SCOPE_FUNCTION(vkFreeDescriptorSets);
    INSTANCE_SCOPE_FUNCTION(vkUpdateDescriptorSets);
    INSTANCE_SCOPE_FUNCTION(vkCreateFramebuffer);
    INSTANCE_SCOPE_FUNCTION(vkDestroyFramebuffer);
    INSTANCE_SCOPE_FUNCTION(vkCreateRenderPass);
    INSTANCE_SCOPE_FUNCTION(vkDestroyRenderPass);
    INSTANCE_SCOPE_FUNCTION(vkGetRenderAreaGranularity);
    INSTANCE_SCOPE_FUNCTION(vkCreateCommandPool);
    INSTANCE_SCOPE_FUNCTION(vkDestroyCommandPool);
    INSTANCE_SCOPE_FUNCTION(vkResetCommandPool);
    INSTANCE_SCOPE_FUNCTION(vkAllocateCommandBuffers);
    INSTANCE_SCOPE_FUNCTION(vkFreeCommandBuffers);
    INSTANCE_SCOPE_FUNCTION(vkBeginCommandBuffer);
    INSTANCE_SCOPE_FUNCTION(vkEndCommandBuffer);
    INSTANCE_SCOPE_FUNCTION(vkResetCommandBuffer);
    INSTANCE_SCOPE_FUNCTION(vkCmdBindPipeline);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetViewport);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetScissor);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetLineWidth);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetDepthBias);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetBlendConstants);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetDepthBounds);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetStencilCompareMask);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetStencilWriteMask);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetStencilReference);
    INSTANCE_SCOPE_FUNCTION(vkCmdBindDescriptorSets);
    INSTANCE_SCOPE_FUNCTION(vkCmdBindIndexBuffer);
    INSTANCE_SCOPE_FUNCTION(vkCmdBindVertexBuffers);
    INSTANCE_SCOPE_FUNCTION(vkCmdDraw);
    INSTANCE_SCOPE_FUNCTION(vkCmdDrawIndexed);
    INSTANCE_SCOPE_FUNCTION(vkCmdDrawIndirect);
    INSTANCE_SCOPE_FUNCTION(vkCmdDrawIndexedIndirect);
    INSTANCE_SCOPE_FUNCTION(vkCmdDispatch);
    INSTANCE_SCOPE_FUNCTION(vkCmdDispatchIndirect);
    INSTANCE_SCOPE_FUNCTION(vkCmdCopyBuffer);
    INSTANCE_SCOPE_FUNCTION(vkCmdCopyImage);
    INSTANCE_SCOPE_FUNCTION(vkCmdBlitImage);
    INSTANCE_SCOPE_FUNCTION(vkCmdCopyBufferToImage);
    INSTANCE_SCOPE_FUNCTION(vkCmdCopyImageToBuffer);
    INSTANCE_SCOPE_FUNCTION(vkCmdUpdateBuffer);
    INSTANCE_SCOPE_FUNCTION(vkCmdFillBuffer);
    INSTANCE_SCOPE_FUNCTION(vkCmdClearColorImage);
    INSTANCE_SCOPE_FUNCTION(vkCmdClearDepthStencilImage);
    INSTANCE_SCOPE_FUNCTION(vkCmdClearAttachments);
    INSTANCE_SCOPE_FUNCTION(vkCmdResolveImage);
    INSTANCE_SCOPE_FUNCTION(vkCmdSetEvent);
    INSTANCE_SCOPE_FUNCTION(vkCmdResetEvent);
    INSTANCE_SCOPE_FUNCTION(vkCmdWaitEvents);
    INSTANCE_SCOPE_FUNCTION(vkCmdPipelineBarrier);
    INSTANCE_SCOPE_FUNCTION(vkCmdBeginQuery);
    INSTANCE_SCOPE_FUNCTION(vkCmdEndQuery);
    INSTANCE_SCOPE_FUNCTION(vkCmdResetQueryPool);
    INSTANCE_SCOPE_FUNCTION(vkCmdWriteTimestamp);
    INSTANCE_SCOPE_FUNCTION(vkCmdCopyQueryPoolResults);
    INSTANCE_SCOPE_FUNCTION(vkCmdPushConstants);
    INSTANCE_SCOPE_FUNCTION(vkCmdBeginRenderPass);
    INSTANCE_SCOPE_FUNCTION(vkCmdNextSubpass);
    INSTANCE_SCOPE_FUNCTION(vkCmdEndRenderPass);
    INSTANCE_SCOPE_FUNCTION(vkCmdExecuteCommands);

#undef LIBRARY_SCOPE_FUNCTION
#undef INSTANCE_SCOPE_FUNCTION
    return nullptr;
}

PFN_vkVoidFunction Vulkan_loader_interface::get_instance_proc_addr(VkInstance instance,
                                                                   const char *name) noexcept
{
    if(!instance)
        return get_procedure_address(name, Procedure_address_scope::Library);
    return get_procedure_address(name, Procedure_address_scope::Instance);
}

VkResult Vulkan_loader_interface::create_instance(const VkInstanceCreateInfo *create_info,
                                                  const VkAllocationCallbacks *allocator,
                                                  VkInstance *instance) noexcept
{
    validate_allocator(allocator);
    assert(create_info);
    assert(instance);
    return catch_exceptions_and_return_result(
        [&]()
        {
            auto create_result = vulkan::Vulkan_instance::create(*create_info);
            if(util::holds_alternative<VkResult>(create_result))
                return util::get<VkResult>(create_result);
            *instance = move_to_handle(
                util::get<std::unique_ptr<vulkan::Vulkan_instance>>(std::move(create_result)));
            return VK_SUCCESS;
        });
}

VkResult Vulkan_loader_interface::enumerate_instance_extension_properties(
    const char *layer_name, uint32_t *property_count, VkExtensionProperties *properties) noexcept
{
    assert(layer_name == nullptr);
    static constexpr auto extensions = vulkan::get_extensions<vulkan::Extension_scope::Instance>();
    return vulkan_enumerate_list_helper(
        property_count, properties, extensions.data(), extensions.size());
}

VkResult Vulkan_loader_interface::enumerate_device_extension_properties(
    VkPhysicalDevice physical_device,
    const char *layer_name,
    uint32_t *property_count,
    VkExtensionProperties *properties) noexcept
{
    assert(layer_name == nullptr);
    assert(physical_device != VK_NULL_HANDLE);
    static constexpr auto extensions = vulkan::get_extensions<vulkan::Extension_scope::Device>();
    return vulkan_enumerate_list_helper(
        property_count, properties, extensions.data(), extensions.size());
}
}

void vulkan_icd::print_exception(std::exception &e) noexcept
{
    std::cerr << "error: " << e.what() << std::endl;
}
}
