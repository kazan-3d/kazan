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
#include "orc_compile_stack.h"
#include <llvm/ExecutionEngine/ExecutionEngine.h>
#include <llvm/ExecutionEngine/RTDyldMemoryManager.h>
#include <llvm/ExecutionEngine/JITEventListener.h>
#include <llvm/ExecutionEngine/JITSymbol.h>
#include <llvm/ExecutionEngine/Orc/ObjectLinkingLayer.h>
#include <llvm/ExecutionEngine/Orc/IRCompileLayer.h>
#include <llvm/ExecutionEngine/Orc/IRTransformLayer.h>
#include <llvm/ExecutionEngine/Orc/CompileUtils.h>
#include <llvm/ExecutionEngine/Orc/LambdaResolver.h>
#include <llvm/Target/TargetMachine.h>
#include <llvm/Config/llvm-config.h>
#include <unordered_map>
#include <unordered_set>

#if LLVM_VERSION_MAJOR != 4 || LLVM_VERSION_MINOR != 0
#error Orc compile stack is not yet implemented for this version of LLVM
#endif

namespace vulkan_cpu
{
namespace llvm_wrapper
{
// implement the unwrap functions that aren't in public llvm headers
static llvm::TargetMachine *unwrap(::LLVMTargetMachineRef v) noexcept
{
    return reinterpret_cast<llvm::TargetMachine *>(v);
}
static ::LLVMTargetMachineRef wrap(llvm::TargetMachine *v) noexcept
{
    return reinterpret_cast<::LLVMTargetMachineRef>(v);
}

class Orc_compile_stack_implementation
{
    Orc_compile_stack_implementation(const Orc_compile_stack_implementation &) = delete;
    Orc_compile_stack_implementation(Orc_compile_stack_implementation &&) = delete;
    Orc_compile_stack_implementation &operator=(const Orc_compile_stack_implementation &) = delete;
    Orc_compile_stack_implementation &operator=(Orc_compile_stack_implementation &&) = delete;

private:
    typedef Orc_compile_stack::Symbol_resolver_callback Symbol_resolver_callback;
    typedef Orc_compile_stack::Module_handle Module_handle;

private:
    // implement a wrapper for llvm::orc::ObjectLinkingLayer
    // in order to tell GDB about the contained objects
    class My_object_linking_layer
    {
        My_object_linking_layer(const My_object_linking_layer &) = delete;
        My_object_linking_layer(My_object_linking_layer &&) = delete;
        My_object_linking_layer &operator=(const My_object_linking_layer &) = delete;
        My_object_linking_layer &operator=(My_object_linking_layer &&) = delete;

    private:
        struct Object_set_wrapper
        {
            typedef std::unique_ptr<llvm::object::OwningBinary<llvm::object::ObjectFile>>
                value_type;
            const value_type *objects;
            std::size_t object_count;
            explicit Object_set_wrapper(const std::vector<value_type> &objects) noexcept
                : objects(objects.data()),
                  object_count(objects.size())
            {
            }
            auto begin() const noexcept
            {
                return objects;
            }
            auto end() const noexcept
            {
                return objects + object_count;
            }
            std::size_t size() const noexcept
            {
                return object_count;
            }
            auto &operator[](std::size_t index) const noexcept
            {
                return objects[index];
            }
        };
        class On_loaded_functor
        {
            friend class My_object_linking_layer;

        private:
            My_object_linking_layer *my_object_linking_layer;
            explicit On_loaded_functor(My_object_linking_layer *my_object_linking_layer) noexcept
                : my_object_linking_layer(my_object_linking_layer)
            {
            }

        public:
            void operator()(
                llvm::orc::ObjectLinkingLayerBase::ObjSetHandleT,
                const Object_set_wrapper
                        &object_set,
                const std::vector<std::unique_ptr<llvm::RuntimeDyld::LoadedObjectInfo>>
                    &load_result)
            {
                assert(object_set.size() == load_result.size());
                for(std::size_t i = 0; i < object_set.size(); i++)
                    my_object_linking_layer->handle_loaded_object(*object_set[i]->getBinary(),
                                                                  *load_result[i]);
            }
        };

    private:
        Module_handle create_module_handle() noexcept
        {
            return next_module_handle++;
        }
        static std::vector<std::shared_ptr<llvm::JITEventListener>> make_jit_event_listener_list()
        {
            std::vector<std::shared_ptr<llvm::JITEventListener>> retval;
            auto static_deleter = [](llvm::JITEventListener *)
            {
            };
            if(auto *v = llvm::JITEventListener::createGDBRegistrationListener())
            {
                // createGDBRegistrationListener returns a static object
                retval.push_back(std::shared_ptr<llvm::JITEventListener>(v, static_deleter));
            }
            if(auto *v = llvm::JITEventListener::createIntelJITEventListener())
            {
                retval.push_back(std::shared_ptr<llvm::JITEventListener>(v));
            }
            if(auto *v = llvm::JITEventListener::createOProfileJITEventListener())
            {
                retval.push_back(std::shared_ptr<llvm::JITEventListener>(v));
            }
            return retval;
        }
        void handle_loaded_object(const llvm::object::ObjectFile &object,
                                  const llvm::RuntimeDyld::LoadedObjectInfo &load_info)
        {
            loaded_object_set.insert(&object);
            for(auto &jit_event_listener : jit_event_listener_list)
                jit_event_listener->NotifyObjectEmitted(object, load_info);
        }

