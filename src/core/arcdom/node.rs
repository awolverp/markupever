use tendril::StrTendril;

use crate::core::send::OnceLock;
use crate::core::send::{make_atomic_tendril, AtomicTendril};
use std::sync::Arc;
use std::sync::Weak;

pub type SizedSmallVec<T> = smallvec::SmallVec<[T; 4]>;

/// The root of HTML document
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentData;

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
    /// Create a new `DoctypeData`
    #[inline]
    pub fn new(name: AtomicTendril, public_id: AtomicTendril, system_id: AtomicTendril) -> Self {
        Self {
            name,
            public_id,
            system_id,
        }
    }

    /// Create a new `DoctypeData` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(name: StrTendril, public_id: StrTendril, system_id: StrTendril) -> Self {
        Self::new(
            make_atomic_tendril(name),
            make_atomic_tendril(public_id),
            make_atomic_tendril(system_id),
        )
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
    /// Create a new `CommentData`
    #[inline]
    pub fn new(contents: AtomicTendril) -> Self {
        Self { contents }
    }

    /// Create a new `CommentData` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(contents: StrTendril) -> Self {
        Self::new(make_atomic_tendril(contents))
    }
}

/// A text
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextData {
    pub contents: AtomicTendril,
}

impl TextData {
    /// Create a new `TextData`
    #[inline]
    pub fn new(contents: AtomicTendril) -> Self {
        Self { contents }
    }

    /// Create a new `TextData` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(contents: StrTendril) -> Self {
        Self::new(make_atomic_tendril(contents))
    }

    /// Push another StrTendril onto the end of this one.
    #[inline]
    pub fn push_non_atomic(&mut self, contents: StrTendril) {
        self.contents.push_tendril(&make_atomic_tendril(contents));
    }
}

/// ElementData attributes that caches 'id' and 'class' attributes of element
/// and also triggers update will removes caches when attributes updated
#[derive(Clone)]
pub struct ElementAttributeTrigger {
    item: SizedSmallVec<(markup5ever::QualName, AtomicTendril)>,
    id: OnceLock<Option<AtomicTendril>>,
    classes: OnceLock<Vec<markup5ever::LocalName>>,
}

impl ElementAttributeTrigger {
    /// Creates a new `ElementAttributeTrigger`
    #[inline]
    pub fn new<I>(item: I) -> Self
    where
        I: Iterator<Item = (markup5ever::QualName, AtomicTendril)>,
    {
        Self {
            item: item.collect(),
            id: OnceLock::new(),
            classes: OnceLock::new(),
        }
    }

    /// Finds, caches, and returns the 'id' attribute from attributes.
    #[inline]
    pub fn id(&self) -> Option<&str> {
        self.id
            .get_or_init(|| {
                self.item
                    .iter()
                    .find(|(name, _)| &name.local == "id")
                    .map(|(_, value)| value.clone())
            })
            .as_deref()
    }

    /// Finds, caches, and returns the 'class' attributes from attributes.
    #[inline]
    pub fn classes(&self) -> std::slice::Iter<'_, markup5ever::LocalName> {
        let classes = self.classes.get_or_init(|| {
            let mut classes = self
                .item
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
}

impl std::ops::Deref for ElementAttributeTrigger {
    type Target = SizedSmallVec<(markup5ever::QualName, AtomicTendril)>;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl std::ops::DerefMut for ElementAttributeTrigger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.id.take();
        self.classes.take();

        &mut self.item
    }
}

/// An element
#[derive(Clone)]
pub struct ElementData {
    pub name: markup5ever::QualName,
    pub attrs: ElementAttributeTrigger,
    pub template: bool,
    pub mathml_annotation_xml_integration_point: bool,
}

impl ElementData {
    /// Creates a new `ElementData`
    #[inline]
    pub fn new<I>(
        name: markup5ever::QualName,
        attrs: I,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> Box<Self>
    where
        I: Iterator<Item = (markup5ever::QualName, AtomicTendril)>,
    {
        Box::new(Self {
            name,
            attrs: ElementAttributeTrigger::new(attrs),
            template,
            mathml_annotation_xml_integration_point,
        })
    }

    /// Creates a new `ElementData` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic<I>(
        name: markup5ever::QualName,
        attrs: I,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> Box<Self>
    where
        I: Iterator<Item = (markup5ever::QualName, StrTendril)>,
    {
        Self::new(
            name,
            attrs.map(|(key, val)| (key, make_atomic_tendril(val))),
            template,
            mathml_annotation_xml_integration_point,
        )
    }
}

impl PartialEq for ElementData {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name
            || self.template != other.template
            || self.mathml_annotation_xml_integration_point
                != other.mathml_annotation_xml_integration_point
        {
            return false;
        }

