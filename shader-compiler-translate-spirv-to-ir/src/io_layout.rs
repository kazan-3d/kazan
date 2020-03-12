// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use shader_compiler_ir::Alignment;

pub(crate) const COMPONENT_SIZE_IN_BYTES: u32 = 4;
pub(crate) const LOCATION_SIZE_IN_COMPONENTS: u32 = 4;
pub(crate) const LOCATION_SIZE_IN_BYTES: u32 =
    COMPONENT_SIZE_IN_BYTES * LOCATION_SIZE_IN_COMPONENTS;

pub(crate) fn io_interface_block_alignment() -> Alignment {
    Alignment::new(LOCATION_SIZE_IN_BYTES).expect("known to be a valid alignment")
}
