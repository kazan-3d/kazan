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
#include "spirv_to_llvm_implementation.h"

namespace vulkan_cpu
{
namespace spirv_to_llvm
{
using namespace spirv;

void Spirv_to_llvm::handle_instruction_op_nop([[gnu::unused]] Op_nop instruction,
                                              [[gnu::unused]] std::size_t instruction_start_index)
{
}

void Spirv_to_llvm::handle_instruction_op_undef(Op_undef instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_source_continued(
    [[gnu::unused]] Op_source_continued instruction,
    [[gnu::unused]] std::size_t instruction_start_index)
{
}

void Spirv_to_llvm::handle_instruction_op_source(
    Op_source instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    if(stage == util::Enum_traits<Stage>::values[0] && instruction.file)
    {
        std::string filename(
            get_id_state(*instruction.file).op_string.value_or(Op_string_state()).value);
        ::LLVMSetModuleIdentifier(module.get(), filename.data(), filename.size());
    }
}

void Spirv_to_llvm::handle_instruction_op_source_extension(
    [[gnu::unused]] Op_source_extension instruction,
    [[gnu::unused]] std::size_t instruction_start_index)
{
}

void Spirv_to_llvm::handle_instruction_op_name(Op_name instruction,
                                               [[gnu::unused]] std::size_t instruction_start_index)
{
    if(stage == util::Enum_traits<Stage>::values[0])
        get_id_state(instruction.target).name = Name{std::string(instruction.name)};
}

void Spirv_to_llvm::handle_instruction_op_member_name(
    Op_member_name instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    if(stage == util::Enum_traits<Stage>::values[0])
    {
        auto &state = get_id_state(instruction.type);
        state.member_names.push_back(std::move(instruction));
    }
}

void Spirv_to_llvm::handle_instruction_op_string(
    Op_string instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    if(stage == util::Enum_traits<Stage>::values[0])
        get_id_state(instruction.result).op_string = Op_string_state{instruction.string};
}

void Spirv_to_llvm::handle_instruction_op_line(Op_line instruction,
                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_extension(Op_extension instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_ext_inst_import(Op_ext_inst_import instruction,
                                                          std::size_t instruction_start_index)
{
    if(stage == util::Enum_traits<Stage>::values[0])
    {
        get_id_state(instruction.result).op_ext_inst_import = Op_ext_inst_import_state{};
        for(auto instruction_set : util::Enum_traits<Extension_instruction_set>::values)
        {
            if(instruction_set == Extension_instruction_set::unknown)
                continue;
            if(instruction.name == get_enumerant_name(instruction_set))
                return;
        }
        throw Parser_error(instruction_start_index,
                           instruction_start_index,
                           "unknown instruction set: \"" + std::string(instruction.name) + "\"");
    }
}

void Spirv_to_llvm::handle_instruction_op_ext_inst(Op_ext_inst instruction,
                                                   std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_memory_model(Op_memory_model instruction,
                                                       std::size_t instruction_start_index)
{
    if(instruction.addressing_model != Addressing_model::logical)
        throw Parser_error(instruction_start_index,
                           instruction_start_index,
                           "unsupported addressing model: "
                               + std::string(get_enumerant_name(instruction.addressing_model)));
    switch(instruction.memory_model)
    {
    case Memory_model::simple:
    case Memory_model::glsl450:
        break;
    default:
        throw Parser_error(instruction_start_index,
                           instruction_start_index,
                           "unsupported memory model: "
                               + std::string(get_enumerant_name(instruction.memory_model)));
    }
}

void Spirv_to_llvm::handle_instruction_op_entry_point(Op_entry_point instruction,
                                                      std::size_t instruction_start_index)
{
    if(stage == util::Enum_traits<Stage>::values[0])
    {
        if(entry_point_state_pointer)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "invalid location for OpEntryPoint");
        auto &state = get_id_state(instruction.entry_point);
        state.op_entry_points.push_back(
            Op_entry_point_state{std::move(instruction), instruction_start_index});
    }
}

void Spirv_to_llvm::handle_instruction_op_execution_mode(Op_execution_mode instruction,
                                                         std::size_t instruction_start_index)
{
    if(stage == util::Enum_traits<Stage>::values[0])
    {
        auto &state = get_id_state(instruction.entry_point);
        if(state.op_entry_points.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "entry point not defined in OpExecutionMode");
        state.op_entry_points.back().execution_modes.push_back(std::move(instruction.mode));
    }
}

void Spirv_to_llvm::handle_instruction_op_capability(Op_capability instruction,
                                                     std::size_t instruction_start_index)
{
    if(stage == util::Enum_traits<Stage>::values[0])
    {
        util::Enum_set<Capability> work_list{instruction.capability};
        while(!work_list.empty())
        {
            auto capability = *work_list.begin();
            work_list.erase(capability);
            if(std::get<1>(enabled_capabilities.insert(capability)))
            {
                auto additional_capabilities = get_directly_required_capabilities(capability);
                work_list.insert(additional_capabilities.begin(), additional_capabilities.end());
            }
        }
        constexpr util::Enum_set<Capability> implemented_capabilities{
            Capability::matrix,
            Capability::shader,
            Capability::input_attachment,
            Capability::sampled1d,
            Capability::image1d,
            Capability::sampled_buffer,
            Capability::image_buffer,
            Capability::image_query,
            Capability::derivative_control,
            Capability::int64,
        };
        for(auto capability : enabled_capabilities)
        {
            if(implemented_capabilities.count(capability) == 0)
                throw Parser_error(
                    instruction_start_index,
                    instruction_start_index,
                    "capability not implemented: " + std::string(get_enumerant_name(capability)));
        }
    }
}

void Spirv_to_llvm::handle_instruction_op_type_void(
    Op_type_void instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        state.type = std::make_shared<Simple_type_descriptor>(
            state.decorations, LLVM_type_and_alignment(::LLVMVoidTypeInContext(context), 1));
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_bool(Op_type_bool instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_int(Op_type_int instruction,
                                                   std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        switch(instruction.width)
        {
        case 8:
        case 16:
        case 32:
        case 64:
        {
            auto type = ::LLVMIntTypeInContext(context, instruction.width);
            state.type = std::make_shared<Simple_type_descriptor>(
                state.decorations,
                LLVM_type_and_alignment(type, ::LLVMPreferredAlignmentOfType(target_data, type)));
            break;
        }
        default:
            throw Parser_error(
                instruction_start_index, instruction_start_index, "invalid int width");
        }
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_float(Op_type_float instruction,
                                                     std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        ::LLVMTypeRef type = nullptr;
        switch(instruction.width)
        {
        case 16:
            type = ::LLVMHalfTypeInContext(context);
            break;
        case 32:
            type = ::LLVMFloatTypeInContext(context);
            break;
        case 64:
            type = ::LLVMDoubleTypeInContext(context);
            break;
        default:
            throw Parser_error(
                instruction_start_index, instruction_start_index, "invalid float width");
        }
        state.type = std::make_shared<Simple_type_descriptor>(
            state.decorations,
            LLVM_type_and_alignment(type, ::LLVMPreferredAlignmentOfType(target_data, type)));
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_vector(Op_type_vector instruction,
                                                      std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        state.type = std::make_shared<Vector_type_descriptor>(
            state.decorations,
            get_type<Simple_type_descriptor>(instruction.component_type, instruction_start_index),
            instruction.component_count,
            target_data);
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_matrix(Op_type_matrix instruction,
                                                      std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        state.type = std::make_shared<Matrix_type_descriptor>(
            state.decorations,
            get_type<Vector_type_descriptor>(instruction.column_type, instruction_start_index),
            instruction.column_count,
            target_data);
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_image(Op_type_image instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_sampler(Op_type_sampler instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_sampled_image(Op_type_sampled_image instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_array(Op_type_array instruction,
                                                     std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto length = get_unsigned_integer_constant(instruction.length, instruction_start_index);
        if(length <= 0)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "OpTypeArray length must be a positive constant integer");
        state.type = std::make_shared<Array_type_descriptor>(
            state.decorations,
            get_type(instruction.element_type, instruction_start_index),
            length,
            instruction_start_index);
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_runtime_array(Op_type_runtime_array instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_struct(Op_type_struct instruction,
                                                      std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        std::vector<Struct_type_descriptor::Member> members;
        members.reserve(instruction.member_0_type_member_1_type.size());
        for(auto &member_id : instruction.member_0_type_member_1_type)
            members.push_back(
                Struct_type_descriptor::Member({}, get_type(member_id, instruction_start_index)));
        for(auto &decoration : state.member_decorations)
        {
            if(decoration.member >= members.size())
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "member decoration's member index is out of range");
            auto &member = members[decoration.member];
            member.decorations.push_back(decoration.decoration);
        }
        state.type = std::make_shared<Struct_type_descriptor>(
            state.decorations,
            context,
            ::LLVMGetModuleDataLayout(module.get()),
            get_prefixed_name(get_name(instruction.result), false).c_str(),
            instruction_start_index,
            std::move(members));
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_opaque(Op_type_opaque instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_pointer(Op_type_pointer instruction,
                                                       std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        if(!state.type)
        {
            state.type = std::make_shared<Pointer_type_descriptor>(
                state.decorations,
                get_type(instruction.type, instruction_start_index),
                instruction_start_index,
                target_data);
        }
        else if(auto *pointer_type = dynamic_cast<Pointer_type_descriptor *>(state.type.get()))
        {
            if(pointer_type->get_base_type())
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "result type is not a pointer forward declaration");
            pointer_type->set_base_type(get_type(instruction.type, instruction_start_index));
        }
        else
        {
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "result type is not a pointer forward declaration");
        }
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_function(Op_type_function instruction,
                                                        std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        std::vector<std::shared_ptr<Type_descriptor>> args;
        args.reserve(implicit_function_arguments.size()
                     + instruction.parameter_0_type_parameter_1_type.size());
        for(auto &arg : implicit_function_arguments)
            args.push_back(arg);
        bool return_type_is_void = false;
        auto return_type = get_type(instruction.return_type, instruction_start_index);
        if(auto *simple_return_type = dynamic_cast<Simple_type_descriptor *>(return_type.get()))
            if(simple_return_type->get_or_make_type().type == ::LLVMVoidTypeInContext(context))
                return_type_is_void = true;
        bool valid_for_entry_point =
            instruction.parameter_0_type_parameter_1_type.empty() && return_type_is_void;
        for(Id_ref type : instruction.parameter_0_type_parameter_1_type)
        {
            args.push_back(get_type(type, instruction_start_index));
        }
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        constexpr bool is_var_arg = false;
        state.type = std::make_shared<Function_type_descriptor>(
            state.decorations,
            get_type(instruction.return_type, instruction_start_index),
            std::move(args),
            instruction_start_index,
            target_data,
            valid_for_entry_point,
            is_var_arg);
        break;
    }
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_type_event(Op_type_event instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_device_event(Op_type_device_event instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_reserve_id(Op_type_reserve_id instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_queue(Op_type_queue instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_pipe(Op_type_pipe instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_forward_pointer(Op_type_forward_pointer instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_constant_true(Op_constant_true instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_constant_false(Op_constant_false instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_constant(Op_constant instruction,
                                                   std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto type = get_type(instruction.result_type, instruction_start_index);
        if(auto *simple_type = dynamic_cast<Simple_type_descriptor *>(type.get()))
        {
            auto llvm_type = simple_type->get_or_make_type();
            switch(::LLVMGetTypeKind(llvm_type.type))
            {
            case LLVMFloatTypeKind:
            {
                if(instruction.value.size() != 1)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "OpConstant immediate value is wrong size for type float32");
                state.constant = std::make_shared<Simple_constant_descriptor>(
                    type,
                    ::LLVMConstBitCast(
                        ::LLVMConstInt(
                            ::LLVMInt32TypeInContext(context), instruction.value[0], false),
                        llvm_type.type));
                break;
            }
            case LLVMIntegerTypeKind:
            {
                switch(::LLVMGetIntTypeWidth(llvm_type.type))
                {
                case 16:
                {
                    if(instruction.value.size() != 1)
                        throw Parser_error(
                            instruction_start_index,
                            instruction_start_index,
                            "OpConstant immediate value is wrong size for type int16");
                    state.constant = std::make_shared<Simple_constant_descriptor>(
                        type, ::LLVMConstInt(llvm_type.type, instruction.value[0], false));
                    break;
                }
                case 32:
                {
                    if(instruction.value.size() != 1)
                        throw Parser_error(
                            instruction_start_index,
                            instruction_start_index,
                            "OpConstant immediate value is wrong size for type int32");
                    state.constant = std::make_shared<Simple_constant_descriptor>(
                        type, ::LLVMConstInt(llvm_type.type, instruction.value[0], false));
                    break;
                }
                case 64:
                {
                    if(instruction.value.size() != 2)
                        throw Parser_error(
                            instruction_start_index,
                            instruction_start_index,
                            "OpConstant immediate value is wrong size for type int64");
                    state.constant = std::make_shared<Simple_constant_descriptor>(
                        type,
                        ::LLVMConstInt(llvm_type.type,
                                       (static_cast<std::uint64_t>(instruction.value[1]) << 32)
                                           | instruction.value[0],
                                       false));
                    break;
                }
                case 1: // bool
                default:
                    throw Parser_error(
                        instruction_start_index,
                        instruction_start_index,
                        "unimplemented simple type for OpConstant: "
                            + std::string(llvm_wrapper::print_type_to_string(llvm_type.type)));
                }
                break;
            }
            case LLVMDoubleTypeKind:
            {
                if(instruction.value.size() != 2)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "OpConstant immediate value is wrong size for type float64");
                state.constant = std::make_shared<Simple_constant_descriptor>(
                    type,
                    ::LLVMConstBitCast(
                        ::LLVMConstInt(::LLVMInt64TypeInContext(context),
                                       (static_cast<std::uint64_t>(instruction.value[1]) << 32)
                                           | instruction.value[0],
                                       false),
                        llvm_type.type));
                break;
            }
            case LLVMHalfTypeKind:
            {
                if(instruction.value.size() != 1)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "OpConstant immediate value is wrong size for type float16");
                state.constant = std::make_shared<Simple_constant_descriptor>(
                    type,
                    ::LLVMConstBitCast(
                        ::LLVMConstInt(
                            ::LLVMInt16TypeInContext(context), instruction.value[0], false),
                        llvm_type.type));
                break;
            }
            default:
            {
                throw Parser_error(
                    instruction_start_index,
                    instruction_start_index,
                    "unimplemented simple type for OpConstant: "
                        + std::string(llvm_wrapper::print_type_to_string(llvm_type.type)));
            }
            }
        }
        else
        {
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "unimplemented type for OpConstant");
        }
        break;
    }
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        state.value = Value(state.constant->get_or_make_value(),
                            get_type(instruction.result_type, instruction_start_index));
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_constant_composite(Op_constant_composite instruction,
                                                             std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto type = get_type(instruction.result_type, instruction_start_index);
        if(auto *vector_type = dynamic_cast<Vector_type_descriptor *>(type.get()))
        {
            if(instruction.constituents.size() != vector_type->get_element_count())
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "wrong number of constituents for type");
            std::vector<::LLVMValueRef> constituents;
            constituents.reserve(instruction.constituents.size());
            for(Id_ref constituent : instruction.constituents)
            {
                auto &constituent_state = get_id_state(constituent);
                if(!constituent_state.constant)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "constituent must be a constant or OpUndef");
                constituents.push_back(constituent_state.constant->get_or_make_value());
            }
            state.constant = std::make_shared<Simple_constant_descriptor>(
                type, ::LLVMConstVector(constituents.data(), constituents.size()));
            break;
        }
        else if(auto *array_type = dynamic_cast<Array_type_descriptor *>(type.get()))
        {
            if(instruction.constituents.size() != array_type->get_element_count())
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "wrong number of constituents for type");
            std::vector<::LLVMValueRef> constituents;
            constituents.reserve(instruction.constituents.size());
            for(Id_ref constituent : instruction.constituents)
            {
                auto &constituent_state = get_id_state(constituent);
                if(!constituent_state.constant)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "constituent must be a constant or OpUndef");
                constituents.push_back(constituent_state.constant->get_or_make_value());
            }
            state.constant = std::make_shared<Simple_constant_descriptor>(
                type,
                ::LLVMConstArray(array_type->get_element_type()->get_or_make_type().type,
                                 constituents.data(),
                                 constituents.size()));
            break;
        }
        else
        {
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "unimplemented type for OpConstantComposite");
        }
        break;
    }
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        state.value = Value(state.constant->get_or_make_value(),
                            get_type(instruction.result_type, instruction_start_index));
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_constant_sampler(Op_constant_sampler instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_constant_null(Op_constant_null instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_spec_constant_true(Op_spec_constant_true instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_spec_constant_false(Op_spec_constant_false instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_spec_constant(Op_spec_constant instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_spec_constant_composite(
    Op_spec_constant_composite instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_spec_constant_op(Op_spec_constant_op instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_function(Op_function instruction,
                                                   std::size_t instruction_start_index)
{
    if(current_function_id)
        throw Parser_error(instruction_start_index,
                           instruction_start_index,
                           "missing OpFunctionEnd before starting a new function");
    current_function_id = instruction.result;
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(current_function_id);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto function_type =
            get_type<Function_type_descriptor>(instruction.function_type, instruction_start_index);
        auto function_name = get_name(current_function_id);
        if(function_name.empty() && state.op_entry_points.size() == 1)
            function_name = std::string(state.op_entry_points[0].entry_point.name);
        if(!state.op_entry_points.empty() && !function_type->is_valid_for_entry_point())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "invalid function type for entry point");
        function_name = get_or_make_prefixed_name(std::move(function_name), false);
        auto function = ::LLVMAddFunction(
            module.get(), function_name.c_str(), function_type->get_or_make_type().type);
        llvm_wrapper::Module::set_function_target_machine(function, target_machine);
        state.function = Function_state(function_type, function, std::move(function_name));
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_function_parameter(Op_function_parameter instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_function_end([[gnu::unused]] Op_function_end instruction,
                                                       std::size_t instruction_start_index)
{
    if(!current_function_id)
        throw Parser_error(instruction_start_index,
                           instruction_start_index,
                           "OpFunctionEnd without matching OpFunction");
    current_function_id = 0;
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
        break;
    }
}

void Spirv_to_llvm::handle_instruction_op_function_call(Op_function_call instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_variable(Op_variable instruction,
                                                   std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
    {
        auto &state = get_id_state(instruction.result);
        bool check_decorations = true;
        [&]()
        {
            switch(instruction.storage_class)
            {
            case Storage_class::uniform_constant:
#warning finish implementing Storage_class::uniform_constant
                break;
            case Storage_class::input:
            {
                if(instruction.initializer)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "shader input variable initializers are not implemented");
                auto type = get_type<Pointer_type_descriptor>(instruction.result_type,
                                                              instruction_start_index)
                                ->get_base_type();
                state.variable =
                    Input_variable_state{type,
                                         inputs_struct->add_member(Struct_type_descriptor::Member(
                                             state.decorations, type))};
                check_decorations = false;
                return;
            }
            case Storage_class::uniform:
#warning finish implementing Storage_class::uniform
                break;
            case Storage_class::output:
            {
                if(instruction.initializer)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "shader output variable initializers are not implemented");
                auto type = get_type<Pointer_type_descriptor>(instruction.result_type,
                                                              instruction_start_index)
                                ->get_base_type();
                state.variable =
                    Output_variable_state{type,
                                          outputs_struct->add_member(Struct_type_descriptor::Member(
                                              state.decorations, type))};
                check_decorations = false;
                return;
            }
            case Storage_class::workgroup:
#warning finish implementing Storage_class::workgroup
                break;
            case Storage_class::cross_workgroup:
#warning finish implementing Storage_class::cross_workgroup
                break;
            case Storage_class::private_:
#warning finish implementing Storage_class::private_
                break;
            case Storage_class::function:
            {
                if(!current_function_id)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "function-local variable must be inside function");
                return;
            }
            case Storage_class::generic:
#warning finish implementing Storage_class::generic
                break;
            case Storage_class::push_constant:
#warning finish implementing Storage_class::push_constant
                break;
            case Storage_class::atomic_counter:
#warning finish implementing Storage_class::atomic_counter
                break;
            case Storage_class::image:
#warning finish implementing Storage_class::image
                break;
            case Storage_class::storage_buffer:
#warning finish implementing Storage_class::storage_buffer
                break;
            }
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "unimplemented OpVariable storage class: "
                                   + std::string(get_enumerant_name(instruction.storage_class)));
        }();
        if(check_decorations)
        {
            for(auto &decoration : state.decorations)
            {
                switch(decoration.value)
                {
                case Decoration::relaxed_precision:
#warning finish implementing Decoration::relaxed_precision
                    break;
                case Decoration::spec_id:
#warning finish implementing Decoration::spec_id
                    break;
                case Decoration::block:
#warning finish implementing Decoration::block
                    break;
                case Decoration::buffer_block:
#warning finish implementing Decoration::buffer_block
                    break;
                case Decoration::row_major:
#warning finish implementing Decoration::row_major
                    break;
                case Decoration::col_major:
#warning finish implementing Decoration::col_major
                    break;
                case Decoration::array_stride:
#warning finish implementing Decoration::array_stride
                    break;
                case Decoration::matrix_stride:
#warning finish implementing Decoration::matrix_stride
                    break;
                case Decoration::glsl_shared:
#warning finish implementing Decoration::glsl_shared
                    break;
                case Decoration::glsl_packed:
#warning finish implementing Decoration::glsl_packed
                    break;
                case Decoration::c_packed:
#warning finish implementing Decoration::c_packed
                    break;
                case Decoration::built_in:
#warning finish implementing Decoration::built_in
                    break;
                case Decoration::no_perspective:
#warning finish implementing Decoration::no_perspective
                    break;
                case Decoration::flat:
#warning finish implementing Decoration::flat
                    break;
                case Decoration::patch:
#warning finish implementing Decoration::patch
                    break;
                case Decoration::centroid:
#warning finish implementing Decoration::centroid
                    break;
                case Decoration::sample:
#warning finish implementing Decoration::sample
                    break;
                case Decoration::invariant:
#warning finish implementing Decoration::invariant
                    break;
                case Decoration::restrict:
#warning finish implementing Decoration::restrict
                    break;
                case Decoration::aliased:
#warning finish implementing Decoration::aliased
                    break;
                case Decoration::volatile_:
#warning finish implementing Decoration::volatile_
                    break;
                case Decoration::constant:
#warning finish implementing Decoration::constant
                    break;
                case Decoration::coherent:
#warning finish implementing Decoration::coherent
                    break;
                case Decoration::non_writable:
#warning finish implementing Decoration::non_writable
                    break;
                case Decoration::non_readable:
#warning finish implementing Decoration::non_readable
                    break;
                case Decoration::uniform:
#warning finish implementing Decoration::uniform
                    break;
                case Decoration::saturated_conversion:
#warning finish implementing Decoration::saturated_conversion
                    break;
                case Decoration::stream:
#warning finish implementing Decoration::stream
                    break;
                case Decoration::location:
#warning finish implementing Decoration::location
                    break;
                case Decoration::component:
#warning finish implementing Decoration::component
                    break;
                case Decoration::index:
#warning finish implementing Decoration::index
                    break;
                case Decoration::binding:
#warning finish implementing Decoration::binding
                    break;
                case Decoration::descriptor_set:
#warning finish implementing Decoration::descriptor_set
                    break;
                case Decoration::offset:
#warning finish implementing Decoration::offset
                    break;
                case Decoration::xfb_buffer:
#warning finish implementing Decoration::xfb_buffer
                    break;
                case Decoration::xfb_stride:
#warning finish implementing Decoration::xfb_stride
                    break;
                case Decoration::func_param_attr:
#warning finish implementing Decoration::func_param_attr
                    break;
                case Decoration::fp_rounding_mode:
#warning finish implementing Decoration::fp_rounding_mode
                    break;
                case Decoration::fp_fast_math_mode:
#warning finish implementing Decoration::fp_fast_math_mode
                    break;
                case Decoration::linkage_attributes:
#warning finish implementing Decoration::linkage_attributes
                    break;
                case Decoration::no_contraction:
#warning finish implementing Decoration::no_contraction
                    break;
                case Decoration::input_attachment_index:
#warning finish implementing Decoration::input_attachment_index
                    break;
                case Decoration::alignment:
#warning finish implementing Decoration::alignment
                    break;
                case Decoration::max_byte_offset:
#warning finish implementing Decoration::max_byte_offset
                    break;
                case Decoration::alignment_id:
#warning finish implementing Decoration::alignment_id
                    break;
                case Decoration::max_byte_offset_id:
#warning finish implementing Decoration::max_byte_offset_id
                    break;
                case Decoration::override_coverage_nv:
#warning finish implementing Decoration::override_coverage_nv
                    break;
                case Decoration::passthrough_nv:
#warning finish implementing Decoration::passthrough_nv
                    break;
                case Decoration::viewport_relative_nv:
#warning finish implementing Decoration::viewport_relative_nv
                    break;
                case Decoration::secondary_viewport_relative_nv:
#warning finish implementing Decoration::secondary_viewport_relative_nv
                    break;
                }
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "unimplemented decoration on OpVariable: "
                                       + std::string(get_enumerant_name(decoration.value)));
            }
        }
        break;
    }
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        auto &entry_point_state = get_entry_point_state();
        bool is_part_of_entry_point_interface = false;
        for(Id_ref id : entry_point_state.entry_point.interface)
        {
            if(instruction.result == id)
            {
                is_part_of_entry_point_interface = true;
                break;
            }
        }
        switch(instruction.storage_class)
        {
        case Storage_class::uniform_constant:
#warning finish implementing Storage_class::uniform_constant
            break;
        case Storage_class::input:
        {
            if(instruction.initializer)
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "shader input variable initializers are not implemented");
            if(!is_part_of_entry_point_interface)
            {
                auto type = get_type(instruction.result_type, instruction_start_index);
                state.value = Value(::LLVMGetUndef(type->get_or_make_type().type), type);
                return;
            }
            auto set_value_fn = [this, instruction, &state, instruction_start_index]()
            {
                auto &variable = util::get<Input_variable_state>(state.variable);
                state.value = Value(
                    ::LLVMBuildStructGEP(
                        builder.get(),
                        get_id_state(current_function_id).function->entry_block->inputs_struct,
                        inputs_struct->get_members(true)[variable.member_index].llvm_member_index,
                        get_name(instruction.result).c_str()),
                    get_type(instruction.result_type, instruction_start_index));
            };
            if(current_function_id)
                set_value_fn();
            else
                function_entry_block_handlers.push_back(set_value_fn);
            return;
        }
        case Storage_class::uniform:
#warning finish implementing Storage_class::uniform
            break;
        case Storage_class::output:
        {
            if(instruction.initializer)
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "shader output variable initializers are not implemented");
            if(!is_part_of_entry_point_interface)
            {
                auto type = get_type(instruction.result_type, instruction_start_index);
                state.value = Value(::LLVMGetUndef(type->get_or_make_type().type), type);
                return;
            }
            auto set_value_fn = [this, instruction, &state, instruction_start_index]()
            {
                auto &variable = util::get<Output_variable_state>(state.variable);
                state.value = Value(
                    ::LLVMBuildStructGEP(
                        builder.get(),
                        get_id_state(current_function_id).function->entry_block->outputs_struct,
                        outputs_struct->get_members(true)[variable.member_index].llvm_member_index,
                        get_name(instruction.result).c_str()),
                    get_type(instruction.result_type, instruction_start_index));
            };
            if(current_function_id)
                set_value_fn();
            else
                function_entry_block_handlers.push_back(set_value_fn);
            return;
        }
        case Storage_class::workgroup:
#warning finish implementing Storage_class::workgroup
            break;
        case Storage_class::cross_workgroup:
#warning finish implementing Storage_class::cross_workgroup
            break;
        case Storage_class::private_:
#warning finish implementing Storage_class::private_
            break;
        case Storage_class::function:
        {
            if(!current_function_id)
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "function-local variable must be inside function");
            auto &function = get_id_state(current_function_id).function.value();
            if(!function.entry_block
               || function.entry_block->entry_block != get_or_make_label(current_basic_block_id))
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "function-local variable must be inside initial basic block");
            auto type =
                get_type<Pointer_type_descriptor>(instruction.result_type, instruction_start_index);
            state.value = Value(::LLVMBuildAlloca(builder.get(),
                                                  type->get_base_type()->get_or_make_type().type,
                                                  get_name(instruction.result).c_str()),
                                type);
            ::LLVMSetAlignment(state.value->value,
                               type->get_base_type()->get_or_make_type().alignment);
            return;
        }
        case Storage_class::generic:
#warning finish implementing Storage_class::generic
            break;
        case Storage_class::push_constant:
#warning finish implementing Storage_class::push_constant
            break;
        case Storage_class::atomic_counter:
#warning finish implementing Storage_class::atomic_counter
            break;
        case Storage_class::image:
#warning finish implementing Storage_class::image
            break;
        case Storage_class::storage_buffer:
#warning finish implementing Storage_class::storage_buffer
            break;
        }
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_image_texel_pointer(Op_image_texel_pointer instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_load(Op_load instruction,
                                               std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto memory_access = instruction.memory_access.value_or(
            Memory_access_with_parameters(Memory_access::none, {}));
        if((memory_access.value & Memory_access::volatile_) == Memory_access::volatile_)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "OpLoad volatile not implemented");
        if((memory_access.value & Memory_access::aligned) == Memory_access::aligned)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "OpLoad alignment not implemented");
        if((memory_access.value & Memory_access::nontemporal) == Memory_access::nontemporal)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "OpLoad nontemporal not implemented");
        state.value = Value(::LLVMBuildLoad(builder.get(),
                                            get_id_state(instruction.pointer).value.value().value,
                                            get_name(instruction.result).c_str()),
                            get_type(instruction.result_type, instruction_start_index));
        ::LLVMSetAlignment(state.value->value, state.value->type->get_or_make_type().alignment);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_store(Op_store instruction,
                                                std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto memory_access = instruction.memory_access.value_or(
            Memory_access_with_parameters(Memory_access::none, {}));
        if((memory_access.value & Memory_access::volatile_) == Memory_access::volatile_)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "OpStore volatile not implemented");
        if((memory_access.value & Memory_access::aligned) == Memory_access::aligned)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "OpStore alignment not implemented");
        if((memory_access.value & Memory_access::nontemporal) == Memory_access::nontemporal)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "OpStore nontemporal not implemented");
        auto &object_value = get_id_state(instruction.object).value.value();
        auto &pointer_value = get_id_state(instruction.pointer).value.value();
        ::LLVMSetAlignment(::LLVMBuildStore(builder.get(), object_value.value, pointer_value.value),
                           object_value.type->get_or_make_type().alignment);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_copy_memory(Op_copy_memory instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_copy_memory_sized(Op_copy_memory_sized instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_access_chain(Op_access_chain instruction,
                                                       std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto base = get_id_state(instruction.base).value.value();
        std::string name = get_name(instruction.result);
        std::vector<::LLVMValueRef> llvm_indexes;
        llvm_indexes.reserve(instruction.indexes.size() + 1);
        auto *base_pointer_type = dynamic_cast<const Pointer_type_descriptor *>(base.type.get());
        if(!base_pointer_type)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "base type is not a pointer for OpAccessChain");
        llvm_indexes.push_back(::LLVMConstInt(::LLVMInt32TypeInContext(context), 0, false));
        auto current_type = base_pointer_type->get_base_type();
        for(std::size_t i = 0; i < instruction.indexes.size(); i++)
        {
            Id index = instruction.indexes[i];
            struct Visitor
            {
                std::size_t instruction_start_index;
                std::shared_ptr<Type_descriptor> &current_type;
                std::vector<::LLVMValueRef> &llvm_indexes;
                Id index;
                Spirv_to_llvm *this_;
                void operator()(Simple_type_descriptor &)
                {
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "invalid composite type for OpAccessChain");
                }
                void operator()(Vector_type_descriptor &type)
                {
                    auto &index_value = this_->get_id_state(index).value.value();
                    llvm_indexes.push_back(index_value.value);
                    current_type = type.get_element_type();
                }
                void operator()(Matrix_type_descriptor &)
                {
#warning finish
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "unimplemented composite type for OpAccessChain");
                }
                void operator()(Array_type_descriptor &type)
                {
                    auto &index_value = this_->get_id_state(index).value.value();
                    llvm_indexes.push_back(index_value.value);
                    current_type = type.get_element_type();
                }
                void operator()(Pointer_type_descriptor &)
                {
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "invalid composite type for OpAccessChain");
                }
                void operator()(Function_type_descriptor &)
                {
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "invalid composite type for OpAccessChain");
                }
                void operator()(Struct_type_descriptor &type)
                {
                    auto index_value = ::LLVMConstIntGetZExtValue(
                        this_->get_id_state(index).constant->get_or_make_value());
                    auto &members = type.get_members(true);
                    if(index_value >= members.size())
                        throw Parser_error(instruction_start_index,
                                           instruction_start_index,
                                           "index out of range in OpAccessChain");
                    llvm_indexes.push_back(::LLVMConstInt(::LLVMInt32TypeInContext(this_->context),
                                                          members[index_value].llvm_member_index,
                                                          false));
                    current_type = members[index_value].type;
                }
            };
            auto *type = current_type.get();
            type->visit(Visitor{instruction_start_index, current_type, llvm_indexes, index, this});
        }
        state.value = Value(
            ::LLVMBuildGEP(
                builder.get(), base.value, llvm_indexes.data(), llvm_indexes.size(), name.c_str()),
            get_type(instruction.result_type, instruction_start_index));
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_in_bounds_access_chain(
    Op_in_bounds_access_chain instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_ptr_access_chain(Op_ptr_access_chain instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_array_length(Op_array_length instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_generic_ptr_mem_semantics(
    Op_generic_ptr_mem_semantics instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_in_bounds_ptr_access_chain(
    Op_in_bounds_ptr_access_chain instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_decorate(
    Op_decorate instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    get_id_state(instruction.target).decorations.push_back(std::move(instruction.decoration));
}

void Spirv_to_llvm::handle_instruction_op_member_decorate(
    Op_member_decorate instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    auto &state = get_id_state(instruction.structure_type);
    state.member_decorations.push_back(std::move(instruction));
}

void Spirv_to_llvm::handle_instruction_op_decoration_group(Op_decoration_group instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_decorate(Op_group_decorate instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_member_decorate(
    Op_group_member_decorate instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_vector_extract_dynamic(
    Op_vector_extract_dynamic instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_vector_insert_dynamic(
    Op_vector_insert_dynamic instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_vector_shuffle(Op_vector_shuffle instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_composite_construct(Op_composite_construct instruction,
                                                              std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        ::LLVMValueRef result_value = nullptr;
        std::string name = get_name(instruction.result);
        struct Visitor
        {
            Op_composite_construct &instruction;
            std::size_t instruction_start_index;
            Id_state &state;
            ::LLVMValueRef &result_value;
            std::string &name;
            Spirv_to_llvm *this_;
            void operator()(Simple_type_descriptor &)
            {
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "invalid result type for OpCompositeConstruct");
            }
            void operator()(Vector_type_descriptor &type)
            {
                if(instruction.constituents.size() < 2)
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "too few inputs to construct a vector");
                result_value = ::LLVMGetUndef(type.get_or_make_type().type);
                std::uint32_t insert_index = 0;
                auto insert_element = [&](::LLVMValueRef element)
                {
                    if(insert_index >= type.get_element_count())
                        throw Parser_error(
                            instruction_start_index,
                            instruction_start_index,
                            "too many input vector elements to fit in output vector");
                    result_value = ::LLVMBuildInsertElement(
                        this_->builder.get(),
                        result_value,
                        element,
                        ::LLVMConstInt(
                            ::LLVMInt32TypeInContext(this_->context), insert_index, false),
                        insert_index + 1 == type.get_element_count() ? name.c_str() : "");
                    insert_index++;
                };
                for(Id input : instruction.constituents)
                {
                    auto &value = this_->get_id_state(input).value.value();
                    if(auto *vector_type = dynamic_cast<Vector_type_descriptor *>(value.type.get()))
                    {
                        for(std::uint32_t i = 0; i < vector_type->get_element_count(); i++)
                        {
                            insert_element(::LLVMBuildExtractElement(
                                this_->builder.get(),
                                value.value,
                                ::LLVMConstInt(
                                    ::LLVMInt32TypeInContext(this_->context), insert_index, false),
                                ""));
                        }
                    }
                    else
                    {
                        insert_element(value.value);
                    }
                }
                if(insert_index < type.get_element_count())
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "too few input vector elements to fill output vector");
            }
            void operator()(Matrix_type_descriptor &)
            {
#warning finish
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "unimplemented result type for OpCompositeConstruct");
            }
            void operator()(Array_type_descriptor &)
            {
#warning finish
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "unimplemented result type for OpCompositeConstruct");
            }
            void operator()(Pointer_type_descriptor &)
            {
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "invalid result type for OpCompositeConstruct");
            }
            void operator()(Function_type_descriptor &)
            {
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "invalid result type for OpCompositeConstruct");
            }
            void operator()(Struct_type_descriptor &)
            {
#warning finish
                throw Parser_error(instruction_start_index,
                                   instruction_start_index,
                                   "unimplemented result type for OpCompositeConstruct");
            }
        };
        result_type->visit(
            Visitor{instruction, instruction_start_index, state, result_value, name, this});
        state.value = Value(result_value, std::move(result_type));
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_composite_extract(Op_composite_extract instruction,
                                                            std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result = get_id_state(instruction.composite).value.value();
        std::string name = "";
        for(std::size_t i = 0; i < instruction.indexes.size(); i++)
        {
            std::uint32_t index = instruction.indexes[i];
            if(i == instruction.indexes.size() - 1)
                name = get_name(instruction.result);
            struct Visitor
            {
                std::size_t instruction_start_index;
                Id_state &state;
                Value &result;
                std::string &name;
                std::uint32_t index;
                ::LLVMContextRef context;
                llvm_wrapper::Builder &builder;
                void operator()(Simple_type_descriptor &)
                {
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "invalid composite type for OpCompositeExtract");
                }
                void operator()(Vector_type_descriptor &type)
                {
                    if(index >= type.get_element_count())
                        throw Parser_error(instruction_start_index,
                                           instruction_start_index,
                                           "index out of range in OpCompositeExtract");
                    result =
                        Value(::LLVMBuildExtractElement(
                                  builder.get(),
                                  result.value,
                                  ::LLVMConstInt(::LLVMInt32TypeInContext(context), index, false),
                                  name.c_str()),
                              type.get_element_type());
                }
                void operator()(Matrix_type_descriptor &)
                {
#warning finish
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "unimplemented composite type for OpCompositeExtract");
                }
                void operator()(Array_type_descriptor &)
                {
#warning finish
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "unimplemented composite type for OpCompositeExtract");
                }
                void operator()(Pointer_type_descriptor &)
                {
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "invalid composite type for OpCompositeExtract");
                }
                void operator()(Function_type_descriptor &)
                {
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "invalid composite type for OpCompositeExtract");
                }
                void operator()(Struct_type_descriptor &)
                {
#warning finish
                    throw Parser_error(instruction_start_index,
                                       instruction_start_index,
                                       "unimplemented composite type for OpCompositeExtract");
                }
            };
            auto *type = result.type.get();
            type->visit(
                Visitor{instruction_start_index, state, result, name, index, context, builder});
        }
        state.value = result;
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_composite_insert(Op_composite_insert instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_copy_object(Op_copy_object instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_transpose(Op_transpose instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_sampled_image(Op_sampled_image instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sample_implicit_lod(
    Op_image_sample_implicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sample_explicit_lod(
    Op_image_sample_explicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sample_dref_implicit_lod(
    Op_image_sample_dref_implicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sample_dref_explicit_lod(
    Op_image_sample_dref_explicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sample_proj_implicit_lod(
    Op_image_sample_proj_implicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sample_proj_explicit_lod(
    Op_image_sample_proj_explicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sample_proj_dref_implicit_lod(
    Op_image_sample_proj_dref_implicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sample_proj_dref_explicit_lod(
    Op_image_sample_proj_dref_explicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_fetch(Op_image_fetch instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_gather(Op_image_gather instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_dref_gather(Op_image_dref_gather instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_read(Op_image_read instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_write(Op_image_write instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image(Op_image instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_query_format(Op_image_query_format instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_query_order(Op_image_query_order instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_query_size_lod(Op_image_query_size_lod instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_query_size(Op_image_query_size instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_query_lod(Op_image_query_lod instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_query_levels(Op_image_query_levels instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_query_samples(Op_image_query_samples instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_convert_f_to_u(Op_convert_f_to_u instruction,
                                                         std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        state.value =
            Value(::LLVMBuildFPToUI(builder.get(),
                                    get_id_state(instruction.float_value).value.value().value,
                                    result_type->get_or_make_type().type,
                                    get_name(instruction.result).c_str()),
                  result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_convert_f_to_s(Op_convert_f_to_s instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_convert_s_to_f(Op_convert_s_to_f instruction,
                                                         std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        state.value =
            Value(::LLVMBuildSIToFP(builder.get(),
                                    get_id_state(instruction.signed_value).value.value().value,
                                    result_type->get_or_make_type().type,
                                    get_name(instruction.result).c_str()),
                  result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_convert_u_to_f(Op_convert_u_to_f instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_u_convert(Op_u_convert instruction,
                                                    std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        auto result_type_int_width = ::LLVMGetIntTypeWidth(
            llvm_wrapper::get_scalar_or_vector_element_type(result_type->get_or_make_type().type));
        auto &arg = get_id_state(instruction.unsigned_value).value.value();
        auto arg_int_width = ::LLVMGetIntTypeWidth(
            llvm_wrapper::get_scalar_or_vector_element_type(arg.type->get_or_make_type().type));
        auto opcode = ::LLVMTrunc;
        if(result_type_int_width > arg_int_width)
            opcode = ::LLVMZExt;
        state.value = Value(::LLVMBuildCast(builder.get(),
                                            opcode,
                                            arg.value,
                                            result_type->get_or_make_type().type,
                                            get_name(instruction.result).c_str()),
                            result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_s_convert(Op_s_convert instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_convert(Op_f_convert instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_quantize_to_f16(Op_quantize_to_f16 instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_convert_ptr_to_u(Op_convert_ptr_to_u instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_sat_convert_s_to_u(Op_sat_convert_s_to_u instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_sat_convert_u_to_s(Op_sat_convert_u_to_s instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_convert_u_to_ptr(Op_convert_u_to_ptr instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_ptr_cast_to_generic(Op_ptr_cast_to_generic instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_generic_cast_to_ptr(Op_generic_cast_to_ptr instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_generic_cast_to_ptr_explicit(
    Op_generic_cast_to_ptr_explicit instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bitcast(Op_bitcast instruction,
                                                  std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        auto &arg = get_id_state(instruction.operand).value.value();
        std::size_t result_element_count = 1; // scalar is equivalent to size 1 vector
        std::size_t arg_element_count = 1;
        if(auto *t = dynamic_cast<const Vector_type_descriptor *>(result_type.get()))
            result_element_count = t->get_element_count();
        if(auto *t = dynamic_cast<const Vector_type_descriptor *>(result_type.get()))
            arg_element_count = t->get_element_count();
        if(result_element_count != arg_element_count)
        {
// need to bitcast as if on little endian system even on big endian
#warning finish implementing element-count-changing bitcast
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "element-count-changing OpBitcast is not implemented");
        }
        state.value = Value(::LLVMBuildBitCast(builder.get(),
                                               arg.value,
                                               result_type->get_or_make_type().type,
                                               get_name(instruction.result).c_str()),
                            result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_s_negate(Op_s_negate instruction,
                                                   std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_negate(Op_f_negate instruction,
                                                   std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        auto &arg = get_id_state(instruction.operand).value.value();
        state.value =
            Value(::LLVMBuildFNeg(builder.get(), arg.value, get_name(instruction.result).c_str()),
                  result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_i_add(Op_i_add instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_add(Op_f_add instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_i_sub(Op_i_sub instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_sub(Op_f_sub instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_i_mul(Op_i_mul instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_mul(Op_f_mul instruction,
                                                std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        state.value = Value(::LLVMBuildFMul(builder.get(),
                                            get_id_state(instruction.operand_1).value.value().value,
                                            get_id_state(instruction.operand_2).value.value().value,
                                            get_name(instruction.result).c_str()),
                            result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_u_div(Op_u_div instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_s_div(Op_s_div instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_div(Op_f_div instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_u_mod(Op_u_mod instruction,
                                                std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        state.value = Value(::LLVMBuildURem(builder.get(),
                                            get_id_state(instruction.operand_1).value.value().value,
                                            get_id_state(instruction.operand_2).value.value().value,
                                            get_name(instruction.result).c_str()),
                            result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_s_rem(Op_s_rem instruction,
                                                std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        state.value = Value(::LLVMBuildSRem(builder.get(),
                                            get_id_state(instruction.operand_1).value.value().value,
                                            get_id_state(instruction.operand_2).value.value().value,
                                            get_name(instruction.result).c_str()),
                            result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_s_mod(Op_s_mod instruction,
                                                std::size_t instruction_start_index)
{
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &state = get_id_state(instruction.result);
        if(!state.decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on instruction not implemented: "
                                   + std::string(get_enumerant_name(instruction.get_operation())));
        auto result_type = get_type(instruction.result_type, instruction_start_index);
        state.value =
            Value(builder.build_smod(get_id_state(instruction.operand_1).value.value().value,
                                     get_id_state(instruction.operand_2).value.value().value,
                                     get_name(instruction.result).c_str()),
                  result_type);
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_f_rem(Op_f_rem instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_mod(Op_f_mod instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_vector_times_scalar(Op_vector_times_scalar instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_matrix_times_scalar(Op_matrix_times_scalar instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_vector_times_matrix(Op_vector_times_matrix instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_matrix_times_vector(Op_matrix_times_vector instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_matrix_times_matrix(Op_matrix_times_matrix instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_outer_product(Op_outer_product instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_dot(Op_dot instruction,
                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_i_add_carry(Op_i_add_carry instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_i_sub_borrow(Op_i_sub_borrow instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_u_mul_extended(Op_u_mul_extended instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_s_mul_extended(Op_s_mul_extended instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_any(Op_any instruction,
                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_all(Op_all instruction,
                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_is_nan(Op_is_nan instruction,
                                                 std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_is_inf(Op_is_inf instruction,
                                                 std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_is_finite(Op_is_finite instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_is_normal(Op_is_normal instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_sign_bit_set(Op_sign_bit_set instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_less_or_greater(Op_less_or_greater instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_ordered(Op_ordered instruction,
                                                  std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_unordered(Op_unordered instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_logical_equal(Op_logical_equal instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_logical_not_equal(Op_logical_not_equal instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_logical_or(Op_logical_or instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_logical_and(Op_logical_and instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_logical_not(Op_logical_not instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_select(Op_select instruction,
                                                 std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_i_equal(Op_i_equal instruction,
                                                  std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_i_not_equal(Op_i_not_equal instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_u_greater_than(Op_u_greater_than instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_s_greater_than(Op_s_greater_than instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_u_greater_than_equal(Op_u_greater_than_equal instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_s_greater_than_equal(Op_s_greater_than_equal instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_u_less_than(Op_u_less_than instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_s_less_than(Op_s_less_than instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_u_less_than_equal(Op_u_less_than_equal instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_s_less_than_equal(Op_s_less_than_equal instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_ord_equal(Op_f_ord_equal instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_unord_equal(Op_f_unord_equal instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_ord_not_equal(Op_f_ord_not_equal instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_unord_not_equal(Op_f_unord_not_equal instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_ord_less_than(Op_f_ord_less_than instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_unord_less_than(Op_f_unord_less_than instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_ord_greater_than(Op_f_ord_greater_than instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_unord_greater_than(Op_f_unord_greater_than instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_ord_less_than_equal(
    Op_f_ord_less_than_equal instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_unord_less_than_equal(
    Op_f_unord_less_than_equal instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_ord_greater_than_equal(
    Op_f_ord_greater_than_equal instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_f_unord_greater_than_equal(
    Op_f_unord_greater_than_equal instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_shift_right_logical(Op_shift_right_logical instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_shift_right_arithmetic(
    Op_shift_right_arithmetic instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_shift_left_logical(Op_shift_left_logical instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bitwise_or(Op_bitwise_or instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bitwise_xor(Op_bitwise_xor instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bitwise_and(Op_bitwise_and instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_not(Op_not instruction,
                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bit_field_insert(Op_bit_field_insert instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bit_field_s_extract(Op_bit_field_s_extract instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bit_field_u_extract(Op_bit_field_u_extract instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bit_reverse(Op_bit_reverse instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_bit_count(Op_bit_count instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_d_pdx(Op_d_pdx instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_d_pdy(Op_d_pdy instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_fwidth(Op_fwidth instruction,
                                                 std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_d_pdx_fine(Op_d_pdx_fine instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_d_pdy_fine(Op_d_pdy_fine instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_fwidth_fine(Op_fwidth_fine instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_d_pdx_coarse(Op_d_pdx_coarse instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_d_pdy_coarse(Op_d_pdy_coarse instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_fwidth_coarse(Op_fwidth_coarse instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_emit_vertex(Op_emit_vertex instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_end_primitive(Op_end_primitive instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_emit_stream_vertex(Op_emit_stream_vertex instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_end_stream_primitive(Op_end_stream_primitive instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_control_barrier(Op_control_barrier instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_memory_barrier(Op_memory_barrier instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_load(Op_atomic_load instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_store(Op_atomic_store instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_exchange(Op_atomic_exchange instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_compare_exchange(
    Op_atomic_compare_exchange instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_compare_exchange_weak(
    Op_atomic_compare_exchange_weak instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_i_increment(Op_atomic_i_increment instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_i_decrement(Op_atomic_i_decrement instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_i_add(Op_atomic_i_add instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_i_sub(Op_atomic_i_sub instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_s_min(Op_atomic_s_min instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_u_min(Op_atomic_u_min instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_s_max(Op_atomic_s_max instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_u_max(Op_atomic_u_max instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_and(Op_atomic_and instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_or(Op_atomic_or instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_xor(Op_atomic_xor instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_phi(Op_phi instruction,
                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_loop_merge(Op_loop_merge instruction,
                                                     std::size_t instruction_start_index)
{
    last_merge_instruction =
        Last_merge_instruction(std::move(instruction), instruction_start_index);
}

void Spirv_to_llvm::handle_instruction_op_selection_merge(Op_selection_merge instruction,
                                                          std::size_t instruction_start_index)
{
    last_merge_instruction =
        Last_merge_instruction(std::move(instruction), instruction_start_index);
}

void Spirv_to_llvm::handle_instruction_op_label(Op_label instruction,
                                                std::size_t instruction_start_index)
{
    if(current_function_id == 0)
        throw Parser_error(instruction_start_index,
                           instruction_start_index,
                           "OpLabel not allowed outside a function");
    if(current_basic_block_id != 0)
        throw Parser_error(instruction_start_index,
                           instruction_start_index,
                           "missing block terminator before OpLabel");
    current_basic_block_id = instruction.result;
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        auto &function = get_id_state(current_function_id).function.value();
        if(!get_id_state(current_basic_block_id).decorations.empty())
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "decorations on label not implemented");
        auto block = get_or_make_label(instruction.result);
        ::LLVMPositionBuilderAtEnd(builder.get(), block);
        if(!function.entry_block)
        {
            auto io_struct_value = ::LLVMGetParam(function.function, io_struct_argument_index);
            auto inputs_struct_value = ::LLVMBuildLoad(
                builder.get(),
                ::LLVMBuildStructGEP(
                    builder.get(),
                    io_struct_value,
                    io_struct->get_members(true)[this->inputs_member].llvm_member_index,
                    "inputs_pointer"),
                "inputs");
            auto outputs_struct_value = ::LLVMBuildLoad(
                builder.get(),
                ::LLVMBuildStructGEP(
                    builder.get(),
                    io_struct_value,
                    io_struct->get_members(true)[this->outputs_member].llvm_member_index,
                    "outputs_pointer"),
                "outputs");
            function.entry_block = Function_state::Entry_block(
                block, io_struct_value, inputs_struct_value, outputs_struct_value);
            for(auto iter = function_entry_block_handlers.begin();
                iter != function_entry_block_handlers.end();)
            {
                auto fn = *iter++;
                // increment before calling in case the hander removes itself
                fn();
            }
        }
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_branch(
    Op_branch instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    auto merge = std::move(last_merge_instruction);
    last_merge_instruction.reset();
    current_basic_block_id = 0;
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        ::LLVMBuildBr(builder.get(), get_or_make_label(instruction.target_label));
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_branch_conditional(Op_branch_conditional instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_switch(
    Op_switch instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    auto merge = std::move(last_merge_instruction.value());
    last_merge_instruction.reset();
    current_basic_block_id = 0;
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        for(auto &target : instruction.target)
            get_or_make_label(target.part_2); // create basic blocks first
        auto selector = get_id_state(instruction.selector).value.value();
        auto switch_instruction = ::LLVMBuildSwitch(builder.get(),
                                                    selector.value,
                                                    get_or_make_label(instruction.default_),
                                                    instruction.target.size());
        for(auto &target : instruction.target)
            ::LLVMAddCase(
                switch_instruction,
                ::LLVMConstInt(selector.type->get_or_make_type().type, target.part_1, false),
                get_or_make_label(target.part_2));
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_kill(Op_kill instruction,
                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_return(
    [[gnu::unused]] Op_return instruction, [[gnu::unused]] std::size_t instruction_start_index)
{
    current_basic_block_id = 0;
    switch(stage)
    {
    case Stage::calculate_types:
        break;
    case Stage::generate_code:
    {
        ::LLVMBuildRetVoid(builder.get());
        break;
    }
    }
}

void Spirv_to_llvm::handle_instruction_op_return_value(Op_return_value instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_unreachable(Op_unreachable instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_lifetime_start(Op_lifetime_start instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_lifetime_stop(Op_lifetime_stop instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_async_copy(Op_group_async_copy instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_wait_events(Op_group_wait_events instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_all(Op_group_all instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_any(Op_group_any instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_broadcast(Op_group_broadcast instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_i_add(Op_group_i_add instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_f_add(Op_group_f_add instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_f_min(Op_group_f_min instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_u_min(Op_group_u_min instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_s_min(Op_group_s_min instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_f_max(Op_group_f_max instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_u_max(Op_group_u_max instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_s_max(Op_group_s_max instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_read_pipe(Op_read_pipe instruction,
                                                    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_write_pipe(Op_write_pipe instruction,
                                                     std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_reserved_read_pipe(Op_reserved_read_pipe instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_reserved_write_pipe(Op_reserved_write_pipe instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_reserve_read_pipe_packets(
    Op_reserve_read_pipe_packets instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_reserve_write_pipe_packets(
    Op_reserve_write_pipe_packets instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_commit_read_pipe(Op_commit_read_pipe instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_commit_write_pipe(Op_commit_write_pipe instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_is_valid_reserve_id(Op_is_valid_reserve_id instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_num_pipe_packets(Op_get_num_pipe_packets instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_max_pipe_packets(Op_get_max_pipe_packets instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_reserve_read_pipe_packets(
    Op_group_reserve_read_pipe_packets instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_reserve_write_pipe_packets(
    Op_group_reserve_write_pipe_packets instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_commit_read_pipe(
    Op_group_commit_read_pipe instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_group_commit_write_pipe(
    Op_group_commit_write_pipe instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_enqueue_marker(Op_enqueue_marker instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_enqueue_kernel(Op_enqueue_kernel instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_kernel_n_drange_sub_group_count(
    Op_get_kernel_n_drange_sub_group_count instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_kernel_n_drange_max_sub_group_size(
    Op_get_kernel_n_drange_max_sub_group_size instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_kernel_work_group_size(
    Op_get_kernel_work_group_size instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_kernel_preferred_work_group_size_multiple(
    Op_get_kernel_preferred_work_group_size_multiple instruction,
    std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_retain_event(Op_retain_event instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_release_event(Op_release_event instruction,
                                                        std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_create_user_event(Op_create_user_event instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_is_valid_event(Op_is_valid_event instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_set_user_event_status(
    Op_set_user_event_status instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_capture_event_profiling_info(
    Op_capture_event_profiling_info instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_default_queue(Op_get_default_queue instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_build_nd_range(Op_build_nd_range instruction,
                                                         std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_sample_implicit_lod(
    Op_image_sparse_sample_implicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_sample_explicit_lod(
    Op_image_sparse_sample_explicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_sample_dref_implicit_lod(
    Op_image_sparse_sample_dref_implicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_sample_dref_explicit_lod(
    Op_image_sparse_sample_dref_explicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_sample_proj_implicit_lod(
    Op_image_sparse_sample_proj_implicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_sample_proj_explicit_lod(
    Op_image_sparse_sample_proj_explicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_sample_proj_dref_implicit_lod(
    Op_image_sparse_sample_proj_dref_implicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_sample_proj_dref_explicit_lod(
    Op_image_sparse_sample_proj_dref_explicit_lod instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_fetch(Op_image_sparse_fetch instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_gather(Op_image_sparse_gather instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_dref_gather(
    Op_image_sparse_dref_gather instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_texels_resident(
    Op_image_sparse_texels_resident instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_no_line(Op_no_line instruction,
                                                  std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_flag_test_and_set(
    Op_atomic_flag_test_and_set instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_atomic_flag_clear(Op_atomic_flag_clear instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_image_sparse_read(Op_image_sparse_read instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_size_of(Op_size_of instruction,
                                                  std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_pipe_storage(Op_type_pipe_storage instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_constant_pipe_storage(
    Op_constant_pipe_storage instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_create_pipe_from_pipe_storage(
    Op_create_pipe_from_pipe_storage instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_kernel_local_size_for_subgroup_count(
    Op_get_kernel_local_size_for_subgroup_count instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_get_kernel_max_num_subgroups(
    Op_get_kernel_max_num_subgroups instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_type_named_barrier(Op_type_named_barrier instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_named_barrier_initialize(
    Op_named_barrier_initialize instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_memory_named_barrier(Op_memory_named_barrier instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_module_processed(Op_module_processed instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_execution_mode_id(Op_execution_mode_id instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_decorate_id(Op_decorate_id instruction,
                                                      std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_subgroup_ballot_khr(Op_subgroup_ballot_khr instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_subgroup_first_invocation_khr(
    Op_subgroup_first_invocation_khr instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_subgroup_all_khr(Op_subgroup_all_khr instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_subgroup_any_khr(Op_subgroup_any_khr instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_subgroup_all_equal_khr(
    Op_subgroup_all_equal_khr instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_subgroup_read_invocation_khr(
    Op_subgroup_read_invocation_khr instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}
}
}
