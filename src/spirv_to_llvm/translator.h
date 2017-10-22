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
#ifndef SPIRV_TO_LLVM_TRANSLATOR_H_
#define SPIRV_TO_LLVM_TRANSLATOR_H_

#include "spirv/parser.h"
#include "util/enum.h"
#include "spirv_id.h"
#include "spirv_to_llvm.h"
#include "vulkan/api_objects.h"
#include "util/string_view.h"
#include <unordered_map>
#include <type_traits>

namespace kazan
{
namespace spirv_to_llvm
{
struct Translator
{
    struct Per_shader_state
    {
        Spirv_id_list id_list;
        std::unordered_map<spirv::Id, std::string> names;
        std::unordered_map<spirv::Id, std::unordered_map<spirv::Word, std::string>> member_names;
        std::unordered_multimap<spirv::Id, Spirv_decoration> decorations;
        explicit Per_shader_state(spirv::Word id_bound) : id_list(id_bound)
        {
        }
    };
    util::Enum_map<spirv::Execution_model, Per_shader_state> per_shader_states;
    Per_shader_state &get_per_shader_state(spirv::Execution_model execution_model) noexcept
    {
        auto iter = per_shader_states.find(execution_model);
        assert(iter != per_shader_states.end());
        return std::get<1>(*iter);
    }
    const Per_shader_state &get_per_shader_state(spirv::Execution_model execution_model) const
        noexcept
    {
        auto iter = per_shader_states.find(execution_model);
        assert(iter != per_shader_states.end());
        return std::get<1>(*iter);
    }
    template <typename T = Spirv_id>
    T *get_id_or_null(spirv::Execution_model execution_model, spirv::Id id) const noexcept
    {
        return get_per_shader_state(execution_model).id_list.get_or_null<T>(id);
    }
    template <typename T = Spirv_id>
    T &get_id(spirv::Execution_model execution_model, spirv::Id id) const noexcept
    {
        return get_per_shader_state(execution_model).id_list.get<T>(id);
    }
    void set_id(spirv::Execution_model execution_model,
                spirv::Id id,
                std::unique_ptr<Spirv_id> value) noexcept
    {
        get_per_shader_state(execution_model).id_list.set(id, std::move(value));
    }
    util::Enum_map<spirv::Execution_model, vulkan::Vulkan_shader_module *> shader_modules;
    explicit Translator(util::Enum_map<spirv::Execution_model, vulkan::Vulkan_shader_module *>
                            shader_modules) noexcept : shader_modules(std::move(shader_modules))
    {
    }
    util::string_view get_name(spirv::Execution_model execution_model,
                               spirv::Id id,
                               util::string_view default_name = {}) const
    {
        auto &map = get_per_shader_state(execution_model).names;
        auto iter = map.find(id);
        if(iter != map.end())
            return std::get<1>(*iter);
        return default_name;
    }
    std::pair<std::unordered_map<spirv::Word, std::string>::const_iterator,
              std::unordered_map<spirv::Word, std::string>::const_iterator>
        get_member_name_range(spirv::Execution_model execution_model, spirv::Id id) const
    {
        auto &map = get_per_shader_state(execution_model).member_names;
        auto iter = map.find(id);
        if(iter != map.end())
            return {std::get<1>(*iter).begin(), std::get<1>(*iter).end()};
        return {};
    }
    std::pair<std::unordered_multimap<spirv::Id, Spirv_decoration>::const_iterator,
              std::unordered_multimap<spirv::Id, Spirv_decoration>::const_iterator>
        get_decoration_range(spirv::Execution_model execution_model, spirv::Id id) const
    {
        return get_per_shader_state(execution_model).decorations.equal_range(id);
    }

private:
    template <typename Fn,
              typename... Args,
              typename = typename std::enable_if<std::is_void<decltype(
                  std::declval<Fn &>()(std::declval<Args>()...))>::value>::type>
    static bool for_each_helper(Fn &&fn, Args &&... args)
    {
        fn(std::forward<Args>(args)...);
        return true;
    }
    template <typename Fn, typename... Args>
    static typename std::
        enable_if<!std::is_void<decltype(std::declval<Fn &>()(std::declval<Args>()...))>::value,
                  bool>::type
        for_each_helper(Fn &&fn, Args &&... args)
    {
        return fn(std::forward<Args>(args)...);
    }

public:
    /// fn is the callback function; have fn return true or void to continue, false to break
    template <typename Fn>
    bool for_each_decoration(spirv::Execution_model execution_model, spirv::Id id, Fn &&fn)
    {
        std::unordered_multimap<spirv::Id, Spirv_decoration>::const_iterator start, finish;
        std::tie(start, finish) = get_decoration_range(execution_model, id);
        for(auto iter = start; iter != finish; ++iter)
            if(!for_each_helper(fn, std::get<1>(*iter)))
                return false;
        return true;
    }
};

class Parser_callbacks_implementation;
struct Spirv_location;

class Parser_callbacks_base : public spirv::Parser_callbacks
{
    friend class Parser_callbacks_implementation;

protected:
    Translator *translator{};
    spirv::Execution_model execution_model{};
    Per_shader_state *per_shader_state{};

private:
    void init(Translator *translator, spirv::Execution_model execution_model) noexcept
    {
        this->translator = translator;
        this->execution_model = execution_model;
    }

protected:
    template <typename T = Spirv_id>
    T *get_id_or_null(spirv::Id id) const noexcept
    {
        return translator->get_id_or_null<T>(execution_model, id);
    }
    template <typename T = Spirv_id>
    T *get_id(spirv::Id id) const noexcept
    {
        return translator->get_id<T>(execution_model, id);
    }
    void set_id(spirv::Id id, std::unique_ptr<Spirv_id> value) noexcept
    {
        translator->set_id(execution_model, id, std::move(value));
    }
    util::string_view get_name(spirv::Id id, util::string_view default_name = {})
    {
        return translator->get_name(execution_model, id, default_name);
    }
    std::pair<std::unordered_multimap<spirv::Id, Spirv_decoration>::const_iterator,
              std::unordered_multimap<spirv::Id, Spirv_decoration>::const_iterator>
        get_decoration_range(spirv::Id id) noexcept
    {
        return translator->get_decoration_range(execution_model, id);
    }
    /// fn is the callback function; have fn return true or void to continue, false to break
    template <typename Fn>
    bool for_each_decoration(spirv::Id id, Fn &&fn)
    {
        return translator->for_each_decoration(execution_model, id, fn);
    }

protected:
    virtual void clear_line_info_because_end_of_block() = 0;
    virtual Spirv_location get_location(std::size_t instruction_start_index) const noexcept = 0;
};

class Parser_header_callbacks : public virtual Parser_callbacks_base
{
public:
    virtual void handle_header(unsigned version_number_major,
                               unsigned version_number_minor,
                               spirv::Word generator_magic_number,
                               spirv::Word id_bound,
                               spirv::Word instruction_schema) override final;
};
}
}

#endif // SPIRV_TO_LLVM_TRANSLATOR_H_
