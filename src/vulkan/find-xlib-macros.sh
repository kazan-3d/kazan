#!/bin/bash
# Copyright 2017 Jacob Lifshay
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

printf "#include <X11/Xlib.h>\n#include <X11/Xlib-xcb.h>\n" |
clang++ -std=c++14 -E -dD -x c++ -o - - |
grep '^#' |
{
    mapfile -t lines
    filename=""
    for line in "${lines[@]}"; do
        if [[ "$line" =~ ^'# '[0-9]+' "'([^\"]*)'"'.* ]]; then # line number indicator
            filename="${BASH_REMATCH[1]}"
        elif [[ "$line" =~ ^'#define '([a-zA-Z0-9_]+) ]]; then
            macro="${BASH_REMATCH[1]}"
            if [[ ! "$filename" =~ ^'/usr/include/X11' || "$macro" =~ ^'_'[A-Z] || "$macro" =~ '__' || "$macro" =~ '_H'$ ]]; then
                continue
            fi
            echo "$macro"
        fi
    done
} |
sort |
sed 's/\(.*\)/#ifdef \1\n#undef \1\n#endif/'
