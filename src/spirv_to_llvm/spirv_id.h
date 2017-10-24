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
#ifndef SPIRV_TO_LLVM_SPIRV_ID_H_
#define SPIRV_TO_LLVM_SPIRV_ID_H_

#include <cstdint>
#include <memory>
#include <vector>
#include <cassert>
#include "spirv/spirv.h"

namespace kazan
{
namespace spirv_to_llvm
{
class Spirv_id
{
public:
    const std::size_t defining_instruction_start_index;
    explicit Spirv_id(std::size_t defining_instruction_start_index) noexcept
        : defining_instruction_start_index(defining_instruction_start_index)
    {
    }
    virtual ~Spirv_id() = default;
};

class Spirv_id_list
{
private:
    std::vector<std::unique_ptr<Spirv_id>> id_list;
    spirv::Word id_bound;

public:
    explicit Spirv_id_list(spirv::Word id_bound) : id_list(), id_bound(id_bound)
    {
        assert(id_bound > 0);
        id_list.resize(id_bound - 1);
    }
    std::unique_ptr<Spirv_id> &operator[](spirv::Id id) noexcept
    {
        assert(id > 0 && id < id_bound);
        return id_list[id - 1];
    }
    const std::unique_ptr<Spirv_id> &operator[](spirv::Id id) const noexcept
    {
        assert(id > 0 && id < id_bound);
        return id_list[id - 1];
    }
    template <typename T = Spirv_id>
    typename std::enable_if<std::is_base_of<Spirv_id, T>::value, T>::type *get_or_null(
        spirv::Id id) const noexcept
    {
        auto *base = operator[](id).get();
        if(!base)
            return nullptr;
        auto *retval = dynamic_cast<T *>(base);
        assert(retval && "SPIR-V id is of improper type");
        return retval;
    }
    template <typename T = Spirv_id>
    typename std::enable_if<std::is_base_of<Spirv_id, T>::value, T>::type &get(spirv::Id id) const
        noexcept
    {
        auto *retval = get_or_null<T>(id);
        assert(retval && "SPIR-V id is undefined");
        return *retval;
    }
    bool is_defined_at(spirv::Id id, std::size_t defining_instruction_start_index) const noexcept
    {
        if(auto *v = operator[](id).get())
            return v->defining_instruction_start_index == defining_instruction_start_index;
        return false;
    }
    void set(spirv::Id id, std::unique_ptr<Spirv_id> value) noexcept
    {
        auto &v = operator[](id);
        assert(!v && "SPIR-V id is already defined");
        assert(value);
        v = std::move(value);
    }
};
}
}

#endif // SPIRV_TO_LLVM_SPIRV_ID_H_