    public:
        My_object_linking_layer()
            : jit_event_listener_list(make_jit_event_listener_list()),
              object_linking_layer(On_loaded_functor(this))
        {
        }
        ~My_object_linking_layer()
        {
            for(auto i = loaded_object_set.begin(); i != loaded_object_set.end();)
            {
                for(auto &jit_event_listener : jit_event_listener_list)
                    jit_event_listener->NotifyFreeingObject(**i);
                i = loaded_object_set.erase(i);
            }
        }
        typedef Module_handle ObjSetHandleT;
        llvm::JITSymbol findSymbol(const std::string &name, bool exported_symbols_only)
        {
            return object_linking_layer.findSymbol(name, exported_symbols_only);
        }
        template <typename Symbol_resolver_pointer>
        Module_handle addObjectSet(
            std::vector<std::unique_ptr<llvm::object::OwningBinary<llvm::object::ObjectFile>>>
                object_set,
            std::unique_ptr<llvm::SectionMemoryManager> memory_manager,
            Symbol_resolver_pointer symbol_resolver)
        {
            auto retval = create_module_handle();
            auto &handle = handle_map[retval];
            auto object_set_iter = object_sets.insert(object_sets.end(), std::move(object_set));
            handle = object_linking_layer.addObjectSet(
                Object_set_wrapper(*object_set_iter), std::move(memory_manager), std::move(symbol_resolver));
            return retval;
        }

    private:
        Module_handle next_module_handle = 1;
        std::vector<std::shared_ptr<llvm::JITEventListener>> jit_event_listener_list;
        llvm::orc::ObjectLinkingLayer<On_loaded_functor> object_linking_layer;
        std::unordered_map<Module_handle, decltype(object_linking_layer)::ObjSetHandleT> handle_map;
        std::list<std::vector<std::unique_ptr<llvm::object::OwningBinary<llvm::object::ObjectFile>>>> object_sets;
        std::unordered_multiset<const llvm::object::ObjectFile *> loaded_object_set;
    };
    typedef std::function<std::unique_ptr<llvm::Module>(std::unique_ptr<llvm::Module>)>
        Optimize_function;

private:
    Orc_compile_stack::Optimize_function optimize_function;
    std::unique_ptr<llvm::TargetMachine> target_machine;
    My_object_linking_layer object_linking_layer;
    llvm::orc::IRCompileLayer<decltype(object_linking_layer)> compile_layer;
    llvm::orc::IRTransformLayer<decltype(compile_layer), Optimize_function> optimize_layer;

public:
    explicit Orc_compile_stack_implementation(
        Target_machine target_machine_in, Orc_compile_stack::Optimize_function optimize_function)
        : optimize_function(std::move(optimize_function)),
          target_machine(unwrap(target_machine_in.release())),
          object_linking_layer(),
          compile_layer(object_linking_layer, llvm::orc::SimpleCompiler(*target_machine)),
          optimize_layer(compile_layer,
                         [this](std::unique_ptr<llvm::Module> module)
                         {
                             if(this->optimize_function)
                             {
                                 auto rewrapped_module = Module(llvm::wrap(module.release()));
                                 rewrapped_module = this->optimize_function(
                                     std::move(rewrapped_module), wrap(target_machine.get()));
                                 return std::unique_ptr<llvm::Module>(
                                     llvm::unwrap(rewrapped_module.release()));
                             }
                             return module;
                         })
    {
    }
    Module_handle add_eagerly_compiled_ir(Module module,
                                          Symbol_resolver_callback symbol_resolver_callback,
                                          void *symbol_resolver_user_data)
    {
        auto resolver = llvm::orc::createLambdaResolver(
            [this](const std::string &name)
            {
                return compile_layer.findSymbol(name, false);
            },
            [symbol_resolver_user_data, symbol_resolver_callback](const std::string &name)
            {
                return llvm::JITSymbol(symbol_resolver_callback(name, symbol_resolver_user_data),
                                       llvm::JITSymbolFlags::Exported);
            });
        std::vector<std::unique_ptr<llvm::Module>> module_set;
        module_set.reserve(1);
        module_set.push_back(std::unique_ptr<llvm::Module>(llvm::unwrap(module.release())));
        return optimize_layer.addModuleSet(std::move(module_set),
                                           std::make_unique<llvm::SectionMemoryManager>(),
                                           std::move(resolver));
    }
    std::uintptr_t get_symbol_address(const std::string &symbol_name)
    {
        auto symbol = compile_layer.findSymbol(symbol_name, true);
        if(symbol)
            return symbol.getAddress();
        return 0;
    }
};

void Orc_compile_stack_deleter::operator()(Orc_compile_stack_ref v) const noexcept
{
    delete v;
}

Orc_compile_stack Orc_compile_stack::create(Target_machine target_machine,
                                            Optimize_function optimize_function)
{
    return Orc_compile_stack(new Orc_compile_stack_implementation(std::move(target_machine),
                                                                  std::move(optimize_function)));
}

Orc_compile_stack::Module_handle Orc_compile_stack::add_eagerly_compiled_ir(
    Orc_compile_stack_ref orc_compile_stack,
    Module module,
    Symbol_resolver_callback symbol_resolver_callback,
    void *symbol_resolver_user_data)
{
    return orc_compile_stack->add_eagerly_compiled_ir(
        std::move(module), symbol_resolver_callback, symbol_resolver_user_data);
}

std::uintptr_t Orc_compile_stack::get_symbol_address(Orc_compile_stack_ref orc_compile_stack,
                                                     const std::string &symbol_name)
{
    return orc_compile_stack->get_symbol_address(symbol_name);
}
}
}
