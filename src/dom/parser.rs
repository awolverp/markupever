use super::data;

use std::cell::{Cell, UnsafeCell};

pub struct Parser {
    tree: UnsafeCell<unitree::UNITree<data::NodeData>>,
    errors: UnsafeCell<Vec<std::borrow::Cow<'static, str>>>,
    quirks_mode: Cell<markup5ever::interface::QuirksMode>,
    namespaces: UnsafeCell<hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace>>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tree: UnsafeCell::new(unitree::UNITree::new(data::NodeData::new(data::Document))),
            errors: UnsafeCell::new(Vec::new()),
            quirks_mode: Cell::new(markup5ever::interface::QuirksMode::NoQuirks),
            namespaces: UnsafeCell::new(hashbrown::HashMap::new()),
        }
    }

    fn tree_mut(&self) -> &mut unitree::UNITree<data::NodeData> {
        // SAFETY: Parser is not Send/Sync so cannot be used in multi threads.
        unsafe { &mut *self.tree.get() }
    }

    fn errors_mut(&self) -> &mut Vec<std::borrow::Cow<'static, str>> {
        // SAFETY: Parser is not Send/Sync so cannot be used in multi threads.
        unsafe { &mut *self.errors.get() }
    }

    fn namespaces_mut(
        &self,
    ) -> &mut hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace> {
        // SAFETY: Parser is not Send/Sync so cannot be used in multi threads.
        unsafe { &mut *self.namespaces.get() }
    }
}
