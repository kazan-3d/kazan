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
#include "spirv_to_llvm.h"
#include "util/optional.h"
#include "util/enum.h"

namespace vulkan_cpu
{
namespace spirv_to_llvm
{
using namespace spirv;

void Struct_type_descriptor::complete_type(bool need_complete_structs)
{
#warning finish Struct_type_descriptor::complete_type
    static_cast<void>(need_complete_structs);
    throw Parser_error(0, 0, "not implemented: Struct_descriptor::complete_type");
}

namespace
{
enum class Stage
{
    calculate_types,
    generate_code,
};

vulkan_cpu_util_generate_enum_traits(Stage, Stage::calculate_types, Stage::generate_code);

static_assert(util::Enum_traits<Stage>::is_compact, "");
}

class Spirv_to_llvm : public Parser_callbacks
{
    Spirv_to_llvm(const Spirv_to_llvm &) = delete;
    Spirv_to_llvm &operator=(const Spirv_to_llvm &) = delete;

private:
    struct Op_string_state
    {
        Literal_string value;
    };
    struct Op_ext_inst_import_state
    {
    };
    struct Op_entry_point_state
    {
        Op_entry_point entry_point;
        std::size_t instruction_start_index;
        util::optional<Execution_mode_with_parameters> execution_mode;
    };
    struct Name
    {
        std::string name;
    };
    struct Id_state
    {
        util::optional<Op_string_state> op_string;
        util::optional<Op_ext_inst_import_state> op_ext_inst_import;
        util::optional<Name> name;
        std::shared_ptr<Type_descriptor> type;
        std::vector<Op_entry_point_state> op_entry_points;
        std::vector<Decoration_with_parameters> decorations;
        std::vector<Op_member_decorate> member_decorations;
        std::vector<Op_member_name> member_names;
        template <typename Fn>
        void visit(Fn fn)
        {
            if(op_string)
                fn(*op_string);
            if(op_ext_inst_import)
                fn(*op_ext_inst_import);
            if(name)
                fn(*name);
            if(type)
                fn(*type);
            for(auto &i : op_entry_points)
                fn(i);
            for(auto &i : decorations)
                fn(i);
            for(auto &i : member_decorations)
                fn(i);
            for(auto &i : member_names)
                fn(i);
        }
        Id_state() noexcept
        {
        }
    };

private:
    std::vector<Id_state> id_states;
    unsigned input_version_number_major = 0;
    unsigned input_version_number_minor = 0;
    Word input_generator_magic_number = 0;
    util::Enum_set<Capability> enabled_capabilities;
    ::LLVMContextRef context;
    llvm_wrapper::Module module;
    std::shared_ptr<Struct_type_descriptor> io_struct;
    std::array<std::shared_ptr<Type_descriptor>, 1> implicit_function_arguments;
    Stage stage;

private:
    Id_state &get_id_state(Id id)
    {
        assert(id != 0 && id <= id_states.size());
        return id_states[id - 1];
    }
    const std::shared_ptr<Type_descriptor> &get_type(Id id, std::size_t instruction_start_index)
    {
        auto &state = get_id_state(id);
        if(!state.type)
            throw Parser_error(
                instruction_start_index, instruction_start_index, "id is not a type");
        return state.type;
    }

public:
    explicit Spirv_to_llvm(::LLVMContextRef context) : context(context), stage()
    {
        module = llvm_wrapper::Module::create("", context);
        constexpr std::size_t no_instruction_index = 0;
        io_struct =
            std::make_shared<Struct_type_descriptor>(context, "Io_struct", no_instruction_index);
        assert(implicit_function_arguments.size() == 1);
        implicit_function_arguments[0] = io_struct;
    }
    Converted_module run(const Word *shader_words, std::size_t shader_size)
    {
        stage = Stage::calculate_types;
        spirv::parse(*this, shader_words, shader_size);
        for(auto &id_state : id_states)
            if(id_state.type)
                id_state.type->get_or_make_type(true);
        for(auto &arg : implicit_function_arguments)
            arg->get_or_make_type(true);
#warning finish Spirv_to_llvm::run
        stage = Stage::generate_code;
        spirv::parse(*this, shader_words, shader_size);
        std::vector<Converted_module::Entry_point> entry_points;
        for(auto &id_state : id_states)
        {
            for(auto &entry_point : id_state.op_entry_points)
            {
                entry_points.push_back(
                    Converted_module::Entry_point(std::string(entry_point.entry_point.name)));
            }
        }
        Converted_module retval(std::move(module), std::move(entry_points), std::move(io_struct));
        return retval;
    }
    virtual void handle_header(unsigned version_number_major,
                               unsigned version_number_minor,
                               Word generator_magic_number,
                               Word id_bound,
                               Word instruction_schema) override;
    virtual void handle_instruction_op_nop(Op_nop instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_undef(Op_undef instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_source_continued(
        Op_source_continued instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_source(Op_source instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_source_extension(
        Op_source_extension instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_name(Op_name instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_member_name(Op_member_name instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_string(Op_string instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_line(Op_line instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_extension(Op_extension instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ext_inst_import(
        Op_ext_inst_import instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ext_inst(Op_ext_inst instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_memory_model(Op_memory_model instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_entry_point(Op_entry_point instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_execution_mode(Op_execution_mode instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_capability(Op_capability instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_void(Op_type_void instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_bool(Op_type_bool instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_int(Op_type_int instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_float(Op_type_float instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_vector(Op_type_vector instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_matrix(Op_type_matrix instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_image(Op_type_image instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_sampler(Op_type_sampler instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_sampled_image(
        Op_type_sampled_image instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_array(Op_type_array instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_runtime_array(
        Op_type_runtime_array instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_struct(Op_type_struct instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_opaque(Op_type_opaque instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_pointer(Op_type_pointer instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_function(Op_type_function instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_event(Op_type_event instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_device_event(
        Op_type_device_event instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_reserve_id(
        Op_type_reserve_id instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_queue(Op_type_queue instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_pipe(Op_type_pipe instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_forward_pointer(
        Op_type_forward_pointer instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_true(Op_constant_true instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_false(Op_constant_false instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant(Op_constant instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_composite(
        Op_constant_composite instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_sampler(
        Op_constant_sampler instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_null(Op_constant_null instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant_true(
        Op_spec_constant_true instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant_false(
        Op_spec_constant_false instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant(Op_spec_constant instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant_composite(
        Op_spec_constant_composite instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_spec_constant_op(
        Op_spec_constant_op instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_function(Op_function instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_function_parameter(
        Op_function_parameter instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_function_end(Op_function_end instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_function_call(Op_function_call instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_variable(Op_variable instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_texel_pointer(
        Op_image_texel_pointer instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_load(Op_load instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_store(Op_store instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_copy_memory(Op_copy_memory instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_copy_memory_sized(
        Op_copy_memory_sized instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_access_chain(Op_access_chain instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_in_bounds_access_chain(
        Op_in_bounds_access_chain instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ptr_access_chain(
        Op_ptr_access_chain instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_array_length(Op_array_length instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_generic_ptr_mem_semantics(
        Op_generic_ptr_mem_semantics instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_in_bounds_ptr_access_chain(
        Op_in_bounds_ptr_access_chain instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_decorate(Op_decorate instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_member_decorate(
        Op_member_decorate instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_decoration_group(
        Op_decoration_group instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_decorate(Op_group_decorate instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_member_decorate(
        Op_group_member_decorate instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_extract_dynamic(
        Op_vector_extract_dynamic instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_insert_dynamic(
        Op_vector_insert_dynamic instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_shuffle(Op_vector_shuffle instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_composite_construct(
        Op_composite_construct instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_composite_extract(
        Op_composite_extract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_composite_insert(
        Op_composite_insert instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_copy_object(Op_copy_object instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_transpose(Op_transpose instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_sampled_image(Op_sampled_image instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_implicit_lod(
        Op_image_sample_implicit_lod instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_explicit_lod(
        Op_image_sample_explicit_lod instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_dref_implicit_lod(
        Op_image_sample_dref_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_dref_explicit_lod(
        Op_image_sample_dref_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_proj_implicit_lod(
        Op_image_sample_proj_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_proj_explicit_lod(
        Op_image_sample_proj_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_proj_dref_implicit_lod(
        Op_image_sample_proj_dref_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sample_proj_dref_explicit_lod(
        Op_image_sample_proj_dref_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_fetch(Op_image_fetch instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_gather(Op_image_gather instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_dref_gather(
        Op_image_dref_gather instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_read(Op_image_read instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_write(Op_image_write instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image(Op_image instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_format(
        Op_image_query_format instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_order(
        Op_image_query_order instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_size_lod(
        Op_image_query_size_lod instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_size(
        Op_image_query_size instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_lod(
        Op_image_query_lod instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_levels(
        Op_image_query_levels instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_query_samples(
        Op_image_query_samples instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_f_to_u(Op_convert_f_to_u instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_f_to_s(Op_convert_f_to_s instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_s_to_f(Op_convert_s_to_f instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_u_to_f(Op_convert_u_to_f instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_convert(Op_u_convert instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_convert(Op_s_convert instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_convert(Op_f_convert instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_quantize_to_f16(
        Op_quantize_to_f16 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_ptr_to_u(
        Op_convert_ptr_to_u instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_sat_convert_s_to_u(
        Op_sat_convert_s_to_u instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_sat_convert_u_to_s(
        Op_sat_convert_u_to_s instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_convert_u_to_ptr(
        Op_convert_u_to_ptr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ptr_cast_to_generic(
        Op_ptr_cast_to_generic instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_generic_cast_to_ptr(
        Op_generic_cast_to_ptr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_generic_cast_to_ptr_explicit(
        Op_generic_cast_to_ptr_explicit instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bitcast(Op_bitcast instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_negate(Op_s_negate instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_negate(Op_f_negate instruction,
                                                std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_add(Op_i_add instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_add(Op_f_add instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_sub(Op_i_sub instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_sub(Op_f_sub instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_mul(Op_i_mul instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_mul(Op_f_mul instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_div(Op_u_div instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_div(Op_s_div instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_div(Op_f_div instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_mod(Op_u_mod instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_rem(Op_s_rem instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_mod(Op_s_mod instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_rem(Op_f_rem instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_mod(Op_f_mod instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_times_scalar(
        Op_vector_times_scalar instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_matrix_times_scalar(
        Op_matrix_times_scalar instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_vector_times_matrix(
        Op_vector_times_matrix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_matrix_times_vector(
        Op_matrix_times_vector instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_matrix_times_matrix(
        Op_matrix_times_matrix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_outer_product(Op_outer_product instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_dot(Op_dot instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_add_carry(Op_i_add_carry instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_sub_borrow(Op_i_sub_borrow instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_mul_extended(Op_u_mul_extended instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_mul_extended(Op_s_mul_extended instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_any(Op_any instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_all(Op_all instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_nan(Op_is_nan instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_inf(Op_is_inf instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_finite(Op_is_finite instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_normal(Op_is_normal instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_sign_bit_set(Op_sign_bit_set instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_less_or_greater(
        Op_less_or_greater instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_ordered(Op_ordered instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_unordered(Op_unordered instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_equal(Op_logical_equal instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_not_equal(
        Op_logical_not_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_or(Op_logical_or instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_and(Op_logical_and instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_logical_not(Op_logical_not instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_select(Op_select instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_equal(Op_i_equal instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_i_not_equal(Op_i_not_equal instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_greater_than(Op_u_greater_than instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_greater_than(Op_s_greater_than instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_greater_than_equal(
        Op_u_greater_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_greater_than_equal(
        Op_s_greater_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_less_than(Op_u_less_than instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_less_than(Op_s_less_than instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_u_less_than_equal(
        Op_u_less_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_s_less_than_equal(
        Op_s_less_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_equal(Op_f_ord_equal instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_equal(Op_f_unord_equal instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_not_equal(
        Op_f_ord_not_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_not_equal(
        Op_f_unord_not_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_less_than(
        Op_f_ord_less_than instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_less_than(
        Op_f_unord_less_than instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_greater_than(
        Op_f_ord_greater_than instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_greater_than(
        Op_f_unord_greater_than instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_less_than_equal(
        Op_f_ord_less_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_less_than_equal(
        Op_f_unord_less_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_ord_greater_than_equal(
        Op_f_ord_greater_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_f_unord_greater_than_equal(
        Op_f_unord_greater_than_equal instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_shift_right_logical(
        Op_shift_right_logical instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_shift_right_arithmetic(
        Op_shift_right_arithmetic instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_shift_left_logical(
        Op_shift_left_logical instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bitwise_or(Op_bitwise_or instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bitwise_xor(Op_bitwise_xor instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bitwise_and(Op_bitwise_and instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_not(Op_not instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_field_insert(
        Op_bit_field_insert instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_field_s_extract(
        Op_bit_field_s_extract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_field_u_extract(
        Op_bit_field_u_extract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_reverse(Op_bit_reverse instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_bit_count(Op_bit_count instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdx(Op_d_pdx instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdy(Op_d_pdy instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_fwidth(Op_fwidth instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdx_fine(Op_d_pdx_fine instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdy_fine(Op_d_pdy_fine instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_fwidth_fine(Op_fwidth_fine instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdx_coarse(Op_d_pdx_coarse instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_d_pdy_coarse(Op_d_pdy_coarse instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_fwidth_coarse(Op_fwidth_coarse instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_emit_vertex(Op_emit_vertex instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_end_primitive(Op_end_primitive instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_emit_stream_vertex(
        Op_emit_stream_vertex instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_end_stream_primitive(
        Op_end_stream_primitive instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_control_barrier(
        Op_control_barrier instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_memory_barrier(Op_memory_barrier instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_load(Op_atomic_load instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_store(Op_atomic_store instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_exchange(
        Op_atomic_exchange instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_compare_exchange(
        Op_atomic_compare_exchange instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_compare_exchange_weak(
        Op_atomic_compare_exchange_weak instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_i_increment(
        Op_atomic_i_increment instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_i_decrement(
        Op_atomic_i_decrement instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_i_add(Op_atomic_i_add instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_i_sub(Op_atomic_i_sub instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_s_min(Op_atomic_s_min instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_u_min(Op_atomic_u_min instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_s_max(Op_atomic_s_max instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_u_max(Op_atomic_u_max instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_and(Op_atomic_and instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_or(Op_atomic_or instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_xor(Op_atomic_xor instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_phi(Op_phi instruction,
                                           std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_loop_merge(Op_loop_merge instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_selection_merge(
        Op_selection_merge instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_label(Op_label instruction,
                                             std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_branch(Op_branch instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_branch_conditional(
        Op_branch_conditional instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_switch(Op_switch instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_kill(Op_kill instruction,
                                            std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_return(Op_return instruction,
                                              std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_return_value(Op_return_value instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_unreachable(Op_unreachable instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_lifetime_start(Op_lifetime_start instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_lifetime_stop(Op_lifetime_stop instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_async_copy(
        Op_group_async_copy instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_wait_events(
        Op_group_wait_events instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_all(Op_group_all instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_any(Op_group_any instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_broadcast(
        Op_group_broadcast instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_i_add(Op_group_i_add instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_f_add(Op_group_f_add instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_f_min(Op_group_f_min instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_u_min(Op_group_u_min instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_s_min(Op_group_s_min instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_f_max(Op_group_f_max instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_u_max(Op_group_u_max instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_s_max(Op_group_s_max instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_read_pipe(Op_read_pipe instruction,
                                                 std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_write_pipe(Op_write_pipe instruction,
                                                  std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_reserved_read_pipe(
        Op_reserved_read_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_reserved_write_pipe(
        Op_reserved_write_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_reserve_read_pipe_packets(
        Op_reserve_read_pipe_packets instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_reserve_write_pipe_packets(
        Op_reserve_write_pipe_packets instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_commit_read_pipe(
        Op_commit_read_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_commit_write_pipe(
        Op_commit_write_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_valid_reserve_id(
        Op_is_valid_reserve_id instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_num_pipe_packets(
        Op_get_num_pipe_packets instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_max_pipe_packets(
        Op_get_max_pipe_packets instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_reserve_read_pipe_packets(
        Op_group_reserve_read_pipe_packets instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_reserve_write_pipe_packets(
        Op_group_reserve_write_pipe_packets instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_commit_read_pipe(
        Op_group_commit_read_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_group_commit_write_pipe(
        Op_group_commit_write_pipe instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_enqueue_marker(Op_enqueue_marker instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_enqueue_kernel(Op_enqueue_kernel instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_n_drange_sub_group_count(
        Op_get_kernel_n_drange_sub_group_count instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_n_drange_max_sub_group_size(
        Op_get_kernel_n_drange_max_sub_group_size instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_work_group_size(
        Op_get_kernel_work_group_size instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_preferred_work_group_size_multiple(
        Op_get_kernel_preferred_work_group_size_multiple instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_retain_event(Op_retain_event instruction,
                                                    std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_release_event(Op_release_event instruction,
                                                     std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_create_user_event(
        Op_create_user_event instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_is_valid_event(Op_is_valid_event instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_set_user_event_status(
        Op_set_user_event_status instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_capture_event_profiling_info(
        Op_capture_event_profiling_info instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_default_queue(
        Op_get_default_queue instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_build_nd_range(Op_build_nd_range instruction,
                                                      std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_implicit_lod(
        Op_image_sparse_sample_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_explicit_lod(
        Op_image_sparse_sample_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_dref_implicit_lod(
        Op_image_sparse_sample_dref_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_dref_explicit_lod(
        Op_image_sparse_sample_dref_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_proj_implicit_lod(
        Op_image_sparse_sample_proj_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_proj_explicit_lod(
        Op_image_sparse_sample_proj_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_proj_dref_implicit_lod(
        Op_image_sparse_sample_proj_dref_implicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_sample_proj_dref_explicit_lod(
        Op_image_sparse_sample_proj_dref_explicit_lod instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_fetch(
        Op_image_sparse_fetch instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_gather(
        Op_image_sparse_gather instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_dref_gather(
        Op_image_sparse_dref_gather instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_texels_resident(
        Op_image_sparse_texels_resident instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_no_line(Op_no_line instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_flag_test_and_set(
        Op_atomic_flag_test_and_set instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_atomic_flag_clear(
        Op_atomic_flag_clear instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_image_sparse_read(
        Op_image_sparse_read instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_size_of(Op_size_of instruction,
                                               std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_pipe_storage(
        Op_type_pipe_storage instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_constant_pipe_storage(
        Op_constant_pipe_storage instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_create_pipe_from_pipe_storage(
        Op_create_pipe_from_pipe_storage instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_local_size_for_subgroup_count(
        Op_get_kernel_local_size_for_subgroup_count instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_get_kernel_max_num_subgroups(
        Op_get_kernel_max_num_subgroups instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_type_named_barrier(
        Op_type_named_barrier instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_named_barrier_initialize(
        Op_named_barrier_initialize instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_memory_named_barrier(
        Op_memory_named_barrier instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_module_processed(
        Op_module_processed instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_execution_mode_id(
        Op_execution_mode_id instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_decorate_id(Op_decorate_id instruction,
                                                   std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_ballot_khr(
        Op_subgroup_ballot_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_first_invocation_khr(
        Op_subgroup_first_invocation_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_all_khr(
        Op_subgroup_all_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_any_khr(
        Op_subgroup_any_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_all_equal_khr(
        Op_subgroup_all_equal_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_op_subgroup_read_invocation_khr(
        Op_subgroup_read_invocation_khr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_acos(
        Open_cl_std_op_acos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_acosh(
        Open_cl_std_op_acosh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_acospi(
        Open_cl_std_op_acospi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_asin(
        Open_cl_std_op_asin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_asinh(
        Open_cl_std_op_asinh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_asinpi(
        Open_cl_std_op_asinpi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atan(
        Open_cl_std_op_atan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atan2(
        Open_cl_std_op_atan2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atanh(
        Open_cl_std_op_atanh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atanpi(
        Open_cl_std_op_atanpi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_atan2pi(
        Open_cl_std_op_atan2pi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cbrt(
        Open_cl_std_op_cbrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_ceil(
        Open_cl_std_op_ceil instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_copysign(
        Open_cl_std_op_copysign instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cos(
        Open_cl_std_op_cos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cosh(
        Open_cl_std_op_cosh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cospi(
        Open_cl_std_op_cospi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_erfc(
        Open_cl_std_op_erfc instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_erf(
        Open_cl_std_op_erf instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_exp(
        Open_cl_std_op_exp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_exp2(
        Open_cl_std_op_exp2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_exp10(
        Open_cl_std_op_exp10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_expm1(
        Open_cl_std_op_expm1 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fabs(
        Open_cl_std_op_fabs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fdim(
        Open_cl_std_op_fdim instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_floor(
        Open_cl_std_op_floor instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fma(
        Open_cl_std_op_fma instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmax(
        Open_cl_std_op_fmax instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmin(
        Open_cl_std_op_fmin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmod(
        Open_cl_std_op_fmod instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fract(
        Open_cl_std_op_fract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_frexp(
        Open_cl_std_op_frexp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_hypot(
        Open_cl_std_op_hypot instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_ilogb(
        Open_cl_std_op_ilogb instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_ldexp(
        Open_cl_std_op_ldexp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_lgamma(
        Open_cl_std_op_lgamma instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_lgamma_r(
        Open_cl_std_op_lgamma_r instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_log(
        Open_cl_std_op_log instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_log2(
        Open_cl_std_op_log2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_log10(
        Open_cl_std_op_log10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_log1p(
        Open_cl_std_op_log1p instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_logb(
        Open_cl_std_op_logb instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_mad(
        Open_cl_std_op_mad instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_maxmag(
        Open_cl_std_op_maxmag instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_minmag(
        Open_cl_std_op_minmag instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_modf(
        Open_cl_std_op_modf instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_nan(
        Open_cl_std_op_nan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_nextafter(
        Open_cl_std_op_nextafter instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_pow(
        Open_cl_std_op_pow instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_pown(
        Open_cl_std_op_pown instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_powr(
        Open_cl_std_op_powr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_remainder(
        Open_cl_std_op_remainder instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_remquo(
        Open_cl_std_op_remquo instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_rint(
        Open_cl_std_op_rint instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_rootn(
        Open_cl_std_op_rootn instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_round(
        Open_cl_std_op_round instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_rsqrt(
        Open_cl_std_op_rsqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sin(
        Open_cl_std_op_sin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sincos(
        Open_cl_std_op_sincos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sinh(
        Open_cl_std_op_sinh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sinpi(
        Open_cl_std_op_sinpi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sqrt(
        Open_cl_std_op_sqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_tan(
        Open_cl_std_op_tan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_tanh(
        Open_cl_std_op_tanh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_tanpi(
        Open_cl_std_op_tanpi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_tgamma(
        Open_cl_std_op_tgamma instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_trunc(
        Open_cl_std_op_trunc instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_cos(
        Open_cl_std_op_half_cos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_divide(
        Open_cl_std_op_half_divide instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_exp(
        Open_cl_std_op_half_exp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_exp2(
        Open_cl_std_op_half_exp2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_exp10(
        Open_cl_std_op_half_exp10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_log(
        Open_cl_std_op_half_log instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_log2(
        Open_cl_std_op_half_log2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_log10(
        Open_cl_std_op_half_log10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_powr(
        Open_cl_std_op_half_powr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_recip(
        Open_cl_std_op_half_recip instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_rsqrt(
        Open_cl_std_op_half_rsqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_sin(
        Open_cl_std_op_half_sin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_sqrt(
        Open_cl_std_op_half_sqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_half_tan(
        Open_cl_std_op_half_tan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_cos(
        Open_cl_std_op_native_cos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_divide(
        Open_cl_std_op_native_divide instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_exp(
        Open_cl_std_op_native_exp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_exp2(
        Open_cl_std_op_native_exp2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_exp10(
        Open_cl_std_op_native_exp10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_log(
        Open_cl_std_op_native_log instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_log2(
        Open_cl_std_op_native_log2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_log10(
        Open_cl_std_op_native_log10 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_powr(
        Open_cl_std_op_native_powr instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_recip(
        Open_cl_std_op_native_recip instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_rsqrt(
        Open_cl_std_op_native_rsqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_sin(
        Open_cl_std_op_native_sin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_sqrt(
        Open_cl_std_op_native_sqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_native_tan(
        Open_cl_std_op_native_tan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_abs(
        Open_cl_std_op_s_abs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_abs_diff(
        Open_cl_std_op_s_abs_diff instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_add_sat(
        Open_cl_std_op_s_add_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_add_sat(
        Open_cl_std_op_u_add_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_hadd(
        Open_cl_std_op_s_hadd instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_hadd(
        Open_cl_std_op_u_hadd instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_rhadd(
        Open_cl_std_op_s_rhadd instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_rhadd(
        Open_cl_std_op_u_rhadd instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_clamp(
        Open_cl_std_op_s_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_clamp(
        Open_cl_std_op_u_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_clz(
        Open_cl_std_op_clz instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_ctz(
        Open_cl_std_op_ctz instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mad_hi(
        Open_cl_std_op_s_mad_hi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mad_sat(
        Open_cl_std_op_u_mad_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mad_sat(
        Open_cl_std_op_s_mad_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_max(
        Open_cl_std_op_s_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_max(
        Open_cl_std_op_u_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_min(
        Open_cl_std_op_s_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_min(
        Open_cl_std_op_u_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mul_hi(
        Open_cl_std_op_s_mul_hi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_rotate(
        Open_cl_std_op_rotate instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_sub_sat(
        Open_cl_std_op_s_sub_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_sub_sat(
        Open_cl_std_op_u_sub_sat instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_upsample(
        Open_cl_std_op_u_upsample instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_upsample(
        Open_cl_std_op_s_upsample instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_popcount(
        Open_cl_std_op_popcount instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mad24(
        Open_cl_std_op_s_mad24 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mad24(
        Open_cl_std_op_u_mad24 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_s_mul24(
        Open_cl_std_op_s_mul24 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mul24(
        Open_cl_std_op_u_mul24 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_abs(
        Open_cl_std_op_u_abs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_abs_diff(
        Open_cl_std_op_u_abs_diff instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mul_hi(
        Open_cl_std_op_u_mul_hi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_u_mad_hi(
        Open_cl_std_op_u_mad_hi instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fclamp(
        Open_cl_std_op_fclamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_degrees(
        Open_cl_std_op_degrees instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmax_common(
        Open_cl_std_op_fmax_common instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fmin_common(
        Open_cl_std_op_fmin_common instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_mix(
        Open_cl_std_op_mix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_radians(
        Open_cl_std_op_radians instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_step(
        Open_cl_std_op_step instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_smoothstep(
        Open_cl_std_op_smoothstep instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_sign(
        Open_cl_std_op_sign instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_cross(
        Open_cl_std_op_cross instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_distance(
        Open_cl_std_op_distance instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_length(
        Open_cl_std_op_length instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_normalize(
        Open_cl_std_op_normalize instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fast_distance(
        Open_cl_std_op_fast_distance instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fast_length(
        Open_cl_std_op_fast_length instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_fast_normalize(
        Open_cl_std_op_fast_normalize instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_bitselect(
        Open_cl_std_op_bitselect instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_select(
        Open_cl_std_op_select instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vloadn(
        Open_cl_std_op_vloadn instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstoren(
        Open_cl_std_op_vstoren instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vload_half(
        Open_cl_std_op_vload_half instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vload_halfn(
        Open_cl_std_op_vload_halfn instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstore_half(
        Open_cl_std_op_vstore_half instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstore_half_r(
        Open_cl_std_op_vstore_half_r instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstore_halfn(
        Open_cl_std_op_vstore_halfn instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstore_halfn_r(
        Open_cl_std_op_vstore_halfn_r instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vloada_halfn(
        Open_cl_std_op_vloada_halfn instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstorea_halfn(
        Open_cl_std_op_vstorea_halfn instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_vstorea_halfn_r(
        Open_cl_std_op_vstorea_halfn_r instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_shuffle(
        Open_cl_std_op_shuffle instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_shuffle2(
        Open_cl_std_op_shuffle2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_printf(
        Open_cl_std_op_printf instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_open_cl_std_op_prefetch(
        Open_cl_std_op_prefetch instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_round(
        Glsl_std_450_op_round instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_round_even(
        Glsl_std_450_op_round_even instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_trunc(
        Glsl_std_450_op_trunc instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_abs(
        Glsl_std_450_op_f_abs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_abs(
        Glsl_std_450_op_s_abs instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_sign(
        Glsl_std_450_op_f_sign instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_sign(
        Glsl_std_450_op_s_sign instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_floor(
        Glsl_std_450_op_floor instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_ceil(
        Glsl_std_450_op_ceil instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_fract(
        Glsl_std_450_op_fract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_radians(
        Glsl_std_450_op_radians instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_degrees(
        Glsl_std_450_op_degrees instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_sin(
        Glsl_std_450_op_sin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_cos(
        Glsl_std_450_op_cos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_tan(
        Glsl_std_450_op_tan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_asin(
        Glsl_std_450_op_asin instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_acos(
        Glsl_std_450_op_acos instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_atan(
        Glsl_std_450_op_atan instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_sinh(
        Glsl_std_450_op_sinh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_cosh(
        Glsl_std_450_op_cosh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_tanh(
        Glsl_std_450_op_tanh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_asinh(
        Glsl_std_450_op_asinh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_acosh(
        Glsl_std_450_op_acosh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_atanh(
        Glsl_std_450_op_atanh instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_atan2(
        Glsl_std_450_op_atan2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pow(
        Glsl_std_450_op_pow instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_exp(
        Glsl_std_450_op_exp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_log(
        Glsl_std_450_op_log instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_exp2(
        Glsl_std_450_op_exp2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_log2(
        Glsl_std_450_op_log2 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_sqrt(
        Glsl_std_450_op_sqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_inverse_sqrt(
        Glsl_std_450_op_inverse_sqrt instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_determinant(
        Glsl_std_450_op_determinant instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_matrix_inverse(
        Glsl_std_450_op_matrix_inverse instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_modf(
        Glsl_std_450_op_modf instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_modf_struct(
        Glsl_std_450_op_modf_struct instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_min(
        Glsl_std_450_op_f_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_u_min(
        Glsl_std_450_op_u_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_min(
        Glsl_std_450_op_s_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_max(
        Glsl_std_450_op_f_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_u_max(
        Glsl_std_450_op_u_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_max(
        Glsl_std_450_op_s_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_clamp(
        Glsl_std_450_op_f_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_u_clamp(
        Glsl_std_450_op_u_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_s_clamp(
        Glsl_std_450_op_s_clamp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_f_mix(
        Glsl_std_450_op_f_mix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_i_mix(
        Glsl_std_450_op_i_mix instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_step(
        Glsl_std_450_op_step instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_smooth_step(
        Glsl_std_450_op_smooth_step instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_fma(
        Glsl_std_450_op_fma instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_frexp(
        Glsl_std_450_op_frexp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_frexp_struct(
        Glsl_std_450_op_frexp_struct instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_ldexp(
        Glsl_std_450_op_ldexp instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_snorm4x8(
        Glsl_std_450_op_pack_snorm4x8 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_unorm4x8(
        Glsl_std_450_op_pack_unorm4x8 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_snorm2x16(
        Glsl_std_450_op_pack_snorm2x16 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_unorm2x16(
        Glsl_std_450_op_pack_unorm2x16 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_half2x16(
        Glsl_std_450_op_pack_half2x16 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_pack_double2x32(
        Glsl_std_450_op_pack_double2x32 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_snorm2x16(
        Glsl_std_450_op_unpack_snorm2x16 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_unorm2x16(
        Glsl_std_450_op_unpack_unorm2x16 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_half2x16(
        Glsl_std_450_op_unpack_half2x16 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_snorm4x8(
        Glsl_std_450_op_unpack_snorm4x8 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_unorm4x8(
        Glsl_std_450_op_unpack_unorm4x8 instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_unpack_double2x32(
        Glsl_std_450_op_unpack_double2x32 instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_length(
        Glsl_std_450_op_length instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_distance(
        Glsl_std_450_op_distance instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_cross(
        Glsl_std_450_op_cross instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_normalize(
        Glsl_std_450_op_normalize instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_face_forward(
        Glsl_std_450_op_face_forward instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_reflect(
        Glsl_std_450_op_reflect instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_refract(
        Glsl_std_450_op_refract instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_find_i_lsb(
        Glsl_std_450_op_find_i_lsb instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_find_s_msb(
        Glsl_std_450_op_find_s_msb instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_find_u_msb(
        Glsl_std_450_op_find_u_msb instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_interpolate_at_centroid(
        Glsl_std_450_op_interpolate_at_centroid instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_interpolate_at_sample(
        Glsl_std_450_op_interpolate_at_sample instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_interpolate_at_offset(
        Glsl_std_450_op_interpolate_at_offset instruction,
        std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_n_min(
        Glsl_std_450_op_n_min instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_n_max(
        Glsl_std_450_op_n_max instruction, std::size_t instruction_start_index) override;
    virtual void handle_instruction_glsl_std_450_op_n_clamp(
        Glsl_std_450_op_n_clamp instruction, std::size_t instruction_start_index) override;
};

void Spirv_to_llvm::handle_header(unsigned version_number_major,
                                  unsigned version_number_minor,
                                  Word generator_magic_number,
                                  Word id_bound,
                                  [[gnu::unused]] Word instruction_schema)
{
    if(stage == util::Enum_traits<Stage>::values[0])
    {
        input_version_number_major = version_number_major;
        input_version_number_minor = version_number_minor;
        input_generator_magic_number = generator_magic_number;
        id_states.resize(id_bound - 1);
    }
}

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
        ::LLVMSetModuleIdentifier(module, filename.data(), filename.size());
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
        if(state.op_entry_points.back().execution_mode)
            throw Parser_error(
                instruction_start_index, instruction_start_index, "execution mode already set");
        state.op_entry_points.back().execution_mode = std::move(instruction.mode);
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
        get_id_state(instruction.result).type =
            std::make_shared<Simple_type_descriptor>(::LLVMVoidTypeInContext(context));
        break;
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
        switch(instruction.width)
        {
        case 8:
        case 16:
        case 32:
        case 64:
            state.type = std::make_shared<Simple_type_descriptor>(
                ::LLVMIntTypeInContext(context, instruction.width));
            break;
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
        switch(instruction.width)
        {
        case 16:
            state.type = std::make_shared<Simple_type_descriptor>(::LLVMHalfTypeInContext(context));
            break;
        case 32:
            state.type =
                std::make_shared<Simple_type_descriptor>(::LLVMFloatTypeInContext(context));
            break;
        case 64:
            state.type =
                std::make_shared<Simple_type_descriptor>(::LLVMDoubleTypeInContext(context));
            break;
        default:
            throw Parser_error(
                instruction_start_index, instruction_start_index, "invalid float width");
        }
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
        get_id_state(instruction.result)
            .type = std::make_shared<Simple_type_descriptor>(::LLVMVectorType(
            get_type(instruction.component_type, instruction_start_index)->get_or_make_type(false),
            instruction.component_count));
        break;
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
        auto column_type =
            get_type(instruction.column_type, instruction_start_index)->get_or_make_type(false);
        if(::LLVMGetTypeKind(column_type) != LLVMVectorTypeKind)
            throw Parser_error(instruction_start_index,
                               instruction_start_index,
                               "column type must be a vector type");
        get_id_state(instruction.result).type = std::make_shared<Simple_type_descriptor>(
            ::LLVMVectorType(::LLVMGetElementType(column_type),
                             instruction.column_count * ::LLVMGetVectorSize(column_type)));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
        state.type =
            std::make_shared<Struct_type_descriptor>(context,
                                                     state.name.value_or(Name{}).name.c_str(),
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
        if(!state.type)
        {
            state.type = std::make_shared<Pointer_type_descriptor>(
                get_type(instruction.type, instruction_start_index), instruction_start_index);
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
        for(Id_ref type : instruction.parameter_0_type_parameter_1_type)
            args.push_back(get_type(type, instruction_start_index));
        get_id_state(instruction.result).type = std::make_shared<Function_type_descriptor>(
            get_type(instruction.return_type, instruction_start_index),
            std::move(args),
            instruction_start_index);
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_constant_composite(Op_constant_composite instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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

void Spirv_to_llvm::handle_instruction_op_function_end(Op_function_end instruction,
                                                       std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_store(Op_store instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_composite_extract(Op_composite_extract instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_s_rem(Op_s_rem instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_s_mod(Op_s_mod instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_selection_merge(Op_selection_merge instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_label(Op_label instruction,
                                                std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_op_branch(Op_branch instruction,
                                                 std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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

void Spirv_to_llvm::handle_instruction_op_switch(Op_switch instruction,
                                                 std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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

void Spirv_to_llvm::handle_instruction_op_return(Op_return instruction,
                                                 std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
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

void Spirv_to_llvm::handle_instruction_open_cl_std_op_acos(Open_cl_std_op_acos instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_acosh(Open_cl_std_op_acosh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_acospi(Open_cl_std_op_acospi instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_asin(Open_cl_std_op_asin instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_asinh(Open_cl_std_op_asinh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_asinpi(Open_cl_std_op_asinpi instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_atan(Open_cl_std_op_atan instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_atan2(Open_cl_std_op_atan2 instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_atanh(Open_cl_std_op_atanh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_atanpi(Open_cl_std_op_atanpi instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_atan2pi(Open_cl_std_op_atan2pi instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_cbrt(Open_cl_std_op_cbrt instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_ceil(Open_cl_std_op_ceil instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_copysign(Open_cl_std_op_copysign instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_cos(Open_cl_std_op_cos instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_cosh(Open_cl_std_op_cosh instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_cospi(Open_cl_std_op_cospi instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_erfc(Open_cl_std_op_erfc instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_erf(Open_cl_std_op_erf instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_exp(Open_cl_std_op_exp instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_exp2(Open_cl_std_op_exp2 instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_exp10(Open_cl_std_op_exp10 instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_expm1(Open_cl_std_op_expm1 instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fabs(Open_cl_std_op_fabs instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fdim(Open_cl_std_op_fdim instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_floor(Open_cl_std_op_floor instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fma(Open_cl_std_op_fma instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fmax(Open_cl_std_op_fmax instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fmin(Open_cl_std_op_fmin instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fmod(Open_cl_std_op_fmod instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fract(Open_cl_std_op_fract instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_frexp(Open_cl_std_op_frexp instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_hypot(Open_cl_std_op_hypot instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_ilogb(Open_cl_std_op_ilogb instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_ldexp(Open_cl_std_op_ldexp instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_lgamma(Open_cl_std_op_lgamma instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_lgamma_r(Open_cl_std_op_lgamma_r instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_log(Open_cl_std_op_log instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_log2(Open_cl_std_op_log2 instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_log10(Open_cl_std_op_log10 instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_log1p(Open_cl_std_op_log1p instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_logb(Open_cl_std_op_logb instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_mad(Open_cl_std_op_mad instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_maxmag(Open_cl_std_op_maxmag instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_minmag(Open_cl_std_op_minmag instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_modf(Open_cl_std_op_modf instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_nan(Open_cl_std_op_nan instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_nextafter(
    Open_cl_std_op_nextafter instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_pow(Open_cl_std_op_pow instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_pown(Open_cl_std_op_pown instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_powr(Open_cl_std_op_powr instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_remainder(
    Open_cl_std_op_remainder instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_remquo(Open_cl_std_op_remquo instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_rint(Open_cl_std_op_rint instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_rootn(Open_cl_std_op_rootn instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_round(Open_cl_std_op_round instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_rsqrt(Open_cl_std_op_rsqrt instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_sin(Open_cl_std_op_sin instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_sincos(Open_cl_std_op_sincos instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_sinh(Open_cl_std_op_sinh instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_sinpi(Open_cl_std_op_sinpi instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_sqrt(Open_cl_std_op_sqrt instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_tan(Open_cl_std_op_tan instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_tanh(Open_cl_std_op_tanh instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_tanpi(Open_cl_std_op_tanpi instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_tgamma(Open_cl_std_op_tgamma instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_trunc(Open_cl_std_op_trunc instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_cos(Open_cl_std_op_half_cos instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_divide(
    Open_cl_std_op_half_divide instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_exp(Open_cl_std_op_half_exp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_exp2(
    Open_cl_std_op_half_exp2 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_exp10(
    Open_cl_std_op_half_exp10 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_log(Open_cl_std_op_half_log instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_log2(
    Open_cl_std_op_half_log2 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_log10(
    Open_cl_std_op_half_log10 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_powr(
    Open_cl_std_op_half_powr instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_recip(
    Open_cl_std_op_half_recip instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_rsqrt(
    Open_cl_std_op_half_rsqrt instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_sin(Open_cl_std_op_half_sin instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_sqrt(
    Open_cl_std_op_half_sqrt instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_half_tan(Open_cl_std_op_half_tan instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_cos(
    Open_cl_std_op_native_cos instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_divide(
    Open_cl_std_op_native_divide instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_exp(
    Open_cl_std_op_native_exp instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_exp2(
    Open_cl_std_op_native_exp2 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_exp10(
    Open_cl_std_op_native_exp10 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_log(
    Open_cl_std_op_native_log instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_log2(
    Open_cl_std_op_native_log2 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_log10(
    Open_cl_std_op_native_log10 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_powr(
    Open_cl_std_op_native_powr instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_recip(
    Open_cl_std_op_native_recip instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_rsqrt(
    Open_cl_std_op_native_rsqrt instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_sin(
    Open_cl_std_op_native_sin instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_sqrt(
    Open_cl_std_op_native_sqrt instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_native_tan(
    Open_cl_std_op_native_tan instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_abs(Open_cl_std_op_s_abs instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_abs_diff(
    Open_cl_std_op_s_abs_diff instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_add_sat(
    Open_cl_std_op_s_add_sat instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_add_sat(
    Open_cl_std_op_u_add_sat instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_hadd(Open_cl_std_op_s_hadd instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_hadd(Open_cl_std_op_u_hadd instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_rhadd(Open_cl_std_op_s_rhadd instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_rhadd(Open_cl_std_op_u_rhadd instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_clamp(Open_cl_std_op_s_clamp instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_clamp(Open_cl_std_op_u_clamp instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_clz(Open_cl_std_op_clz instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_ctz(Open_cl_std_op_ctz instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_mad_hi(Open_cl_std_op_s_mad_hi instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_mad_sat(
    Open_cl_std_op_u_mad_sat instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_mad_sat(
    Open_cl_std_op_s_mad_sat instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_max(Open_cl_std_op_s_max instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_max(Open_cl_std_op_u_max instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_min(Open_cl_std_op_s_min instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_min(Open_cl_std_op_u_min instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_mul_hi(Open_cl_std_op_s_mul_hi instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_rotate(Open_cl_std_op_rotate instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_sub_sat(
    Open_cl_std_op_s_sub_sat instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_sub_sat(
    Open_cl_std_op_u_sub_sat instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_upsample(
    Open_cl_std_op_u_upsample instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_upsample(
    Open_cl_std_op_s_upsample instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_popcount(Open_cl_std_op_popcount instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_mad24(Open_cl_std_op_s_mad24 instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_mad24(Open_cl_std_op_u_mad24 instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_s_mul24(Open_cl_std_op_s_mul24 instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_mul24(Open_cl_std_op_u_mul24 instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_abs(Open_cl_std_op_u_abs instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_abs_diff(
    Open_cl_std_op_u_abs_diff instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_mul_hi(Open_cl_std_op_u_mul_hi instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_u_mad_hi(Open_cl_std_op_u_mad_hi instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fclamp(Open_cl_std_op_fclamp instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_degrees(Open_cl_std_op_degrees instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fmax_common(
    Open_cl_std_op_fmax_common instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fmin_common(
    Open_cl_std_op_fmin_common instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_mix(Open_cl_std_op_mix instruction,
                                                          std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_radians(Open_cl_std_op_radians instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_step(Open_cl_std_op_step instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_smoothstep(
    Open_cl_std_op_smoothstep instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_sign(Open_cl_std_op_sign instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_cross(Open_cl_std_op_cross instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_distance(Open_cl_std_op_distance instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_length(Open_cl_std_op_length instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_normalize(
    Open_cl_std_op_normalize instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fast_distance(
    Open_cl_std_op_fast_distance instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fast_length(
    Open_cl_std_op_fast_length instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_fast_normalize(
    Open_cl_std_op_fast_normalize instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_bitselect(
    Open_cl_std_op_bitselect instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_select(Open_cl_std_op_select instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vloadn(Open_cl_std_op_vloadn instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vstoren(Open_cl_std_op_vstoren instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vload_half(
    Open_cl_std_op_vload_half instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vload_halfn(
    Open_cl_std_op_vload_halfn instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vstore_half(
    Open_cl_std_op_vstore_half instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vstore_half_r(
    Open_cl_std_op_vstore_half_r instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vstore_halfn(
    Open_cl_std_op_vstore_halfn instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vstore_halfn_r(
    Open_cl_std_op_vstore_halfn_r instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vloada_halfn(
    Open_cl_std_op_vloada_halfn instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vstorea_halfn(
    Open_cl_std_op_vstorea_halfn instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_vstorea_halfn_r(
    Open_cl_std_op_vstorea_halfn_r instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_shuffle(Open_cl_std_op_shuffle instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_shuffle2(Open_cl_std_op_shuffle2 instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_printf(Open_cl_std_op_printf instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_open_cl_std_op_prefetch(Open_cl_std_op_prefetch instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_round(Glsl_std_450_op_round instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_round_even(
    Glsl_std_450_op_round_even instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_trunc(Glsl_std_450_op_trunc instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_abs(Glsl_std_450_op_f_abs instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_abs(Glsl_std_450_op_s_abs instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_sign(Glsl_std_450_op_f_sign instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_sign(Glsl_std_450_op_s_sign instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_floor(Glsl_std_450_op_floor instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_ceil(Glsl_std_450_op_ceil instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_fract(Glsl_std_450_op_fract instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_radians(Glsl_std_450_op_radians instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_degrees(Glsl_std_450_op_degrees instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_sin(Glsl_std_450_op_sin instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_cos(Glsl_std_450_op_cos instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_tan(Glsl_std_450_op_tan instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_asin(Glsl_std_450_op_asin instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_acos(Glsl_std_450_op_acos instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_atan(Glsl_std_450_op_atan instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_sinh(Glsl_std_450_op_sinh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_cosh(Glsl_std_450_op_cosh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_tanh(Glsl_std_450_op_tanh instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_asinh(Glsl_std_450_op_asinh instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_acosh(Glsl_std_450_op_acosh instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_atanh(Glsl_std_450_op_atanh instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_atan2(Glsl_std_450_op_atan2 instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pow(Glsl_std_450_op_pow instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_exp(Glsl_std_450_op_exp instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_log(Glsl_std_450_op_log instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_exp2(Glsl_std_450_op_exp2 instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_log2(Glsl_std_450_op_log2 instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_sqrt(Glsl_std_450_op_sqrt instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_inverse_sqrt(
    Glsl_std_450_op_inverse_sqrt instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_determinant(
    Glsl_std_450_op_determinant instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_matrix_inverse(
    Glsl_std_450_op_matrix_inverse instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_modf(Glsl_std_450_op_modf instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_modf_struct(
    Glsl_std_450_op_modf_struct instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_min(Glsl_std_450_op_f_min instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_u_min(Glsl_std_450_op_u_min instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_min(Glsl_std_450_op_s_min instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_max(Glsl_std_450_op_f_max instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_u_max(Glsl_std_450_op_u_max instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_max(Glsl_std_450_op_s_max instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_clamp(Glsl_std_450_op_f_clamp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_u_clamp(Glsl_std_450_op_u_clamp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_s_clamp(Glsl_std_450_op_s_clamp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_f_mix(Glsl_std_450_op_f_mix instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_i_mix(Glsl_std_450_op_i_mix instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_step(Glsl_std_450_op_step instruction,
                                                            std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_smooth_step(
    Glsl_std_450_op_smooth_step instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_fma(Glsl_std_450_op_fma instruction,
                                                           std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_frexp(Glsl_std_450_op_frexp instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_frexp_struct(
    Glsl_std_450_op_frexp_struct instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_ldexp(Glsl_std_450_op_ldexp instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_snorm4x8(
    Glsl_std_450_op_pack_snorm4x8 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_unorm4x8(
    Glsl_std_450_op_pack_unorm4x8 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_snorm2x16(
    Glsl_std_450_op_pack_snorm2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_unorm2x16(
    Glsl_std_450_op_pack_unorm2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_half2x16(
    Glsl_std_450_op_pack_half2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_pack_double2x32(
    Glsl_std_450_op_pack_double2x32 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_snorm2x16(
    Glsl_std_450_op_unpack_snorm2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_unorm2x16(
    Glsl_std_450_op_unpack_unorm2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_half2x16(
    Glsl_std_450_op_unpack_half2x16 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_snorm4x8(
    Glsl_std_450_op_unpack_snorm4x8 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_unorm4x8(
    Glsl_std_450_op_unpack_unorm4x8 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_unpack_double2x32(
    Glsl_std_450_op_unpack_double2x32 instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_length(Glsl_std_450_op_length instruction,
                                                              std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_distance(
    Glsl_std_450_op_distance instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_cross(Glsl_std_450_op_cross instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_normalize(
    Glsl_std_450_op_normalize instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_face_forward(
    Glsl_std_450_op_face_forward instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_reflect(Glsl_std_450_op_reflect instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_refract(Glsl_std_450_op_refract instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_find_i_lsb(
    Glsl_std_450_op_find_i_lsb instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_find_s_msb(
    Glsl_std_450_op_find_s_msb instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_find_u_msb(
    Glsl_std_450_op_find_u_msb instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_interpolate_at_centroid(
    Glsl_std_450_op_interpolate_at_centroid instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_interpolate_at_sample(
    Glsl_std_450_op_interpolate_at_sample instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_interpolate_at_offset(
    Glsl_std_450_op_interpolate_at_offset instruction, std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_n_min(Glsl_std_450_op_n_min instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_n_max(Glsl_std_450_op_n_max instruction,
                                                             std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

void Spirv_to_llvm::handle_instruction_glsl_std_450_op_n_clamp(Glsl_std_450_op_n_clamp instruction,
                                                               std::size_t instruction_start_index)
{
#warning finish
    throw Parser_error(instruction_start_index,
                       instruction_start_index,
                       "instruction not implemented: "
                           + std::string(get_enumerant_name(instruction.get_operation())));
}

Converted_module spirv_to_llvm(::LLVMContextRef context,
                               const Word *shader_words,
                               std::size_t shader_size)
{
    return Spirv_to_llvm(context).run(shader_words, shader_size);
}
}
}
