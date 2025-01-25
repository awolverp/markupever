pub mod data;
mod parser;


// pub struct Parser {
//     tree: 
// }

// use std::cell::{Cell, RefCell};
// use std::sync::Arc;

// /// We have to implement a clonable
// #[derive(Debug, Clone)]
// pub struct ClonedExpandedName {
//     pub ns: markup5ever::Namespace,
//     pub local: markup5ever::LocalName,
// }

// impl markup5ever::interface::ElemName for ClonedExpandedName {
//     fn local_name(&self) -> &markup5ever::LocalName {
//         &self.local
//     }
//     fn ns(&self) -> &markup5ever::Namespace {
//         &self.ns
//     }
// }

// impl From<markup5ever::ExpandedName<'_>> for ClonedExpandedName {
//     fn from(value: markup5ever::ExpandedName<'_>) -> Self {
//         Self {
//             ns: value.ns.clone(),
//             local: value.local.clone(),
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct Node {
//     index: unitree::Index,
//     tree: Arc<parking_lot::Mutex<unitree::UNITree<nodes::NodeData>>>,
// }

// impl Node {
//     #[inline]
//     fn from_index(
//         index: unitree::Index,
//         tree: Arc<parking_lot::Mutex<unitree::UNITree<nodes::NodeData>>>,
//     ) -> Self {
//         Node { index, tree }
//     }
// }

// pub struct TreeDom {
//     tree: Arc<parking_lot::Mutex<unitree::UNITree<nodes::NodeData>>>,
//     pub errors: RefCell<Vec<std::borrow::Cow<'static, str>>>,
//     pub quirks_mode: Cell<markup5ever::interface::QuirksMode>,
//     pub namespaces: RefCell<hashbrown::HashMap<markup5ever::Prefix, markup5ever::Namespace>>,
// }

// impl TreeDom {
//     #[inline]
//     pub fn new() -> Self {
//         let tree = unitree::UNITree::new(nodes::NodeData::new(nodes::Document));

//         Self {
//             tree: Arc::new(parking_lot::Mutex::new(tree)),
//             errors: RefCell::new(Vec::new()),
//             quirks_mode: Cell::new(markup5ever::interface::QuirksMode::NoQuirks),
//             namespaces: RefCell::new(hashbrown::HashMap::new()),
//         }
//     }
// }

// impl Default for TreeDom {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl markup5ever::interface::TreeSink for TreeDom {
//     type Handle = Node;
//     type Output = Self;
//     type ElemName<'a> = ClonedExpandedName;

//     fn finish(self) -> Self::Output {
//         self
//     }

//     fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
//         self.errors.borrow_mut().push(msg);
//     }

//     fn set_current_line(&self, _line_number: u64) {}

//     fn get_document(&self) -> Self::Handle {
//         Node::from_index(unitree::Index::default(), self.tree.clone())
//     }
// }