        self.attrs
            .iter()
            .all(|x| other.attrs.binary_search(x).is_ok())
    }
}

impl Eq for ElementData {}

impl std::fmt::Debug for ElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ElementData")
            .field("name", &self.name)
            .field("attrs", &*self.attrs)
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
    /// Creates a new `ProcessingInstructionData`
    #[inline]
    pub fn new(data: AtomicTendril, target: AtomicTendril) -> Self {
        Self { data, target }
    }

    /// Creates a new `ProcessingInstructionData` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(data: StrTendril, target: StrTendril) -> Self {
        Self::new(make_atomic_tendril(data), make_atomic_tendril(target))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeData {
    Document(DocumentData),
    Doctype(DoctypeData),
    Comment(CommentData),
    Text(TextData),
    Element(Box<ElementData>),
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
_impl_nodedata_from_trait!(DoctypeData, Doctype);
_impl_nodedata_from_trait!(CommentData, Comment);
_impl_nodedata_from_trait!(TextData, Text);
_impl_nodedata_from_trait!(Box<ElementData>, Element);
_impl_nodedata_from_trait!(ProcessingInstructionData, ProcessingInstruction);

pub(super) struct NodeInner {
    pub(super) parent: parking_lot::Mutex<Option<WeakNode>>,
    pub(super) children: parking_lot::Mutex<smallvec::SmallVec<[Node; 4]>>,
    pub(super) data: parking_lot::Mutex<NodeData>,
}

/// A `Node` of DOM. each data is wrapped by [`parking_lot::Mutex`]
/// to keep `Node` [`Sync`] and [`Send`].
///
/// All data is wrapped by a [`Arc`] so you can clone and move the node without worry.
#[derive(Clone)]
pub struct Node {
    pub(super) inner: Arc<NodeInner>,
}

macro_rules! _impl_nodedata_functions {
    (
        $(#[$docs1:meta])*
        is $isname:ident($enum_:pat)

        $(#[$docs2:meta])*
        get $gname:ident($pattern:pat_param => $param:expr, $ret:ty)
    ) => {
        $(#[$docs1])*
        pub fn $isname(&self) -> bool {
            matches!(
                // SAFETY: there's no important what is the data.
                // we only want to know what is the NodeData used here
                // and the NodeData never be changed.
                unsafe { &*self.inner.data.data_ptr() },
                $enum_
            )
        }

        $(#[$docs2])*
        pub fn $gname(&self) -> Option<parking_lot::MappedMutexGuard<'_, $ret>> {
            let ref_ = self.inner.data.lock();
            let mapped = parking_lot::MutexGuard::try_map(ref_, |x| match x {
                $pattern => Some($param),
                _ => None,
            });

            match mapped {
                Ok(x) => Some(x),
                Err(_) => None,
            }
        }
    };
}

impl Node {
    /// Creates a new `Node` with a parent [`None`] and an empty children
    #[inline]
    pub fn new<T: Into<NodeData>>(data: T) -> Self {
        Self::new_with(data, None, [])
    }

    /// Creates a new `Node`
    #[inline]
    pub fn new_with<T: Into<NodeData>, I: IntoIterator<Item = Node>>(
        data: T,
        parent: Option<WeakNode>,
        children: I,
    ) -> Self {
        Self {
            inner: Arc::new(NodeInner {
                parent: parking_lot::Mutex::new(parent),
                children: parking_lot::Mutex::new(children.into_iter().collect()),
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
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`DoctypeData`]
        is is_doctype(NodeData::Doctype(..))

        /// Locks the data mutex and returns the [`DoctypeData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_doctype(NodeData::Doctype(d) => d, DoctypeData)
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`CommentData`]
        is is_comment(NodeData::Comment(..))

        /// Locks the data mutex and returns the [`CommentData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_comment(NodeData::Comment(d) => d, CommentData)
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`TextData`]
        is is_text(NodeData::Text(..))

        /// Locks the data mutex and returns the [`TextData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_text(NodeData::Text(d) => d, TextData)
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`ElementData`]
        is is_element(NodeData::Element(..))

        /// Locks the data mutex and returns the [`ElementData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_element(NodeData::Element(d) => d, Box<ElementData>)
    );

    _impl_nodedata_functions!(
        /// Returns `true` if the node data is [`ProcessingInstructionData`]
        is is_processing_instruction(NodeData::ProcessingInstruction(..))

        /// Locks the data mutex and returns the [`ProcessingInstructionData`]
        ///
        /// It is necessary to drop it when you don't need it
        get as_processing_instruction(NodeData::ProcessingInstruction(d) => d, ProcessingInstructionData)
    );

    /// Creates a new [`WeakNode`] pointer to this allocation.
    #[inline]
    pub fn downgrade(&self) -> WeakNode {
        WeakNode {
            inner: std::sync::Arc::downgrade(&self.inner),
        }
    }

    /// Locks the parent and returns it
    ///
    /// It is necessary to drop it when you don't need it
    pub fn parent(&self) -> parking_lot::MappedMutexGuard<'_, Option<WeakNode>> {
        #[cfg(debug_assertions)]
        let ref_ = self.inner.parent.try_lock().expect("parent is locked");

        #[cfg(not(debug_assertions))]
        let ref_ = self.inner.parent.lock();

        parking_lot::MutexGuard::map(ref_, |x| x)
    }

    /// Locks the children and returns it
    ///
    /// It is necessary to drop it when you don't need it
    #[inline]
    pub fn children(&self) -> super::iter::ChildrenMutexGuard<'_> {
        super::iter::ChildrenMutexGuard::new(self)
    }

    /// Returns a [`TreeIterator`](struct@super::iter::TreeIterator) that iterates all children
    /// and also their children like a tree.
    ///
    /// Use [`Node::into_tree`] method if you want to include self in [`TreeIterator`](struct@super::iter::TreeIterator).
    pub fn tree(&self) -> super::iter::TreeIterator {
        super::iter::TreeIterator::new(self.children())
    }

    /// Returns a [`TreeIterator`](struct@super::iter::TreeIterator) that iterates all children
    /// and also their children like a tree.
    ///
    /// See also [`Node::tree`].
    pub fn into_tree(self) -> super::iter::TreeIterator {
        super::iter::TreeIterator::new_with_node(self)
    }

    /// Returns a [`ParentsIterator`](struct@super::iter::ParentsIterator) that iterates all parents.
    ///
    /// Use [`Node::into_parents`] method if you want to include self
    pub fn parents(&self) -> super::iter::ParentsIterator {
        super::iter::ParentsIterator::new(self.parent().clone())
    }

    /// Returns a [`ParentsIterator`](struct@super::iter::ParentsIterator) that iterates all parents.
    ///
    /// See also [`Node::parents`]
    pub fn into_parents(self) -> super::iter::ParentsIterator {
        super::iter::ParentsIterator::new_with_node(self)
    }

    /// Returns `true` if the two [`Node`]s point to the same allocation
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    /// Locks and returns the `Node`'s data as [`NodeData`]
    pub fn as_enum(&self) -> parking_lot::MappedMutexGuard<'_, NodeData> {
        let ref_ = self.inner.data.lock();
        parking_lot::MutexGuard::map(ref_, |x| x)
    }

    /// Serializes node as HTML5
    pub fn serialize_html<Wr>(&self, writer: Wr, include_self: bool) -> std::io::Result<()>
    where
        Wr: std::io::Write,
    {
        html5ever::serialize::serialize(
            writer,
            self,
            html5ever::serialize::SerializeOpts {
                scripting_enabled: false,
                create_missing_parent: false,
                traversal_scope: if include_self {
                    html5ever::serialize::TraversalScope::IncludeNode
                } else {
                    html5ever::serialize::TraversalScope::ChildrenOnly(None)
                },
            },
        )
    }

    /// Serializes node as XML
    pub fn serialize_xml<Wr>(&self, writer: Wr, include_self: bool) -> std::io::Result<()>
    where
        Wr: std::io::Write,
    {
        xml5ever::serialize::serialize(
            writer,
            self,
            xml5ever::serialize::SerializeOpts {
                traversal_scope: if include_self {
                    xml5ever::serialize::TraversalScope::IncludeNode
                } else {
                    xml5ever::serialize::TraversalScope::ChildrenOnly(None)
                },
            },
        )
    }

    /// Clones the inner data and returns a new `Node` that uses another `Arc` and `Mutex`s.
    pub fn copy(node: &Node) -> Node {
        Self {
            inner: Arc::new(NodeInner {
                parent: parking_lot::Mutex::new(node.inner.parent.lock().clone()),
                children: parking_lot::Mutex::new(node.inner.children.lock().clone()),
                data: parking_lot::Mutex::new(node.inner.data.lock().clone()),
            }),
        }
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("data", &*self.inner.data.lock())
            .field("children", &*self.inner.children.lock())
            .finish()
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

                NodeEdge::Open(node) => match &*node.as_enum() {
                    NodeData::Element(elem) => {
                        serializer.start_elem(
                            elem.name.clone(),
                            elem.attrs.iter().map(|at| (&at.0, &at.1[..])),
                        )?;

                        edges.push(NodeEdge::Close(elem.name.clone()));

                        edges.extend(node.children().iter().cloned().rev().map(NodeEdge::Open));
                    }

                    NodeData::Doctype(doctype) => serializer.write_doctype(&doctype.name)?,

                    NodeData::Text(t) => serializer.write_text(&t.contents)?,

                    NodeData::Comment(c) => serializer.write_comment(&c.contents)?,

                    NodeData::ProcessingInstruction(pi) => {
                        serializer.write_processing_instruction(&pi.target, &pi.data)?
                    }

                    NodeData::Document(_) => (),
                },
            }
        }

