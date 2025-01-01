use tendril::StrTendril;

use crate::core::send::OnceLock;
use crate::core::send::{make_atomic_tendril, AtomicTendril};
use std::sync::Arc;
use std::sync::Weak;

/// The root of HTML document
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentData;

/// The root of a minimal document object
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FragmentData;

/// the doctype is the required <!doctype html> preamble found at the top of all documents.
/// Its sole purpose is to prevent a browser from switching into so-called "quirks mode"
/// when rendering a document; that is, the <!doctype html> doctype ensures that the browser makes
/// a best-effort attempt at following the relevant specifications, rather than using a different
/// rendering mode that is incompatible with some specifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctypeData {
    pub name: AtomicTendril,
    pub public_id: AtomicTendril,
    pub system_id: AtomicTendril,
}

impl DoctypeData {
    pub fn new(name: StrTendril, public_id: StrTendril, system_id: StrTendril) -> Self {
        Self {
            name: make_atomic_tendril(name),
            public_id: make_atomic_tendril(public_id),
            system_id: make_atomic_tendril(system_id),
        }
    }
}

/// The Comment interface represents textual notations within markup; although it is generally not
/// visually shown, such comments are available to be read in the source view.
///
/// Comments are represented in HTML and XML as content between <!-- and -->. In XML,
/// like inside SVG or MathML markup, the character sequence -- cannot be used within a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentData {
    pub contents: AtomicTendril,
}

impl CommentData {
    pub fn new(contents: StrTendril) -> Self {
        Self {
            contents: make_atomic_tendril(contents),
        }
    }
}

/// A text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextData {
    pub contents: AtomicTendril,
}

impl TextData {
    pub fn new(contents: StrTendril) -> Self {
        Self {
            contents: make_atomic_tendril(contents),
        }
    }
}

/// An element
pub struct ElementData {
    pub name: markup5ever::QualName,
    pub attrs: Vec<(markup5ever::QualName, AtomicTendril)>,
    pub template: bool,
    pub mathml_annotation_xml_integration_point: bool,

    /// cache id attribute
    id: OnceLock<Option<AtomicTendril>>,

    /// cache class attribute
    classes: OnceLock<Vec<markup5ever::LocalName>>,
}

impl ElementData {
    pub fn new(
        name: markup5ever::QualName,
        attrs: Vec<(markup5ever::QualName, AtomicTendril)>,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> Self {
        Self {
            name,
            attrs,
            template,
            mathml_annotation_xml_integration_point,
            id: OnceLock::new(),
            classes: OnceLock::new(),
        }
    }

    /// Finds the first 'id' attribute and returns its value.
    ///
    /// Also, caches it for the next calls.
    pub fn id(&self) -> Option<&str> {
        self.id
            .get_or_init(|| {
                self.attrs
                    .iter()
                    .find(|(name, _)| &name.local == "id")
                    .map(|(_, value)| value.clone())
            })
            .as_deref()
    }

    /// Finds 'class' attributes and returns its' values as a list.
    ///
    /// Also, caches it for the next calls.
    pub fn classes(&self) -> std::slice::Iter<'_, markup5ever::LocalName> {
        let classes = self.classes.get_or_init(|| {
            let mut classes = self
                .attrs
                .iter()
                .filter(|(name, _)| name.local.as_ref() == "class")
                .flat_map(|(_, value)| {
                    value
                        .split_ascii_whitespace()
                        .map(markup5ever::LocalName::from)
                })
                .collect::<Vec<_>>();

            classes.sort_unstable();
            classes.dedup();

            classes
        });

        classes.iter()
    }

    /// Clears the 'id' attribute cache
    pub fn clear_id(&mut self) {
        self.id.take();
    }

    /// Clears 'class' attributes cache
    pub fn clear_classes(&mut self) {
        self.classes.take();
    }
}

impl std::fmt::Debug for ElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElementData")
            .field("name", &self.name)
            .field("attrs", &self.attrs)
            .field("template", &self.template)
            .field(
                "mathml_annotation_xml_integration_point",
                &self.mathml_annotation_xml_integration_point,
            )
            .finish()
    }
}

