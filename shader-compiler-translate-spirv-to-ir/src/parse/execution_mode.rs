// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::DuplicateSPIRVLocalSize;
use crate::errors::UnsupportedSPIRVExecutionMode;
use crate::parse::entry_point::TranslationStateParsedEntryPoints;
use crate::parse::ParseInstruction;
use crate::TranslationResult;
use core::mem;
use spirv_parser::ExecutionMode;
use spirv_parser::ExecutionModeLocalSize;
use spirv_parser::ExecutionModeLocalSizeId;
use spirv_parser::IdRef;
use spirv_parser::Instruction;
use spirv_parser::OpExecutionMode;
use spirv_parser::OpExecutionModeId;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum FragmentTestsTime {
    Early,
    Late,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum FragmentDepthWrite {
    Equal,
    Less,
    Greater,
    Unconstrained,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum ComputeLocalSize {
    Literal { x: u32, y: u32, z: u32 },
    Id { x: IdRef, y: IdRef, z: IdRef },
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsedExecutionModes<'g, 'i> {
        base: TranslationStateParsedEntryPoints<'g, 'i>,
        fragment_tests_time: FragmentTestsTime,
        fragment_depth_write: Option<FragmentDepthWrite>,
        compute_local_size: Option<ComputeLocalSize>,
    }
}

impl<'g, 'i> TranslationStateParsedExecutionModes<'g, 'i> {
    fn set_compute_local_size(
        &mut self,
        compute_local_size: ComputeLocalSize,
    ) -> TranslationResult<()> {
        match mem::replace(&mut self.compute_local_size, Some(compute_local_size)) {
            None => Ok(()),
            Some(_) => Err(DuplicateSPIRVLocalSize.into()),
        }
    }
    fn parse_execution_mode_instruction(
        &mut self,
        entry_point_id: IdRef,
        execution_mode: &'i ExecutionMode,
    ) -> TranslationResult<()> {
        if entry_point_id != self.entry_point_id {
            return Ok(());
        }
        match *execution_mode {
            // requires Geometry
            ExecutionMode::Invocations(_)
            | ExecutionMode::InputPoints(_)
            | ExecutionMode::InputLines(_)
            | ExecutionMode::InputLinesAdjacency(_)
            | ExecutionMode::InputTrianglesAdjacency(_)
            | ExecutionMode::OutputPoints(_)
            | ExecutionMode::OutputLineStrip(_)
            | ExecutionMode::OutputTriangleStrip(_) => Err(UnsupportedSPIRVExecutionMode {
                execution_mode: execution_mode.clone(),
            }
            .into()),
            // requires Tessellation
            ExecutionMode::SpacingEqual(_)
            | ExecutionMode::SpacingFractionalEven(_)
            | ExecutionMode::SpacingFractionalOdd(_)
            | ExecutionMode::VertexOrderCw(_)
            | ExecutionMode::VertexOrderCcw(_)
            | ExecutionMode::PointMode(_)
            | ExecutionMode::Quads(_)
            | ExecutionMode::Isolines(_) => Err(UnsupportedSPIRVExecutionMode {
                execution_mode: execution_mode.clone(),
            }
            .into()),
            // requires Geometry or Tessellation
            ExecutionMode::Triangles(_) | ExecutionMode::OutputVertices(_) => {
                Err(UnsupportedSPIRVExecutionMode {
                    execution_mode: execution_mode.clone(),
                }
                .into())
            }
            // requires TransformFeedback
            ExecutionMode::Xfb(_) => Err(UnsupportedSPIRVExecutionMode {
                execution_mode: execution_mode.clone(),
            }
            .into()),
            // requires SPV_KHR_float_controls
            ExecutionMode::DenormPreserve(_)
            | ExecutionMode::DenormFlushToZero(_)
            | ExecutionMode::SignedZeroInfNanPreserve(_)
            | ExecutionMode::RoundingModeRTE(_)
            | ExecutionMode::RoundingModeRTZ(_) => Err(UnsupportedSPIRVExecutionMode {
                execution_mode: execution_mode.clone(),
            }
            .into()),
            // not supported on Vulkan
            ExecutionMode::PixelCenterInteger(_)
            | ExecutionMode::OriginLowerLeft(_)
            | ExecutionMode::LocalSizeHint(_)
            | ExecutionMode::VecTypeHint(_)
            | ExecutionMode::ContractionOff(_)
            | ExecutionMode::Initializer(_)
            | ExecutionMode::Finalizer(_)
            | ExecutionMode::SubgroupSize(_)
            | ExecutionMode::SubgroupsPerWorkgroup(_)
            | ExecutionMode::SubgroupsPerWorkgroupId(_)
            | ExecutionMode::LocalSizeHintId(_) => Err(UnsupportedSPIRVExecutionMode {
                execution_mode: execution_mode.clone(),
            }
            .into()),
            // allowed
            ExecutionMode::OriginUpperLeft(_) => Ok(()),
            ExecutionMode::EarlyFragmentTests(_) => {
                self.fragment_tests_time = FragmentTestsTime::Early;
                Ok(())
            }
            ExecutionMode::DepthReplacing(_) => {
                if self.fragment_depth_write.is_none() {
                    self.fragment_depth_write = Some(FragmentDepthWrite::Unconstrained);
                }
                Ok(())
            }
            ExecutionMode::DepthGreater(_) => {
                self.fragment_depth_write = Some(FragmentDepthWrite::Greater);
                Ok(())
            }
            ExecutionMode::DepthLess(_) => {
                self.fragment_depth_write = Some(FragmentDepthWrite::Less);
                Ok(())
            }
            ExecutionMode::DepthUnchanged(_) => {
                self.fragment_depth_write = Some(FragmentDepthWrite::Equal);
                Ok(())
            }
            ExecutionMode::LocalSize(ExecutionModeLocalSize {
                x_size: x,
                y_size: y,
                z_size: z,
            }) => self.set_compute_local_size(ComputeLocalSize::Literal { x, y, z }),
            ExecutionMode::LocalSizeId(ExecutionModeLocalSizeId {
                x_size: x,
                y_size: y,
                z_size: z,
            }) => self.set_compute_local_size(ComputeLocalSize::Id { x, y, z }),
        }
    }
}

impl<'g, 'i> TranslationStateParsedEntryPoints<'g, 'i> {
    pub(crate) fn parse_execution_mode_section(
        self,
    ) -> TranslationResult<TranslationStateParsedExecutionModes<'g, 'i>> {
        let mut state = TranslationStateParsedExecutionModes {
            base: self,
            fragment_tests_time: FragmentTestsTime::Late,
            fragment_depth_write: None,
            compute_local_size: None,
        };
        writeln!(state.debug_output, "parsing OpExecutionMode section")?;
        while let Some((instruction, location)) = state.get_instruction_and_location()? {
            match *instruction {
                Instruction::ExecutionMode(OpExecutionMode {
                    entry_point,
                    ref mode,
                })
                | Instruction::ExecutionModeId(OpExecutionModeId {
                    entry_point,
                    ref mode,
                }) => state.parse_execution_mode_instruction(entry_point, mode)?,
                _ => {
                    state.spirv_instructions_location = location;
                    break;
                }
            }
        }
        Ok(state)
    }
}

impl ParseInstruction for OpExecutionMode {}
impl ParseInstruction for OpExecutionModeId {}