        Ok(())
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.inner, &other.inner) {
            return true;
        }

        let data1 = self.inner.data.lock();
        let data2: parking_lot::lock_api::MutexGuard<'_, parking_lot::RawMutex, NodeData> =
            other.inner.data.lock();

        data1.eq(&data2)
    }
}

impl Eq for Node {}

/// [`WeakNode`] is a version of [`Node`] that holds a non-owning reference to the managed allocation.
#[derive(Debug, Clone)]
pub struct WeakNode {
    inner: Weak<NodeInner>,
}

impl WeakNode {
    /// Upgrade self to [`Node`]
    #[inline]
    pub fn upgrade(&self) -> Option<Node> {
        self.inner.upgrade().map(|x| Node { inner: x })
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
            ElementData::from_non_atomic(
                markup5ever::QualName::new(
                    None,
                    markup5ever::namespace_url!(""),
                    markup5ever::LocalName::from($name),
                ),
                $attrs.into_iter(),
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
        assert_eq!(elem.attrs.id(), Some("example_id"));
        assert_eq!(elem.attrs.classes().len(), 3);

        elem.attrs.clear();

        assert_eq!(elem.attrs.id(), None);
        assert_eq!(elem.attrs.classes().len(), 0);
    }

    #[test]
    fn test_nodedata() {
        let data: NodeData = DocumentData.into();
        debug_assert!(matches!(data, NodeData::Document(..)));
    }

    #[test]
    fn test_node_children() {
        let node = Node::new(create_element!("div", vec![]));

        let child1 = Node::new(create_element!("h1", vec![]));
        let child1_child = Node::new(TextData::new("Come here 1".into()));
        child1.children().push(child1_child.clone()).unwrap();

        node.children().push(child1.clone()).unwrap();

        let child2 = Node::new(create_element!("h2", vec![]));
        let child2_child = Node::new(TextData::new("Come here 2".into()));
        child2.children().push(child2_child.clone()).unwrap();

        node.children().push(child2.clone()).unwrap();

        let child3 = Node::new(create_element!("p", vec![]));
        let child3_child = Node::new(TextData::new("Come here 3".into()));
        child3.children().push(child3_child).unwrap();

        node.children().push(child3.clone()).unwrap();

        assert_eq!(node.children().len(), 3);

        let mut v = Vec::new();
        for n in node.tree() {
            v.push(n);
        }

        assert_eq!(v.len(), 6);

        assert_eq!(
            node.children().iter().position(|x| x.ptr_eq(&child3)),
            Some(2)
        );
        assert!(node.children().remove(2).ptr_eq(&child3));

        assert_eq!(node.children().len(), 2);

        let mut v = Vec::new();
        for n in node.tree() {
            v.push(n);
        }

        assert_eq!(v.len(), 4);

        let have_to = vec![child1, child1_child, child2, child2_child];

        for (v1, v2) in v.iter().zip(have_to.iter()) {
            assert!(v1.ptr_eq(v2), "{:?} != {:?}", v1, v2);
        }
    }

    #[test]
    fn test_cycle() {
        let node = Node::new(DocumentData);
        node.children().push(node.clone()).unwrap_err();
    }
}
