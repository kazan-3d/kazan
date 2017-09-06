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

#ifndef GENERATE_SPIRV_PARSER_PATCH_H_
#define GENERATE_SPIRV_PARSER_PATCH_H_

#include "ast.h"
#include <ostream>
#include <memory>

namespace kazan
{
namespace generate_spirv_parser
{
struct Ast_patch
{
    constexpr Ast_patch() noexcept
    {
    }
    virtual ~Ast_patch() = default;

protected:
    virtual bool apply(ast::Top_level &top_level) const = 0;

public:
    virtual const char *get_name() const noexcept = 0;
    void run(ast::Top_level &top_level, std::ostream *log_output = nullptr) const;
};

struct Ast_patches
{
    struct Add_image_operands_grad_parameter_names final : public Ast_patch
    {
        using Ast_patch::Ast_patch;

    protected:
        virtual bool apply(ast::Top_level &top_level) const override;

    public:
        virtual const char *get_name() const noexcept override;
    };
    static std::vector<const Ast_patch *> get_patches();
};
}
}

#endif /* GENERATE_SPIRV_PARSER_PATCH_H_ */
