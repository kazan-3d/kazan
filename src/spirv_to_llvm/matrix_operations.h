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

inline ::LLVMValueRef transpose(::LLVMContextRef context,
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
}
}
}

#endif // SPIRV_TO_LLVM_MATRIX_OPERATIONS_H_
