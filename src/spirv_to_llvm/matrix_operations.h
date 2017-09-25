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
#ifndef SPIRV_TO_LLVM_MATRIX_OPERATIONS_H_
#define SPIRV_TO_LLVM_MATRIX_OPERATIONS_H_

#include "llvm_wrapper/llvm_wrapper.h"
#include "spirv_to_llvm/spirv_to_llvm.h"
#include <cstdint>
#include <vector>

namespace kazan
{
namespace spirv_to_llvm
{
namespace matrix_operations
{
struct Matrix_descriptor
{
    std::uint32_t rows;
    std::uint32_t columns;
    ::LLVMTypeRef column_type;
    ::LLVMTypeRef element_type;
    ::LLVMTypeRef matrix_type;
    explicit Matrix_descriptor(::LLVMTypeRef matrix_type) noexcept : matrix_type(matrix_type)
    {
        assert(::LLVMGetTypeKind(matrix_type) == ::LLVMArrayTypeKind);
        columns = ::LLVMGetArrayLength(matrix_type);
        column_type = ::LLVMGetElementType(matrix_type);
        assert(::LLVMGetTypeKind(column_type) == ::LLVMVectorTypeKind);
        rows = ::LLVMGetVectorSize(column_type);
        element_type = ::LLVMGetElementType(column_type);
    }
    Matrix_descriptor(::LLVMTypeRef element_type, std::uint32_t rows, std::uint32_t columns)
        : rows(rows),
          columns(columns),
          column_type(::LLVMVectorType(element_type, rows)),
          element_type(element_type),
          matrix_type(::LLVMArrayType(column_type, columns))
    {
    }
};

struct Vector_descriptor
{
    std::uint32_t element_count;
    ::LLVMTypeRef element_type;
    ::LLVMTypeRef vector_type;
    explicit Vector_descriptor(::LLVMTypeRef vector_type) noexcept : vector_type(vector_type)
    {
        assert(::LLVMGetTypeKind(vector_type) == ::LLVMVectorTypeKind);
        element_count = ::LLVMGetVectorSize(vector_type);
        element_type = ::LLVMGetElementType(vector_type);
    }
    Vector_descriptor(::LLVMTypeRef element_type, std::uint32_t element_count)
        : element_count(element_count),
          element_type(element_type),
          vector_type(::LLVMVectorType(element_type, element_count))
    {
    }
};

inline ::LLVMValueRef transpose(::LLVMContextRef context,
                                ::LLVMModuleRef module,
                                ::LLVMBuilderRef builder,
                                ::LLVMValueRef input_matrix,
                                const char *output_name)
{
    auto i32_type = llvm_wrapper::Create_llvm_type<std::uint32_t>()(context);
    Matrix_descriptor input_matrix_descriptor(::LLVMTypeOf(input_matrix));
    Matrix_descriptor output_matrix_descriptor(input_matrix_descriptor.element_type,
                                               input_matrix_descriptor.columns,
                                               input_matrix_descriptor.rows);
    std::vector<::LLVMValueRef> input_columns;
    input_columns.reserve(input_matrix_descriptor.columns);
    for(std::uint32_t input_column = 0; input_column < input_matrix_descriptor.columns;
        input_column++)
        input_columns.push_back(::LLVMBuildExtractValue(builder, input_matrix, input_column, ""));
    auto output_value = ::LLVMGetUndef(output_matrix_descriptor.matrix_type);
    for(std::uint32_t output_column = 0; output_column < output_matrix_descriptor.columns;
        output_column++)
    {
        auto output_column_value = ::LLVMGetUndef(output_matrix_descriptor.column_type);
        for(std::uint32_t output_row = 0; output_row < output_matrix_descriptor.rows; output_row++)
        {
            auto element_value =
                ::LLVMBuildExtractElement(builder,
                                          input_columns[output_row],
                                          ::LLVMConstInt(i32_type, output_column, false),
                                          "");
            output_column_value =
                ::LLVMBuildInsertElement(builder,
                                         output_column_value,
                                         element_value,
                                         ::LLVMConstInt(i32_type, output_row, false),
                                         "");
        }
        output_value =
            ::LLVMBuildInsertValue(builder, output_value, output_column_value, output_column, "");
    }
    ::LLVMSetValueName(output_value, output_name);
    return output_value;
}

inline ::LLVMValueRef vector_broadcast_from_vector(::LLVMContextRef context,
                                                   ::LLVMBuilderRef builder,
                                                   ::LLVMValueRef input_vector,
                                                   std::uint32_t input_vector_index,
                                                   std::uint32_t output_vector_length,
                                                   const char *output_name)
{
    auto i32_type = llvm_wrapper::Create_llvm_type<std::uint32_t>()(context);
    auto index = ::LLVMConstInt(i32_type, input_vector_index, false);
    std::vector<::LLVMValueRef> shuffle_arguments(output_vector_length, index);
    auto shuffle_index_vector =
        ::LLVMConstVector(shuffle_arguments.data(), shuffle_arguments.size());
    return ::LLVMBuildShuffleVector(builder,
                                    input_vector,
                                    ::LLVMGetUndef(::LLVMTypeOf(input_vector)),
                                    shuffle_index_vector,
                                    output_name);
}

inline ::LLVMValueRef matrix_multiply(::LLVMContextRef context,
                                      ::LLVMModuleRef module,
                                      ::LLVMBuilderRef builder,
                                      ::LLVMValueRef left_matrix,
                                      ::LLVMValueRef right_matrix,
                                      const char *output_name)
{
    Matrix_descriptor left_matrix_descriptor(::LLVMTypeOf(left_matrix));
    Matrix_descriptor right_matrix_descriptor(::LLVMTypeOf(right_matrix));
    assert(left_matrix_descriptor.element_type == right_matrix_descriptor.element_type);
    assert(left_matrix_descriptor.columns == right_matrix_descriptor.rows);
    assert(left_matrix_descriptor.columns != 0);
    assert(left_matrix_descriptor.rows != 0);
    assert(right_matrix_descriptor.columns != 0);
    Matrix_descriptor result_matrix_descriptor(left_matrix_descriptor.element_type,
                                               left_matrix_descriptor.rows,
                                               right_matrix_descriptor.columns);
    ::LLVMValueRef retval = ::LLVMGetUndef(result_matrix_descriptor.matrix_type);
    for(std::size_t i = 0; i < right_matrix_descriptor.columns; i++)
    {
        ::LLVMValueRef right_matrix_column = ::LLVMBuildExtractValue(builder, right_matrix, i, "");
        ::LLVMValueRef sum{};
        for(std::size_t j = 0; j < left_matrix_descriptor.columns; j++)
        {
            auto factor0 = ::LLVMBuildExtractValue(builder, left_matrix, j, "");
            auto factor1 = vector_broadcast_from_vector(
                context, builder, right_matrix_column, j, left_matrix_descriptor.rows, "");
            if(j == 0)
                sum = ::LLVMBuildFMul(builder, factor0, factor1, "");
            else
                sum = llvm_wrapper::Builder::build_fmuladd(
                    builder, module, factor0, factor1, sum, "");
        }
        retval = ::LLVMBuildInsertValue(builder, retval, sum, i, "");
    }
    ::LLVMSetValueName(retval, output_name);
    return retval;
}

inline ::LLVMValueRef matrix_times_vector(::LLVMContextRef context,
                                          ::LLVMModuleRef module,
                                          ::LLVMBuilderRef builder,
                                          ::LLVMValueRef matrix,
                                          ::LLVMValueRef input_vector,
                                          const char *output_name)
{
    Matrix_descriptor matrix_descriptor(::LLVMTypeOf(matrix));
    Vector_descriptor input_vector_descriptor(::LLVMTypeOf(input_vector));
    assert(matrix_descriptor.element_type == input_vector_descriptor.element_type);
    assert(matrix_descriptor.columns == input_vector_descriptor.element_count);
    assert(matrix_descriptor.columns != 0);
    ::LLVMValueRef retval{};
    for(std::size_t i = 0; i < matrix_descriptor.columns; i++)
    {
        auto factor0 = ::LLVMBuildExtractValue(builder, matrix, i, "");
        auto factor1 = vector_broadcast_from_vector(
            context, builder, input_vector, i, matrix_descriptor.rows, "");
        if(i == 0)
            retval = ::LLVMBuildFMul(builder, factor0, factor1, "");
        else
            retval =
                llvm_wrapper::Builder::build_fmuladd(builder, module, factor0, factor1, retval, "");
    }
    ::LLVMSetValueName(retval, output_name);
    return retval;
}
}
}
}

#endif // SPIRV_TO_LLVM_MATRIX_OPERATIONS_H_
