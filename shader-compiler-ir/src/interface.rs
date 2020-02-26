// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    text::{
        FromText, FromTextError, FromTextState, FromToTextListForm, Keyword, ListForm, Punctuation,
        ToText, ToTextState,
    },
    Alignment, DataPointerType, StructMember, StructSize, StructType, ValueDefinition, Variable,
};
use alloc::vec::Vec;
use core::{fmt, iter, slice};
use iter::FusedIterator;

macro_rules! impl_built_in_kind {
    (
        $vis:vis enum BuiltInKind {
            $(
                $(#[doc = $doc:expr])+
                #[text = $text:literal]
                $name:ident,
            )+
        }
    ) => {
        /// the kind of a built-in shader input/output
        #[derive(Copy, Clone, Eq, PartialEq, Hash)]
        $vis enum BuiltInKind {
            $(
                $(#[doc = $doc])+
                $name,
            )+
        }

        impl_display_as_to_text!(BuiltInKind);

        impl FromToTextListForm for BuiltInKind {}

        impl<'g> FromText<'g> for BuiltInKind {
            type Parsed = Self;
            fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
                let retval = match state.peek_token()?.kind.raw_identifier() {
                    $(Some($text) => Self::$name,)+
                    _ => state.error_at_peek_token("expected built-in kind")?.into(),
                };
                state.parse_token()?;
                Ok(retval)
            }
        }

        impl<'g> ToText<'g> for BuiltInKind {
            fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                let text: &'static str = match self {
                    $(Self::$name => $text,)+
                };
                write!(state, "{}", text)
            }
        }
    };
}

impl_built_in_kind! {
    pub enum BuiltInKind {
        /// Vertex Position
        #[text = "vertex_position"]
        VertexPosition,
    }
}