/// The ProcessingInstruction interface represents a processing instruction; that is,
/// a Node which embeds an instruction targeting a specific application but that can
/// be ignored by any other applications which don't recognize the instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessingInstructionData {
    pub data: AtomicTendril,
    pub target: AtomicTendril,
}

impl ProcessingInstructionData {
    pub fn new(data: StrTendril, target: StrTendril) -> Self {
        Self {
            data: make_atomic_tendril(data),
            target: make_atomic_tendril(target),
        }
    }
}

#[derive(Debug)]
pub enum NodeData {
    Document(DocumentData),
    Fragment(FragmentData),
    Doctype(DoctypeData),
    Comment(CommentData),
    Text(TextData),
    Element(ElementData),
    ProcessingInstruction(ProcessingInstructionData),
}

macro_rules! _impl_nodedata_from_trait {
    ($from:ty, $name:ident) => {
        impl From<$from> for NodeData {
            fn from(value: $from) -> NodeData {
                NodeData::$name(value)
            }
        }
    };
}

_impl_nodedata_from_trait!(DocumentData, Document);
_impl_nodedata_from_trait!(FragmentData, Fragment);
_impl_nodedata_from_trait!(DoctypeData, Doctype);
_impl_nodedata_from_trait!(CommentData, Comment);
_impl_nodedata_from_trait!(TextData, Text);
_impl_nodedata_from_trait!(ElementData, Element);
_impl_nodedata_from_trait!(ProcessingInstructionData, ProcessingInstruction);

pub(super) struct NodeInner {
    /// Parent node.
    pub(super) parent: parking_lot::Mutex<Option<WeakNode>>,
    /// Child nodes of this node.
    pub(super) children: parking_lot::Mutex<Vec<Node>>,
    /// Represents this node's data.
    pub(super) data: parking_lot::Mutex<NodeData>,
}

impl std::fmt::Debug for NodeInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("data", &*self.data.lock())
            .field("children", &*self.children.lock())
            .finish()
    }
}

/// A node that uses [`Arc`] to prevent clone. Also [`Arc`] is a help for connecting `Rust` to `Python`.
#[derive(Clone, Debug)]
pub struct Node {
    value: Arc<NodeInner>,
}

