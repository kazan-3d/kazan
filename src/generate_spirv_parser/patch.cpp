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
#include "patch.h"

namespace kazan
{
namespace generate_spirv_parser
{
void Ast_patch::run(ast::Top_level &top_level, std::ostream *log_output) const
{
    auto name = get_name();
    if(log_output)
        *log_output << "PATCH " << name << ": checking if applicable" << std::endl;
    if(apply(top_level))
    {
        if(log_output)
            *log_output << "PATCH " << name << ": applied" << std::endl;
    }
    else if(log_output)
        *log_output << "PATCH " << name << ": not applicable" << std::endl;
}

bool Ast_patches::Add_image_operands_grad_parameter_names::apply(ast::Top_level &top_level) const
{
    for(auto &operand_kind : top_level.operand_kinds.operand_kinds)
    {
        if(operand_kind.kind != "ImageOperands")
            continue;
        auto *enumerants =
            util::get_if<ast::Operand_kinds::Operand_kind::Enumerants>(&operand_kind.value);
        if(enumerants)
        {
            for(auto &enumerant : enumerants->enumerants)
            {
                if(enumerant.enumerant != "Grad")
                    continue;
                if(enumerant.parameters.parameters.size() != 2)
                    return false;
                auto &dx_param = enumerant.parameters.parameters[0];
                if(!dx_param.name.empty())
                    return false;
                auto &dy_param = enumerant.parameters.parameters[1];
                if(!dy_param.name.empty())
                    return false;
                dx_param.name = "dx";
                dy_param.name = "dy";
                return true;
            }
        }
        return false;
    }
    return false;
}

const char *Ast_patches::Add_image_operands_grad_parameter_names::get_name() const noexcept
{
    return "Add_image_operands_grad_parameter_names";
}

std::vector<const Ast_patch *> Ast_patches::get_patches()
{
    static const auto add_image_operands_grad_parameter_names =
        Add_image_operands_grad_parameter_names();
    return {
        &add_image_operands_grad_parameter_names,
    };
}
}
}