/// an interface variable -- a single variable used for an input/output for a shader
pub struct InterfaceVariable<'g, Attributes: 'g + FromText<'g, Parsed = Attributes> + ToText<'g>> {
    /// the variable
    pub variable: Variable<'g>,
    /// the attributes
    pub attributes: Attributes,
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> fmt::Display
    for InterfaceVariable<'g, Attributes>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> fmt::Debug
    for InterfaceVariable<'g, Attributes>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> FromToTextListForm
    for InterfaceVariable<'g, Attributes>
{
    fn from_to_text_list_form() -> ListForm {
        ListForm::STATEMENTS
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> ToText<'g>
    for InterfaceVariable<'g, Attributes>
{
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let InterfaceVariable {
            variable,
            attributes,
        } = self;
        variable.to_text(state)?;
        write!(state, ": ")?;
        attributes.to_text(state)
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> FromText<'g>
    for InterfaceVariable<'g, Attributes>
{
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
        let variable = Variable::from_text(state)?;
        state.parse_punct_token_or_error(
            Punctuation::Colon,
            "missing colon (`:`) before interface variable attributes",
        )?;
        let attributes = Attributes::from_text(state)?;
        Ok(InterfaceVariable {
            variable,
            attributes,
        })
    }
}

/// an interface block -- a block of variables used for inputs/outputs for a shader
pub struct InterfaceBlock<'g, Attributes: 'g + FromText<'g, Parsed = Attributes> + ToText<'g>> {
    /// the type of the block
    block_type: StructType<'g>,
    /// the pointer to this block
    pointer: ValueDefinition<'g>,
    /// the attributes for each member variable
    variable_attributes: Vec<Attributes>,
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> fmt::Display
    for InterfaceBlock<'g, Attributes>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> fmt::Debug
    for InterfaceBlock<'g, Attributes>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> FromToTextListForm
    for InterfaceBlock<'g, Attributes>
{
    fn from_to_text_list_form() -> ListForm {
        ListForm::STATEMENTS
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> ToText<'g>
    for InterfaceBlock<'g, Attributes>
{
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let InterfaceBlock {
            block_type:
                StructType {
                    size,
                    alignment,
                    members: _members,
                },
            pointer,
            variable_attributes: _variable_attributes,
        } = self;
        ListForm::STATEMENTS.list_to_text_with_extra_callbacks(
            state,
            |state| {
                write!(state, "-> ")?;
                pointer.to_text(state)?;
                writeln!(state, ";")?;
                write!(state, "size: ")?;
                size.to_text(state)?;
                writeln!(state, ";")?;
                alignment.to_text(state)?;
                writeln!(state, ";")
            },
            |state, item| item.to_text(state),
            self.members(),
        )
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> FromText<'g>
    for InterfaceBlock<'g, Attributes>
{
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
        let mut members = Vec::new();
        let (pointer, size, alignment) = ListForm::STATEMENTS.parse_list_with_extra_callbacks(
            state,
            |state| {
                state.parse_punct_token_or_error(
                    Punctuation::Arrow,
                    "missing arrow (`->`) before `pointer` value definition",
                )?;
                let pointer_location = state.peek_token()?.span;
                let pointer = ValueDefinition::from_text(state)?;
                if *pointer.value_type != DataPointerType.into() {
                    state.error_at(
                        pointer_location,
                        "pointer value definition must have type `data_ptr`",
                    )?;
                }
                state.parse_punct_token_or_error(
                    Punctuation::Semicolon,
                    "missing semicolon (`;`) after `pointer` value definition",
                )?;
                state.parse_keyword_token_or_error(
                    Keyword::Size,
                    "missing interface block struct size keyword (`size`)",
                )?;
                state.parse_punct_token_or_error(
                    Punctuation::Colon,
                    "missing colon between struct size keyword and struct size",
                )?;
                let size = StructSize::from_text(state)?;
                state.parse_punct_token_or_error(
                    Punctuation::Semicolon,
                    "missing semicolon (`;`) after interface block struct size",
                )?;
                let alignment = Alignment::from_text(state)?;
                state.parse_punct_token_or_error(
                    Punctuation::Semicolon,
                    "missing semicolon (`;`) after interface block struct alignment",
                )?;
                Ok((pointer, size, alignment))
            },
            |state| {
                members.push(InterfaceBlockMember::<Attributes>::from_text(state)?);
                Ok(())
            },
        )?;
        Ok(Self::new(pointer, size, alignment, members))
    }
}

/// a member variable in an interface block
pub struct InterfaceBlockMember<'g, Attributes: 'g + FromText<'g, Parsed = Attributes> + ToText<'g>>
{
    /// the struct member for this member variable
    pub struct_member: StructMember<'g>,
    /// the attributes for this member variable
    pub attributes: Attributes,
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> FromToTextListForm
    for InterfaceBlockMember<'g, Attributes>
{
    fn from_to_text_list_form() -> ListForm {
        ListForm::STATEMENTS
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> FromText<'g>
    for InterfaceBlockMember<'g, Attributes>
{
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
        let struct_member = StructMember::from_text(state)?;
        state.parse_punct_token_or_error(
            Punctuation::Colon,
            "missing colon (`:`) before interface block member attributes",
        )?;
        let attributes = Attributes::from_text(state)?;
        Ok(InterfaceBlockMember {
            struct_member,
            attributes,
        })
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> ToText<'g>
    for InterfaceBlockMember<'g, Attributes>
{
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        InterfaceBlockMemberRef::from(self).to_text(state)
    }
}

/// a reference to a member variable in an interface block
pub struct InterfaceBlockMemberRef<
    'a,
    'g,
    Attributes: 'g + FromText<'g, Parsed = Attributes> + ToText<'g>,
> {
    /// the struct member for this member variable
    pub struct_member: &'a StructMember<'g>,
    /// the attributes for this member variable
    pub attributes: &'a Attributes,
}

impl<'g, Attributes: 'g + FromText<'g, Parsed = Attributes> + ToText<'g>> FromToTextListForm
    for InterfaceBlockMemberRef<'_, 'g, Attributes>
{
    fn from_to_text_list_form() -> ListForm {
        InterfaceBlockMember::<'g, Attributes>::from_to_text_list_form()
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> FromText<'g>
    for InterfaceBlockMemberRef<'_, 'g, Attributes>
{
    type Parsed = InterfaceBlockMember<'g, Attributes>;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
        InterfaceBlockMember::from_text(state)
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> ToText<'g>
    for InterfaceBlockMemberRef<'_, 'g, Attributes>
{
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let InterfaceBlockMemberRef {
            struct_member,
            attributes,
        } = *self;
        struct_member.to_text(state)?;
        write!(state, ": ")?;
        attributes.to_text(state)
    }
}

impl<'a, 'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>>
    From<&'a InterfaceBlockMember<'g, Attributes>> for InterfaceBlockMemberRef<'a, 'g, Attributes>
{
    fn from(v: &'a InterfaceBlockMember<'g, Attributes>) -> Self {
        let InterfaceBlockMember {
            struct_member,
            attributes,
        } = v;
        InterfaceBlockMemberRef {
            struct_member,
            attributes,
        }
    }
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> Copy
    for InterfaceBlockMemberRef<'_, 'g, Attributes>
{
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> Clone
    for InterfaceBlockMemberRef<'_, 'g, Attributes>
{
    fn clone(&self) -> Self {
        *self
    }
}

/// the iterator type retuned by `InterfaceBlock::members()`
#[derive(Debug, Clone)]
pub struct InterfaceBlockMembers<
    'a,
    'g,
    Attributes: 'g + FromText<'g, Parsed = Attributes> + ToText<'g>,
>(iter::Zip<slice::Iter<'a, StructMember<'g>>, slice::Iter<'a, Attributes>>);

impl<'a, 'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> Iterator
    for InterfaceBlockMembers<'a, 'g, Attributes>
{
    type Item = InterfaceBlockMemberRef<'a, 'g, Attributes>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|(struct_member, attributes)| InterfaceBlockMemberRef {
                struct_member,
                attributes,
            })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0
            .nth(n)
            .map(|(struct_member, attributes)| InterfaceBlockMemberRef {
                struct_member,
                attributes,
            })
    }
}

impl<'a, 'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> DoubleEndedIterator
    for InterfaceBlockMembers<'a, 'g, Attributes>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .map(|(struct_member, attributes)| InterfaceBlockMemberRef {
                struct_member,
                attributes,
            })
    }
}

impl<'a, 'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> ExactSizeIterator
    for InterfaceBlockMembers<'a, 'g, Attributes>
{
}

impl<'a, 'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>> FusedIterator
    for InterfaceBlockMembers<'a, 'g, Attributes>
{
}

impl<'g, Attributes: FromText<'g, Parsed = Attributes> + ToText<'g>>
    InterfaceBlock<'g, Attributes>
{
    /// the type of the block
    pub fn block_type(&self) -> &StructType<'g> {
        &self.block_type
    }
    /// the pointer to this block
    pub fn pointer(&self) -> &ValueDefinition<'g> {
        &self.pointer
    }
    /// the attributes for each member variable
    pub fn variable_attributes(&self) -> &[Attributes] {
        &self.variable_attributes
    }
    /// the member variables
    pub fn members<'a>(&'a self) -> InterfaceBlockMembers<'a, 'g, Attributes> {
        InterfaceBlockMembers(
            self.block_type
                .members
                .iter()
                .zip(&self.variable_attributes),
        )
    }
    /// create a new `InterfaceBlock`
    pub fn new(
        pointer: ValueDefinition<'g>,
        size: StructSize,
        alignment: Alignment,
        members: impl IntoIterator<Item = InterfaceBlockMember<'g, Attributes>>,
    ) -> Self {
        assert_eq!(*pointer.value_type, DataPointerType.into());
        let (members, variable_attributes): (Vec<_>, Vec<_>) = members
            .into_iter()
            .map(
                |InterfaceBlockMember {
                     struct_member,
                     attributes,
                 }| (struct_member, attributes),
            )
            .unzip();
        assert_eq!(members.len(), variable_attributes.len());
        Self {
            block_type: StructType {
                size,
                alignment,
                members,
            },
            pointer,
            variable_attributes,
        }
    }
}

/// The attributes for a variable in a user interface block.
///
/// This is used for `Module::user_inputs_block` and `Module::user_outputs_block`
pub struct UserInterfaceVariableAttributes {}

impl_display_as_to_text!(UserInterfaceVariableAttributes);

impl FromToTextListForm for UserInterfaceVariableAttributes {
    fn from_to_text_list_form() -> ListForm {
        ListForm::STATEMENTS
    }
}

impl<'g> FromText<'g> for UserInterfaceVariableAttributes {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
        state.parse_parenthesized(
            Punctuation::LCurlyBrace,
            "missing opening curly brace (`{`)",
            Punctuation::RCurlyBrace,
            "missing closing curly brace (`}`)",
            |_| Ok(UserInterfaceVariableAttributes {}),
        )
    }
}

impl<'g> ToText<'g> for UserInterfaceVariableAttributes {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "{{}}")
    }
}

/// The attributes for a variable in a built-ins interface block.
///
/// This is used for `Module::built_in_inputs_block` and `Module::built_in_outputs_block`
pub struct BuiltInInterfaceVariableAttributes {
    /// The kind of built-in that this variable is
    pub kind: BuiltInKind,
}

impl_display_as_to_text!(BuiltInInterfaceVariableAttributes);

impl FromToTextListForm for BuiltInInterfaceVariableAttributes {
    fn from_to_text_list_form() -> ListForm {
        ListForm::STATEMENTS
    }
}

impl<'g> FromText<'g> for BuiltInInterfaceVariableAttributes {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
        state.parse_parenthesized(
            Punctuation::LCurlyBrace,
            "missing opening curly brace (`{`)",
            Punctuation::RCurlyBrace,
            "missing closing curly brace (`}`)",
            |state| {
                state.parse_keyword_token_or_error(Keyword::Kind, "missing `kind` keyword")?;
                state.parse_punct_token_or_error(
                    Punctuation::Colon,
                    "missing colon (`:`) after `kind` keyword",
                )?;
                let kind = BuiltInKind::from_text(state)?;
                Ok(BuiltInInterfaceVariableAttributes { kind })
            },
        )
    }
}

impl<'g> ToText<'g> for BuiltInInterfaceVariableAttributes {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let BuiltInInterfaceVariableAttributes { kind } = *self;
        write!(state, "{{ kind: ")?;
        kind.to_text(state)?;
        write!(state, " }}")
    }
}