macro_rules! _impl_nodedata_functions {
    (
        $(#[$docs1:meta])*
        is $isname:ident($enum_:pat)

        $(#[$docs2:meta])*
        get $gname:ident($pattern:pat_param => $param:expr, $ret:ty)

        $(#[$docs3:meta])*
        unch $uname:ident()
    ) => {
        $(#[$docs1])*
        pub fn $isname(&self) -> bool {
            let ref_ = self.value.data.lock();
            matches!(&*ref_, $enum_)
        }

        $(#[$docs2])*
        pub fn $gname(&self) -> Option<parking_lot::MappedMutexGuard<'_, $ret>> {
            let ref_ = self.value.data.lock();
            let mapped = parking_lot::MutexGuard::try_map(ref_, |x| match x {
                $pattern => Some($param),
                _ => None,
            });

            match mapped {
                Ok(x) => Some(x),
                Err(_) => None,
            }
        }

        $(#[$docs3])*
        pub unsafe fn $uname(&self) -> parking_lot::MappedMutexGuard<'_, $ret> {
            let ref_ = self.value.data.lock();
            parking_lot::MutexGuard::map(ref_, |x| match x {
                $pattern => $param,
                _ => std::hint::unreachable_unchecked(),
            })
        }
    };
}

impl Node {
    #[inline]
    pub fn new<T: Into<NodeData>>(data: T) -> Self {
        Self::new_with(data, None, Vec::new())
    }

    #[inline]
    pub fn new_with<T: Into<NodeData>>(
        data: T,
        parent: Option<WeakNode>,
        children: Vec<Node>,
    ) -> Self {
        Self {
            value: Arc::new(NodeInner {
                parent: parking_lot::Mutex::new(parent),
                children: parking_lot::Mutex::new(children),
                data: parking_lot::Mutex::new(data.into()),
            }),
        }
    }

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`DocumentData`]
        is is_document(NodeData::Document(..))

        /// Locks the data mutex and returns the [`DocumentData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_document(NodeData::Document(d) => d, DocumentData)

        /// Locks the data mutex and returns the [`DocumentData`] but never checks that is the holded data is document or not.
        ///
        /// # Safety
        /// If you're not sure about the data that node keeps, don't use this method; otherwise you will see a undefined behaviour.
        unch as_document_unchecked()
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`FragmentData`]
        is is_fragment(NodeData::Fragment(..))

        /// Locks the data mutex and returns the [`FragmentData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_fragment(NodeData::Fragment(d) => d, FragmentData)

        /// Locks the data mutex and returns the [`FragmentData`] but never checks that is the holded data is document or not.
        ///
        /// # Safety
        /// If you're not sure about the data that node keeps, don't use this method; otherwise you will see a undefined behaviour.
        unch as_fragment_unchecked()
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`DoctypeData`]
        is is_doctype(NodeData::Doctype(..))

        /// Locks the data mutex and returns the [`DoctypeData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_doctype(NodeData::Doctype(d) => d, DoctypeData)

        /// Locks the data mutex and returns the [`DoctypeData`] but never checks that is the holded data is document or not.
        ///
        /// # Safety
        /// If you're not sure about the data that node keeps, don't use this method; otherwise you will see a undefined behaviour.
        unch as_doctype_unchecked()
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`CommentData`]
        is is_comment(NodeData::Comment(..))

        /// Locks the data mutex and returns the [`CommentData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_comment(NodeData::Comment(d) => d, CommentData)

        /// Locks the data mutex and returns the [`CommentData`] but never checks that is the holded data is document or not.
        ///
        /// # Safety
        /// If you're not sure about the data that node keeps, don't use this method; otherwise you will see a undefined behaviour.
        unch as_comment_unchecked()
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`TextData`]
        is is_text(NodeData::Text(..))

        /// Locks the data mutex and returns the [`TextData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_text(NodeData::Text(d) => d, TextData)

        /// Locks the data mutex and returns the [`TextData`] but never checks that is the holded data is document or not.
        ///
        /// # Safety
        /// If you're not sure about the data that node keeps, don't use this method; otherwise you will see a undefined behaviour.
        unch as_text_unchecked()
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`ElementData`]
        is is_element(NodeData::Element(..))

        /// Locks the data mutex and returns the [`ElementData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_element(NodeData::Element(d) => d, ElementData)

        /// Locks the data mutex and returns the [`ElementData`] but never checks that is the holded data is document or not.
        ///
        /// # Safety
        /// If you're not sure about the data that node keeps, don't use this method; otherwise you will see a undefined behaviour.
        unch as_element_unchecked()
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`ProcessingInstructionData`]
        is is_processing_instruction(NodeData::ProcessingInstruction(..))

        /// Locks the data mutex and returns the [`ProcessingInstructionData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_processing_instruction(NodeData::ProcessingInstruction(d) => d, ProcessingInstructionData)

        /// Locks the data mutex and returns the [`ProcessingInstructionData`] but never checks that is the holded data is document or not.
        ///
        /// # Safety
        /// If you're not sure about the data that node keeps, don't use this method; otherwise you will see a undefined behaviour.
        unch as_processing_instruction_unchecked()
    );

    /// Creates a new [`WeakNode`] pointer to this allocation.
    #[inline]
    pub fn downgrade(&self) -> WeakNode {
        WeakNode {
            value: std::sync::Arc::downgrade(&self.value),
        }
    }

    /// Locks the parent and returns it
    ///
    /// It is necessary to drop it when you don't need it
    pub fn parent(&self) -> parking_lot::MappedMutexGuard<'_, Option<WeakNode>> {
        let ref_ = self.value.parent.lock();
        parking_lot::MutexGuard::map(ref_, |x| x)
    }

    /// Locks the children and returns it
    ///
    /// It is necessary to drop it when you don't need it
    pub fn children(&self) -> Children<'_> {
        let ref_ = self.value.children.lock();

        Children {
            node: self,
            vec: parking_lot::MutexGuard::map(ref_, |x| x),
        }
    }

    /// Iterates all nodes and their children like a tree
    pub fn tree(&self) -> NodesTree {
        NodesTree::new(self.clone(), true)
    }

    /// Unlike the iter method, iterates all the parents.
    pub fn parents(&self) -> ParentsIterator {
        ParentsIterator::new(self, false)
    }

    /// Returns `true` if the two [`Node`]s point to the same allocation
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.value, &other.value)
    }

    /// Works like self.unlink but does not remove self from parent's children
    pub(super) unsafe fn unlink_parent(&self) {
        debug_assert!(
            !self.value.parent.is_locked(),
            "before using this method you have to unlock the parent with dropping the mutex guard"
        );

        self.parent().take();
    }

    /// Removes this node from its parent.
    #[inline]
    pub fn unlink(&self) {
        debug_assert!(
            !self.value.parent.is_locked(),
            "before using this method you have to unlock the parent with dropping the mutex guard"
        );

        if let Some(parent) = self.parent().take() {
            let parent = parent.upgrade().expect("dangling weak pointer");
            let mut c = parent.children();
            unsafe { c.remove_index(c.position(self).unwrap()) };
        }
    }

    pub fn as_nodedata(&self) -> parking_lot::MappedMutexGuard<'_, NodeData> {
        let ref_ = self.value.data.lock();
        parking_lot::MutexGuard::map(ref_, |x| x)
    }
}

