use super::data;
use std::cell::RefCell;
use std::sync::Arc;

#[derive(Debug)]
pub struct TreeDom {
    tree: Arc<parking_lot::Mutex<unitree::UNITree<data::NodeData>>>,
    errors: Vec<std::borrow::Cow<'static, str>>,
    quirks_mode: markup5ever::interface::QuirksMode,
    namespaces: RefCell<hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace>>,
    lineno: u64,
}

impl TreeDom {
    pub(super) fn new(
        tree: unitree::UNITree<data::NodeData>,
        errors: Vec<std::borrow::Cow<'static, str>>,
        quirks_mode: markup5ever::interface::QuirksMode,
        namespaces: hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace>,
        lineno: u64,
    ) -> Self {
        Self {
            tree: Arc::new(parking_lot::Mutex::new(tree)),
            errors,
            quirks_mode,
            namespaces: RefCell::new(namespaces),
            lineno,
        }
    }
}

impl std::fmt::Display for TreeDom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree.lock())
    }
}
