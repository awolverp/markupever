use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub(super) enum NodeGuardType {
    Document,
    Doctype,
    Comment,
    Text,
    Element,
    Pi,
}

impl From<&::treedom::data::NodeData> for NodeGuardType {
    fn from(value: &::treedom::data::NodeData) -> Self {
        match value {
            ::treedom::data::NodeData::Comment(..) => Self::Comment,
            ::treedom::data::NodeData::Doctype(..) => Self::Doctype,
            ::treedom::data::NodeData::Document(..) => Self::Document,
            ::treedom::data::NodeData::Element(..) => Self::Element,
            ::treedom::data::NodeData::ProcessingInstruction(..) => Self::Pi,
            ::treedom::data::NodeData::Text(..) => Self::Text,
        }
    }
}

pub(super) struct NodeGuard {
    pub tree: Arc<parking_lot::Mutex<::treedom::TreeDom>>,
    pub id: ::treedom::ego_tree::NodeId,
    pub type_: NodeGuardType,
}

impl NodeGuard {
    pub fn new(
        tree: Arc<parking_lot::Mutex<::treedom::TreeDom>>,
        id: ::treedom::ego_tree::NodeId,
        type_: NodeGuardType,
    ) -> Self {
        Self { tree, id, type_ }
    }

    pub fn from_nodemut(
        tree: Arc<parking_lot::Mutex<::treedom::TreeDom>>,
        mut node: ::treedom::ego_tree::NodeMut<'_, ::treedom::data::NodeData>,
    ) -> Self {
        Self::new(tree, node.id(), NodeGuardType::from(&*node.value()))
    }

    pub fn from_noderef(
        tree: Arc<parking_lot::Mutex<::treedom::TreeDom>>,
        node: ::treedom::ego_tree::NodeRef<'_, ::treedom::data::NodeData>,
    ) -> Self {
        Self::new(tree, node.id(), NodeGuardType::from(node.value()))
    }

    pub fn tree(&self) -> super::tree::PyTreeDom {
        super::tree::PyTreeDom::from_arc_mutex(self.tree.clone())
    }

    pub fn parent(&self) -> Option<Self> {
        let tree = self.tree.lock();
        let node = tree.get(self.id).unwrap();

        Some(Self::from_noderef(self.tree.clone(), node.parent()?))
    }

    pub fn prev_sibling(&self) -> Option<Self> {
        let tree = self.tree.lock();
        let node = tree.get(self.id).unwrap();

        Some(Self::from_noderef(self.tree.clone(), node.prev_sibling()?))
    }

    pub fn next_sibling(&self) -> Option<Self> {
        let tree = self.tree.lock();
        let node = tree.get(self.id).unwrap();

        Some(Self::from_noderef(self.tree.clone(), node.next_sibling()?))
    }

    pub fn first_child(&self) -> Option<Self> {
        let tree = self.tree.lock();
        let node = tree.get(self.id).unwrap();

        Some(Self::from_noderef(self.tree.clone(), node.first_child()?))
    }

    pub fn last_child(&self) -> Option<Self> {
        let tree = self.tree.lock();
        let node = tree.get(self.id).unwrap();

        Some(Self::from_noderef(self.tree.clone(), node.last_child()?))
    }

    pub fn has_siblings(&self) -> bool {
        let tree = self.tree.lock();
        let node = tree.get(self.id).unwrap();
        node.has_siblings()
    }

    pub fn has_children(&self) -> bool {
        let tree = self.tree.lock();
        let node = tree.get(self.id).unwrap();
        node.has_children()
    }

    pub fn hash(&self) -> Option<u64> {
        use std::hash::{Hash, Hasher};

        if matches!(self.type_, NodeGuardType::Element) {
            None
        } else {
            let mut state = std::hash::DefaultHasher::default();

            let tree = self.tree.lock();
            let node = tree.get(self.id).unwrap();

            node.value().hash(&mut state);
            Some(state.finish())
        }
    }
}

#[pyo3::pyclass(name = "Document", module = "xmarkup._rustlib", frozen)]
pub struct PyDocument(pub(super) NodeGuard);

#[pyo3::pyclass(name = "Doctype", module = "xmarkup._rustlib", frozen)]
pub struct PyDoctype(pub(super) NodeGuard);

#[pyo3::pyclass(name = "Comment", module = "xmarkup._rustlib", frozen)]
pub struct PyComment(pub(super) NodeGuard);

#[pyo3::pyclass(name = "Text", module = "xmarkup._rustlib", frozen)]
pub struct PyText(pub(super) NodeGuard);

#[pyo3::pyclass(name = "Element", module = "xmarkup._rustlib", frozen)]
pub struct PyElement(pub(super) NodeGuard);

#[pyo3::pyclass(name = "ProcessingInstruction", module = "xmarkup._rustlib", frozen)]
pub struct PyProcessingInstruction(pub(super) NodeGuard);
