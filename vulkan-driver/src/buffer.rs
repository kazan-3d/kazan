// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
pub struct BufferMemory {}

pub struct Buffer {
    pub size: usize,
    pub memory: Option<BufferMemory>,
}
