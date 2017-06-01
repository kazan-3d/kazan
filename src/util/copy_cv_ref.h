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

#ifndef SOURCE_UTIL_COPY_CV_REF_H_
#define SOURCE_UTIL_COPY_CV_REF_H_

namespace vulkan_cpu
{
namespace util
{
template <typename Source, typename Dest>
struct copy_const
{
    typedef Dest type;
};

template <typename Source, typename Dest>
struct copy_const<const Source, Dest>
{
    typedef const Dest type;
};

template <typename Source, typename Dest>
using copy_const_t = typename copy_const<Source, Dest>::type;

template <typename Source, typename Dest>
struct copy_volatile
{
    typedef Dest type;
};

template <typename Source, typename Dest>
struct copy_volatile<volatile Source, Dest>
{
    typedef volatile Dest type;
};

template <typename Source, typename Dest>
using copy_volatile_t = typename copy_volatile<Source, Dest>::type;

template <typename Source, typename Dest>
struct copy_cv
{
    typedef copy_const_t<Source, copy_volatile_t<Source, Dest>> type;
};

template <typename Source, typename Dest>
using copy_cv_t = typename copy_cv<Source, Dest>::type;

template <typename Source, typename Dest>
struct copy_ref
{
    typedef Dest type;
};

template <typename Source, typename Dest>
struct copy_ref<Source &, Dest>
{
    typedef Dest &type;
};

template <typename Source, typename Dest>
struct copy_ref<Source &&, Dest>
{
    typedef Dest &&type;
};

template <typename Source, typename Dest>
using copy_ref_t = typename copy_ref<Source, Dest>::type;

template <typename Source, typename Dest>
struct copy_cv_ref
{
    typedef copy_cv_t<Source, copy_ref_t<Source, Dest>> type;
};

template <typename Source, typename Dest>
using copy_cv_ref_t = typename copy_cv_ref<Source, Dest>::type;
}
}

#endif /* SOURCE_UTIL_COPY_CV_REF_H_ */