enum NodeEdge {
    Open(Node),
    Close(markup5ever::QualName),
}

impl markup5ever::serialize::Serialize for Node {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        traversal_scope: markup5ever::serialize::TraversalScope,
    ) -> std::io::Result<()>
    where
        S: markup5ever::serialize::Serializer,
    {
        let mut edges: Vec<NodeEdge> = Vec::new();

        match traversal_scope {
            markup5ever::serialize::TraversalScope::IncludeNode => {
                edges.push(NodeEdge::Open(self.clone()))
            }
            markup5ever::serialize::TraversalScope::ChildrenOnly(_) => edges.extend(
                self.children()
                    .iter()
                    .rev()
                    .map(|h| NodeEdge::Open(h.clone())),
            ),
        }

        while let Some(edge) = edges.pop() {
            match edge {
                NodeEdge::Close(name) => {
                    serializer.end_elem(name)?;
                }

                NodeEdge::Open(node) => match &*node.as_nodedata() {
                    NodeData::Element(elem) => {
                        serializer.start_elem(
                            elem.name.clone(),
                            elem.attrs.iter().map(|at| (&at.0, &at.1[..])),
                        )?;

                        edges.push(NodeEdge::Close(elem.name.clone()));

                        edges.extend(
                            node.children()
                                .into_iter()
                                .rev()
                                .map(|child| NodeEdge::Open(child.clone())),
                        );
                    }

                    NodeData::Doctype(doctype) => serializer.write_doctype(&doctype.name)?,

                    NodeData::Text(t) => serializer.write_text(&t.contents)?,

                    NodeData::Comment(c) => serializer.write_comment(&c.contents)?,

                    NodeData::ProcessingInstruction(pi) => {
                        serializer.write_processing_instruction(&pi.target, &pi.data)?
                    }

                    _ => unreachable!(),
                },
            }
        }

        Ok(())
    }
}

/// [`WeakNode`] is a version of [`Node`] that holds a non-owning reference to the managed allocation.
#[derive(Clone)]
pub struct WeakNode {
    value: Weak<NodeInner>,
}

impl WeakNode {
    /// Upgrade self to [`Node`]
    #[inline]
    pub fn upgrade(&self) -> Option<Node> {
        self.value.upgrade().map(|x| Node { value: x })
    }
}

pub struct Children<'a> {
    node: &'a Node,
    vec: parking_lot::MappedMutexGuard<'a, Vec<Node>>,
}

impl<'a> Children<'a> {
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn position(&self, child: &Node) -> Option<usize> {
        self.vec.iter().position(|x| x.ptr_eq(child))
    }

