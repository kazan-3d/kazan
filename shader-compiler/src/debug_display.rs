// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::cfg::CFGGraph;
use std::collections::HashSet;
use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Copy, Clone, Debug)]
pub struct Indent(usize);

impl Indent {
    pub fn make_more(self) -> Self {
        Indent(self.0 + 1)
    }
}

impl Default for Indent {
    fn default() -> Self {
        Indent(0)
    }
}

impl Display for Indent {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, "    ")?;
        }
        Ok(())
    }
}

pub struct DebugToDisplayWrapper<T>(T);

impl<T: Display> Debug for DebugToDisplayWrapper<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: Display> Display for DebugToDisplayWrapper<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub struct HandleIsDebugWrapper<T: Debug + Display> {
    pub value: T,
    pub is_debug: Option<bool>,
}

impl<T: Debug + Display> HandleIsDebugWrapper<T> {
    fn fmt_with_default_is_debug(&self, f: &mut Formatter, default_is_debug: bool) -> fmt::Result {
        if self.is_debug.unwrap_or(default_is_debug) {
            Debug::fmt(&self.value, f)
        } else {
            Display::fmt(&self.value, f)
        }
    }
}

impl<T: Debug + Display> Debug for HandleIsDebugWrapper<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt_with_default_is_debug(f, true)
    }
}

impl<T: Debug + Display> Display for HandleIsDebugWrapper<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt_with_default_is_debug(f, false)
    }
}

pub trait DisplayWithCFG<'a> {
    type DisplayType: Debug + Display + 'a;
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType;
    fn display_with_cfg_and_indent(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
    ) -> Self::DisplayType {
        self.display_with_cfg_and_indent_and_is_debug(cfg, indent, None)
    }
    fn display_with_cfg_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        self.display_with_cfg_and_indent_and_is_debug(cfg, Indent::default(), is_debug)
    }
    fn display_with_cfg(&'a self, cfg: &'a CFGGraph) -> Self::DisplayType {
        self.display_with_cfg_and_indent_and_is_debug(cfg, Indent::default(), None)
    }
}

impl<'a, 'b, T: DisplayWithCFG<'a>> DisplayWithCFG<'a> for &'b T {
    type DisplayType = T::DisplayType;
    fn display_with_cfg(&'a self, cfg: &'a CFGGraph) -> Self::DisplayType {
        (**self).display_with_cfg(cfg)
    }
    fn display_with_cfg_and_indent(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_indent(cfg, indent)
    }
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_indent_and_is_debug(cfg, indent, is_debug)
    }
    fn display_with_cfg_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_is_debug(cfg, is_debug)
    }
}

impl<'a, 'b, T: DisplayWithCFG<'a>> DisplayWithCFG<'a> for &'b mut T {
    type DisplayType = T::DisplayType;
    fn display_with_cfg(&'a self, cfg: &'a CFGGraph) -> Self::DisplayType {
        (**self).display_with_cfg(cfg)
    }
    fn display_with_cfg_and_indent(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_indent(cfg, indent)
    }
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_indent_and_is_debug(cfg, indent, is_debug)
    }
    fn display_with_cfg_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_is_debug(cfg, is_debug)
    }
}

impl<'a, T: DisplayWithCFG<'a>> DisplayWithCFG<'a> for Box<T> {
    type DisplayType = T::DisplayType;
    fn display_with_cfg(&'a self, cfg: &'a CFGGraph) -> Self::DisplayType {
        (**self).display_with_cfg(cfg)
    }
    fn display_with_cfg_and_indent(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_indent(cfg, indent)
    }
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_indent_and_is_debug(cfg, indent, is_debug)
    }
    fn display_with_cfg_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_is_debug(cfg, is_debug)
    }
}

impl<'a, T: DisplayWithCFG<'a>> DisplayWithCFG<'a> for std::rc::Rc<T> {
    type DisplayType = T::DisplayType;
    fn display_with_cfg(&'a self, cfg: &'a CFGGraph) -> Self::DisplayType {
        (**self).display_with_cfg(cfg)
    }
    fn display_with_cfg_and_indent(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_indent(cfg, indent)
    }
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_indent_and_is_debug(cfg, indent, is_debug)
    }
    fn display_with_cfg_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        (**self).display_with_cfg_and_is_debug(cfg, is_debug)
    }
}

pub struct DisplaySetWithCFG<'a, Set> {
    set: Set,
    cfg: &'a CFGGraph,
    is_debug: Option<bool>,
}

impl<'a, T: for<'b> DisplayWithCFG<'b>, Set: IntoIterator<Item = T> + Clone>
    DisplaySetWithCFG<'a, Set>
{
    pub fn new(set: Set, cfg: &'a CFGGraph, is_debug: Option<bool>) -> Self {
        Self { set, cfg, is_debug }
    }
}

impl<'a, T: for<'b> DisplayWithCFG<'b>, Set: IntoIterator<Item = T> + Clone> Debug
    for DisplaySetWithCFG<'a, Set>
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut debug_set = f.debug_set();
        for entry in self.set.clone().into_iter() {
            debug_set.entry(&HandleIsDebugWrapper {
                value: entry.display_with_cfg(self.cfg),
                is_debug: self.is_debug,
            });
        }
        debug_set.finish()
    }
}

impl<'a, T: for<'b> DisplayWithCFG<'b>, Set: IntoIterator<Item = T> + Clone> Display
    for DisplaySetWithCFG<'a, Set>
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}",
            &Self {
                is_debug: self.is_debug.or(Some(false)),
                set: self.set.clone(),
                cfg: self.cfg,
            }
        )
    }
}

pub struct DisplayListWithCFG<'a, List> {
    list: List,
    cfg: &'a CFGGraph,
    is_debug: Option<bool>,
}

impl<'a, T: for<'b> DisplayWithCFG<'b>, List: IntoIterator<Item = T> + Clone>
    DisplayListWithCFG<'a, List>
{
    pub fn new(list: List, cfg: &'a CFGGraph, is_debug: Option<bool>) -> Self {
        Self {
            list,
            cfg,
            is_debug,
        }
    }
}

impl<'a, T: for<'b> DisplayWithCFG<'b>, List: IntoIterator<Item = T> + Clone> Debug
    for DisplayListWithCFG<'a, List>
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut debug_list = f.debug_set();
        for entry in self.list.clone().into_iter() {
            debug_list.entry(&HandleIsDebugWrapper {
                value: entry.display_with_cfg(self.cfg),
                is_debug: self.is_debug,
            });
        }
        debug_list.finish()
    }
}

impl<'a, T: for<'b> DisplayWithCFG<'b>, List: IntoIterator<Item = T> + Clone> Display
    for DisplayListWithCFG<'a, List>
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}",
            &Self {
                is_debug: self.is_debug.or(Some(false)),
                list: self.list.clone(),
                cfg: self.cfg,
            }
        )
    }
}

impl<'a, T: 'a + for<'b> DisplayWithCFG<'b> + Eq + Hash> DisplayWithCFG<'a> for HashSet<T> {
    type DisplayType = DisplaySetWithCFG<'a, &'a HashSet<T>>;
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        _indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        DisplaySetWithCFG::new(self, cfg, is_debug)
    }
}

impl<'a, T: 'a + for<'b> DisplayWithCFG<'b>> DisplayWithCFG<'a> for Vec<T> {
    type DisplayType = DisplayListWithCFG<'a, &'a Vec<T>>;
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        _indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        DisplayListWithCFG::new(self, cfg, is_debug)
    }
}