    #[inline]
    pub fn append(&mut self, child: Node) -> Result<(), Node> {
        if self.node.ptr_eq(&child) {
            // Avoid cycle
            return Err(child);
        }

        #[cfg(not(debug_assertions))]
        child.parent().replace(self.node.downgrade());

        #[cfg(debug_assertions)]
        let old = child.parent().replace(self.node.downgrade());

        debug_assert!(old.is_none(), "child cannot have existing parent");

        self.vec.push(child);
        Ok(())
    }

    #[inline]
    pub fn insert(&mut self, index: usize, child: Node) -> Result<(), Node> {
        if self.node.ptr_eq(&child) {
            // Avoid cycle
            return Err(child);
        }

        #[cfg(not(debug_assertions))]
        child.parent().replace(self.node.downgrade());

        #[cfg(debug_assertions)]
        let old = child.parent().replace(self.node.downgrade());

        debug_assert!(old.is_none(), "child cannot have existing parent");

        self.vec.insert(index, child);
        Ok(())
    }

    #[inline]
    pub fn last(&self) -> Option<&Node> {
        self.vec.last()
    }

    #[inline]
    pub fn first(&self) -> Option<&Node> {
        self.vec.first()
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Node> {
        self.vec.iter()
    }

    /// NOTE: don't forgot to unlink children
    #[inline]
    pub fn drain<R: std::ops::RangeBounds<usize>>(
        &mut self,
        range: R,
    ) -> std::vec::Drain<'_, Node> {
        self.vec.drain(range)
    }

    pub(super) unsafe fn remove_index(&mut self, index: usize) -> Node {
        self.vec.remove(index)
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Node {
        let node = self.vec.remove(index);

        {
            debug_assert!(
                !node.value.parent.is_locked(),
                "The child node parent is locked; Unlock it first."
            );

            let mut p = node.value.parent.lock();
            p.take();
        }

        node
    }

    #[inline]
    pub fn clear(&mut self) {
        self.vec.clear();
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&Node> {
        self.vec.get(index)
    }
}

impl<'a> std::ops::Index<usize> for Children<'a> {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

impl<'a> std::ops::IndexMut<usize> for Children<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl<'a> std::ops::Index<std::ops::Range<usize>> for Children<'a> {
    type Output = [Node];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.vec[index]
    }
}

impl<'a> std::ops::IndexMut<std::ops::Range<usize>> for Children<'a> {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl<'a> std::ops::Index<std::ops::RangeTo<usize>> for Children<'a> {
    type Output = [Node];

    fn index(&self, index: std::ops::RangeTo<usize>) -> &Self::Output {
        &self.vec[index]
    }
}

impl<'a> std::ops::IndexMut<std::ops::RangeTo<usize>> for Children<'a> {
    fn index_mut(&mut self, index: std::ops::RangeTo<usize>) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl<'a> std::ops::Index<std::ops::RangeFrom<usize>> for Children<'a> {
    type Output = [Node];

    fn index(&self, index: std::ops::RangeFrom<usize>) -> &Self::Output {
        &self.vec[index]
    }
}

impl<'a> std::ops::IndexMut<std::ops::RangeFrom<usize>> for Children<'a> {
    fn index_mut(&mut self, index: std::ops::RangeFrom<usize>) -> &mut Self::Output {
        &mut self.vec[index]
    }
}

impl<'a> IntoIterator for Children<'a> {
    type Item = Node;
    type IntoIter = std::vec::IntoIter<Node>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.clone().into_iter()
    }
}

pub struct NodesTree {
    vec: Vec<Node>,
}

impl NodesTree {
    pub fn new(node: Node, include_node: bool) -> Self {
        let mut s = Self { vec: Vec::new() };

        if include_node {
            s.vec.push(node);
        } else {
            let children = node.children();
            s.vec.extend(children);
        }

        s
    }
}

impl Iterator for NodesTree {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.vec.pop()?;

        self.vec.extend(node.children().into_iter().rev());

        Some(node)
    }
}

pub struct ParentsIterator {
    last: Option<Node>,
}

impl ParentsIterator {
    pub fn new(node: &Node, include_node: bool) -> Self {
        let mut s = Self { last: None };

        if include_node {
            s.last = Some(node.clone());
        } else {
            s.last = node
                .parent()
                .clone()
                .map(|x| x.upgrade().expect("dangling weak reference"));
        }

        s
    }
}

impl Iterator for ParentsIterator {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.last.take()?;

        self.last = match node.parent().clone().map(|x| x.upgrade()) {
            None => None,
            Some(x) => x
        };

        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    trait IsSendAndSync: Send + Sync {}

    impl IsSendAndSync for Node {}
    impl IsSendAndSync for NodeInner {}

    macro_rules! create_element {
        ($name:expr, $attrs:expr) => {
            ElementData::new(
                markup5ever::QualName::new(
                    None,
                    markup5ever::namespace_url!(""),
                    markup5ever::LocalName::from($name),
                ),
                $attrs,
                false,
                false,
            )
        };
    }

    #[test]
    pub fn test_data() {
        let elem = create_element!("div", vec![]);
        assert_eq!(&*elem.name.local, "div");

        let mut elem = create_element!(
            "div",
            Vec::from([
                (
                    markup5ever::QualName::new(
                        None,
                        markup5ever::namespace_url!(""),
                        markup5ever::local_name!("id"),
                    ),
                    "example_id".into()
                ),
                (
                    markup5ever::QualName::new(
                        None,
                        markup5ever::namespace_url!(""),
                        markup5ever::local_name!("class"),
                    ),
                    "cls1 cls2".into()
                ),
                (
                    markup5ever::QualName::new(
                        None,
                        markup5ever::namespace_url!(""),
                        markup5ever::local_name!("class"),
                    ),
                    "cls2 cls3".into()
                ),
                (
                    markup5ever::QualName::new(
                        Some(markup5ever::Prefix::from("data-test")),
                        markup5ever::namespace_url!(""),
                        markup5ever::local_name!(""),
                    ),
                    "test".into()
                ),
            ])
        );
        assert_eq!(elem.id(), Some("example_id"));
        assert_eq!(elem.classes().len(), 3);

        elem.attrs.clear();
        elem.clear_id();
        elem.clear_classes();

        assert_eq!(elem.id(), None);
        assert_eq!(elem.classes().len(), 0);
    }

    #[test]
    fn test_nodedata() {
        let data: NodeData = DocumentData.into();
        debug_assert!(matches!(data, NodeData::Document(..)));

        let data: NodeData = FragmentData.into();
        debug_assert!(matches!(data, NodeData::Fragment(..)));
    }

    #[test]
    fn test_node_children() {
        let node = Node::new(create_element!("div", Default::default()));

        let child1 = Node::new(create_element!("h1", Default::default()));
        let child1_child = Node::new(TextData::new("Come here 1".into()));
        child1.children().append(child1_child.clone()).unwrap();

        node.children().append(child1.clone()).unwrap();

        let child2 = Node::new(create_element!("h2", Default::default()));
        let child2_child = Node::new(TextData::new("Come here 2".into()));
        child2.children().append(child2_child.clone()).unwrap();

        node.children().append(child2.clone()).unwrap();

        let child3 = Node::new(create_element!("p", Default::default()));
        let child3_child = Node::new(TextData::new("Come here 3".into()));
        child3.children().append(child3_child).unwrap();

        node.children().append(child3.clone()).unwrap();

        assert_eq!(node.children().len(), 3);

        let mut v = Vec::new();
        for n in node.tree() {
            v.push(n);
        }

        assert_eq!(v.len(), 7);

        assert_eq!(node.children().position(&child3), Some(2));
        assert!(node.children().remove(2).ptr_eq(&child3));

        assert_eq!(node.children().len(), 2);

        let mut v = Vec::new();
        for n in node.tree() {
            v.push(n);
        }

        assert_eq!(v.len(), 5);

        let have_to = vec![node, child1, child1_child, child2, child2_child];

        for (v1, v2) in v.iter().zip(have_to.iter()) {
            assert!(v1.ptr_eq(v2))
        }
    }

    #[test]
    fn test_cycle() {
        let node = Node::new(DocumentData);
        node.children().append(node.clone()).unwrap_err();
    }
}
