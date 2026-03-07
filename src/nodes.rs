use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeGuardType {
    Document,
    Doctype,
    Comment,
    Text,
    Element,
    Pi,
}

#[derive(pyo3::FromPyObject)]
pub enum PyNodeRef<'p> {
    Document(pyo3::PyRef<'p, PyDocument>),
    Doctype(pyo3::PyRef<'p, PyDoctype>),
    Comment(pyo3::PyRef<'p, PyComment>),
    Text(pyo3::PyRef<'p, PyText>),
    Element(pyo3::PyRef<'p, PyElement>),
    Pi(pyo3::PyRef<'p, PyProcessingInstruction>),
}

impl PyNodeRef<'_> {
    pub fn as_node_guard(&self) -> &NodeGuard {
        match self {
            PyNodeRef::Document(x) => &x.0,
            PyNodeRef::Doctype(x) => &x.0,
            PyNodeRef::Comment(x) => &x.0,
            PyNodeRef::Text(x) => &x.0,
            PyNodeRef::Element(x) => &x.0,
            PyNodeRef::Pi(x) => &x.0,
        }
    }
}

impl From<&::treedom::interface::Interface> for NodeGuardType {
    fn from(value: &::treedom::interface::Interface) -> Self {
        match value {
            ::treedom::interface::Interface::Comment(..) => Self::Comment,
            ::treedom::interface::Interface::Doctype(..) => Self::Doctype,
            ::treedom::interface::Interface::Document(..) => Self::Document,
            ::treedom::interface::Interface::Element(..) => Self::Element,
            ::treedom::interface::Interface::ProcessingInstruction(..) => Self::Pi,
            ::treedom::interface::Interface::Text(..) => Self::Text,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeGuard {
    pub tree: Arc<parking_lot::Mutex<::treedom::IDTreeDOM>>,
    pub id: ::treedom::NodeId,
    pub type_: NodeGuardType,
}

impl NodeGuard {
    pub fn new(
        tree: Arc<parking_lot::Mutex<::treedom::IDTreeDOM>>,
        id: ::treedom::NodeId,
        type_: NodeGuardType,
    ) -> Self {
        Self { tree, id, type_ }
    }

    pub fn from_nodemut(
        tree: Arc<parking_lot::Mutex<::treedom::IDTreeDOM>>,
        mut node: ::treedom::NodeMut<'_>,
    ) -> Self {
        Self::new(tree, node.id(), NodeGuardType::from(&*node.value()))
    }

    pub fn from_noderef(
        tree: Arc<parking_lot::Mutex<::treedom::IDTreeDOM>>,
        node: ::treedom::NodeRef<'_>,
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

    pub fn into_any(self, py: pyo3::Python<'_>) -> pyo3::Py<pyo3::PyAny> {
        match &self.type_ {
            NodeGuardType::Document => pyo3::Py::new(py, PyDocument(self)).unwrap().into_any(),
            NodeGuardType::Comment => pyo3::Py::new(py, PyComment(self)).unwrap().into_any(),
            NodeGuardType::Doctype => pyo3::Py::new(py, PyDoctype(self)).unwrap().into_any(),
            NodeGuardType::Element => pyo3::Py::new(py, PyElement(self)).unwrap().into_any(),
            NodeGuardType::Text => pyo3::Py::new(py, PyText(self)).unwrap().into_any(),
            NodeGuardType::Pi => pyo3::Py::new(py, PyProcessingInstruction(self))
                .unwrap()
                .into_any(),
        }
    }
}

impl PartialEq for NodeGuard {
    fn eq(&self, other: &Self) -> bool {
        if self.type_ != other.type_ || !Arc::ptr_eq(&self.tree, &other.tree) {
            return false;
        }

        if self.id == other.id {
            return true;
        }

        let tree = self.tree.lock();

        let g1 = tree.get(self.id).unwrap();
        let g2 = tree.get(other.id).unwrap();

        g1.value() == g2.value()
    }
}
impl Eq for NodeGuard {}

macro_rules! create_richcmp_notimplemented {
    ($token:expr, $selfobj:expr) => {{
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            format!(
                "'{}' not implemented for '{}'",
                $token,
                crate::tools::get_type_name(&$selfobj),
            ),
        ))
    }};
}

pub(crate) use create_richcmp_notimplemented;

/// A document node
#[pyo3::pyclass(name = "Document", module = "markupever._rustlib", frozen)]
pub struct PyDocument(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyDocument {
    #[new]
    #[allow(unused_variables)]
    fn new(treedom: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        Err(
            pyo3::PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                "PyDocument does not have constructor",
            ),
        )
    }

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::Bound<Self>,
        other: pyo3::Py<pyo3::PyAny>,
        cmp: pyo3::basic::CompareOp,
    ) -> pyo3::PyResult<bool> {
        if matches!(cmp, pyo3::basic::CompareOp::Eq)
            && std::ptr::addr_eq(self_.as_ptr(), other.as_ptr())
        {
            return Ok(true);
        }

        match cmp {
            pyo3::basic::CompareOp::Eq => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        String::from("Document")
    }
}

/// A doctype node
#[pyo3::pyclass(name = "Doctype", module = "markupever._rustlib", frozen)]
pub struct PyDoctype(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyDoctype {
    #[new]
    fn new(
        treedom: &super::tree::PyTreeDom,
        name: String,
        public_id: String,
        system_id: String,
    ) -> pyo3::PyResult<Self> {
        let val = ::treedom::interface::DoctypeInterface::new(
            name.into(),
            public_id.into(),
            system_id.into(),
        );

        let mut dom = treedom.dom.lock();
        let node = dom.orphan(val.into());

        Ok(Self(NodeGuard::from_nodemut(treedom.dom.clone(), node)))
    }

    #[getter]
    fn name(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value().doctype().unwrap().name.to_string()
    }

    #[setter]
    fn set_name(&self, name: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().doctype_mut().unwrap().name = name.into();
    }

    #[getter]
    fn public_id(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value().doctype().unwrap().public_id.to_string()
    }

    #[setter]
    fn set_public_id(&self, public_id: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().doctype_mut().unwrap().public_id = public_id.into();
    }

    #[getter]
    fn system_id(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value().doctype().unwrap().system_id.to_string()
    }

    #[setter]
    fn set_system_id(&self, system_id: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().doctype_mut().unwrap().system_id = system_id.into();
    }

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::Bound<Self>,
        other: pyo3::Py<pyo3::PyAny>,
        cmp: pyo3::basic::CompareOp,
    ) -> pyo3::PyResult<bool> {
        if matches!(cmp, pyo3::basic::CompareOp::Eq)
            && std::ptr::addr_eq(self_.as_ptr(), other.as_ptr())
        {
            return Ok(true);
        }

        match cmp {
            pyo3::basic::CompareOp::Eq => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let doctype = node.value().doctype().unwrap();

        format!(
            "Doctype(name={:?}, public_id={:?}, system_id={:?})",
            &*doctype.name, &*doctype.public_id, &*doctype.system_id
        )
    }
}

/// A comment node
#[pyo3::pyclass(name = "Comment", module = "markupever._rustlib", frozen)]
pub struct PyComment(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyComment {
    #[new]
    fn new(treedom: &super::tree::PyTreeDom, content: String) -> pyo3::PyResult<Self> {
        let val = ::treedom::interface::CommentInterface::new(content.into());

        let mut dom = treedom.dom.lock();
        let node = dom.orphan(val.into());

        Ok(Self(NodeGuard::from_nodemut(treedom.dom.clone(), node)))
    }

    #[getter]
    fn content(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value().comment().unwrap().contents.to_string()
    }

    #[setter]
    fn set_content(&self, content: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().comment_mut().unwrap().contents = content.into();
    }

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::Bound<Self>,
        other: pyo3::Py<pyo3::PyAny>,
        cmp: pyo3::basic::CompareOp,
    ) -> pyo3::PyResult<bool> {
        if matches!(cmp, pyo3::basic::CompareOp::Eq)
            && std::ptr::addr_eq(self_.as_ptr(), other.as_ptr())
        {
            return Ok(true);
        }

        match cmp {
            pyo3::basic::CompareOp::Eq => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let comment = node.value().comment().unwrap();

        format!("Comment(content={:?})", &*comment.contents)
    }
}

/// A text node
#[pyo3::pyclass(name = "Text", module = "markupever._rustlib", frozen)]
pub struct PyText(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyText {
    #[new]
    fn new(treedom: &super::tree::PyTreeDom, content: String) -> pyo3::PyResult<Self> {
        let val = ::treedom::interface::TextInterface::new(content.into());

        let mut dom = treedom.dom.lock();
        let node = dom.orphan(val.into());

        Ok(Self(NodeGuard::from_nodemut(treedom.dom.clone(), node)))
    }

    #[getter]
    fn content(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value().text().unwrap().contents.to_string()
    }

    #[setter]
    fn set_content(&self, content: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().text_mut().unwrap().contents = content.into();
    }

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::Bound<Self>,
        other: pyo3::Py<pyo3::PyAny>,
        cmp: pyo3::basic::CompareOp,
    ) -> pyo3::PyResult<bool> {
        if matches!(cmp, pyo3::basic::CompareOp::Eq)
            && std::ptr::addr_eq(self_.as_ptr(), other.as_ptr())
        {
            return Ok(true);
        }

        match cmp {
            pyo3::basic::CompareOp::Eq => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let text = node.value().text().unwrap();

        format!("Text(content={:?})", &*text.contents)
    }
}

#[pyo3::pyclass(
    name = "AttrsListItems",
    module = "markupever._rustlib",
    mapping,
    frozen
)]
pub struct PyAttrsListItems {
    guard: NodeGuard,
    index: std::sync::atomic::AtomicUsize,
}

#[pyo3::pymethods]
impl PyAttrsListItems {
    #[new]
    #[pyo3(signature=(*args, **kwds))]
    #[allow(unused_variables)]
    fn new(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> pyo3::PyResult<Self> {
        Err(
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "You cannot create PyAttrsListItens instance directly; this structure is design only for communicating with element attributes."
            )
        )
    }

    fn __iter__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        self_
    }

    fn __next__<'py>(
        self_: pyo3::PyRef<'py, Self>,
        py: pyo3::Python<'py>,
    ) -> pyo3::PyResult<(super::qualname::PyQualName, pyo3::Py<pyo3::types::PyString>)> {
        let tree = self_.guard.tree.lock();
        let node = tree.get(self_.guard.id).unwrap().value().element().unwrap();

        let index = self_.index.load(std::sync::atomic::Ordering::Relaxed);
        let (attrkey, t_value) = match node.attrs.get(index) {
            Some(x) => x.clone(),
            None => return Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(())),
        };

        self_
            .index
            .store(index + 1, std::sync::atomic::Ordering::Relaxed);

        std::mem::drop(tree);

        let key = super::qualname::PyQualName {
            name: (*attrkey).clone(),
        };
        let val = pyo3::types::PyString::new(py, &t_value);
        Ok((key, val.unbind()))
    }

    fn __len__(&self) -> usize {
        let tree = self.guard.tree.lock();
        let node = tree.get(self.guard.id).unwrap().value().element().unwrap();

        node.attrs.len()
    }
}

fn repr_attrlist(element: &::treedom::interface::ElementInterface) -> String {
    let mut writer = String::from("[");

    let mut iter_ = element.attrs.iter();

    if let Some((key, val)) = iter_.next() {
        writer += &format!(
            "({}, {:?})",
            super::qualname::repr_qualname(key),
            val.as_ref()
        );
    }

    for (key, val) in iter_ {
        writer += &format!(
            ", ({}, {:?})",
            super::qualname::repr_qualname(key),
            val.as_ref()
        );
    }

    writer + "]"
}

/// This type is design for communicating with element attributes.
#[pyo3::pyclass(name = "AttrsList", module = "markupever._rustlib", frozen)]
pub struct PyAttrsList(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyAttrsList {
    #[new]
    #[pyo3(signature=(*args, **kwds))]
    #[allow(unused_variables)]
    fn new(
        args: &pyo3::Bound<'_, pyo3::types::PyTuple>,
        kwds: Option<&pyo3::Bound<'_, pyo3::types::PyDict>>,
    ) -> pyo3::PyResult<Self> {
        Err(
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "You cannot create PyAttrsList instance directly; this structure is design only for communicating with element attributes."
            )
        )
    }

    fn clear(&self) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();
        elem.attrs.clear();
        elem.attrs.shrink_to_fit();
    }

    fn insert(
        &self,
        index: usize,
        key: crate::tools::PyQualNameOrStr,
        value: &str,
    ) -> pyo3::PyResult<()> {
        let key = key.into_qualname();

        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();

        if index > elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "range out of bound",
            ));
        }

        elem.attrs.insert(index, (key.into(), value.into()));

        Ok(())
    }

    fn push(&self, key: crate::tools::PyQualNameOrStr, value: &str) {
        let key = key.into_qualname();

        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();

        elem.attrs.push((key.into(), value.into()));
    }

    fn items(self_: pyo3::PyRef<'_, Self>) -> PyAttrsListItems {
        PyAttrsListItems {
            guard: self_.0.clone(),
            index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn update_item(
        &self,
        index: usize,
        key: crate::tools::PyQualNameOrStr,
        value: &str,
    ) -> pyo3::PyResult<()> {
        let key = key.into_qualname();

        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();

        match elem.attrs.get_mut(index) {
            Some(x) => {
                x.0 = key.into();
                x.1 = value.into();
                Ok(())
            }
            None => Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "range out of bound",
            )),
        }
    }

    fn update_value(&self, index: usize, value: &str) -> pyo3::PyResult<()> {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();

        match elem.attrs.get_mut(index) {
            Some(x) => {
                x.1 = value.into();
                Ok(())
            }
            None => Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "range out of bound",
            )),
        }
    }

    fn get_by_index(
        self_: pyo3::PyRef<'_, Self>,
        index: usize,
    ) -> pyo3::PyResult<(super::qualname::PyQualName, pyo3::Py<pyo3::types::PyString>)> {
        let mut tree = self_.0.tree.lock();
        let mut node = tree.get_mut(self_.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();

        if index >= elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "range out of bound",
            ));
        }

        let (attrkey, value) = elem.attrs.get(index).unwrap();

        let key = super::qualname::PyQualName {
            name: attrkey.clone().into_qualname(),
        };
        let val = pyo3::types::PyString::new(self_.py(), value);
        std::mem::drop(tree);
        Ok((key, val.unbind()))
    }

    fn remove(
        self_: pyo3::PyRef<'_, Self>,
        index: usize,
    ) -> pyo3::PyResult<(super::qualname::PyQualName, pyo3::Py<pyo3::types::PyString>)> {
        let mut tree = self_.0.tree.lock();
        let mut node = tree.get_mut(self_.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();

        if index >= elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "range out of bound",
            ));
        }

        let (attrkey, value) = elem.attrs.remove(index);

        std::mem::drop(tree);

        let key = super::qualname::PyQualName {
            name: attrkey.into_qualname(),
        };
        let val = pyo3::types::PyString::new(self_.py(), &value);
        Ok((key, val.unbind()))
    }

    fn swap_remove(
        self_: pyo3::PyRef<'_, Self>,
        index: usize,
    ) -> pyo3::PyResult<(super::qualname::PyQualName, pyo3::Py<pyo3::types::PyString>)> {
        let mut tree = self_.0.tree.lock();
        let mut node = tree.get_mut(self_.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();

        if index >= elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "range out of bound",
            ));
        }

        let (attrkey, value) = elem.attrs.swap_remove(index);

        std::mem::drop(tree);

        let key = super::qualname::PyQualName {
            name: attrkey.into_qualname(),
        };
        let val = pyo3::types::PyString::new(self_.py(), &value);
        Ok((key, val.unbind()))
    }

    fn dedup(&self) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();
        elem.attrs.dedup();
        elem.attrs.shrink_to_fit();
    }

    fn __len__(&self) -> usize {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();
        elem.attrs.len()
    }

    fn reverse(&self) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        let elem = node.value().element_mut().unwrap();
        elem.attrs.reverse();
    }

    fn __repr__(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let elem = node.value().element().unwrap();

        repr_attrlist(elem)
    }
}

/// An element node
#[pyo3::pyclass(name = "Element", module = "markupever._rustlib", frozen)]
pub struct PyElement(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyElement {
    #[new]
    fn new(
        treedom: &super::tree::PyTreeDom,
        name: crate::tools::PyQualNameOrStr,
        attrs: Vec<(crate::tools::PyQualNameOrStr, pyo3::pybacked::PyBackedStr)>,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> Self {
        let name = name.into_qualname();

        let mut attributes = Vec::with_capacity(attrs.len());

        for (key, val) in attrs.into_iter() {
            let key = key.into_qualname();
            attributes.push((key, treedom::atomic::AtomicTendril::from(&*val)));
        }

        let val = ::treedom::interface::ElementInterface::new(
            name,
            attributes.into_iter(),
            template,
            mathml_annotation_xml_integration_point,
        );

        let mut dom = treedom.dom.lock();
        let node = dom.orphan(val.into());

        Self(NodeGuard::from_nodemut(treedom.dom.clone(), node))
    }

    #[getter]
    fn name(&self) -> super::qualname::PyQualName {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();

        super::qualname::PyQualName {
            name: node.value().element().unwrap().name.clone(),
        }
    }

    #[setter]
    fn set_name(&self, name: crate::tools::PyQualNameOrStr) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();

        let name = name.into_qualname();

        node.value().element_mut().unwrap().name = name;
    }

    #[getter]
    fn attrs(&self) -> PyAttrsList {
        PyAttrsList(self.0.clone())
    }

    #[setter]
    fn set_attrs(
        &self,
        attrs: Vec<(crate::tools::PyQualNameOrStr, pyo3::pybacked::PyBackedStr)>,
    ) -> pyo3::PyResult<()> {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();

        let mut attributes = Vec::with_capacity(attrs.len());

        for (key, val) in attrs.into_iter() {
            let key = key.into_qualname();
            attributes.push((
                treedom::interface::AttributeKey::from(key),
                treedom::atomic::AtomicTendril::from(&*val),
            ));
        }

        node.value()
            .element_mut()
            .unwrap()
            .attrs
            .replace(attributes);

        Ok(())
    }

    #[getter]
    fn template(&self) -> bool {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value().element().unwrap().template
    }

    #[setter]
    fn set_template(&self, template: bool) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().element_mut().unwrap().template = template;
    }

    #[getter]
    fn mathml_annotation_xml_integration_point(&self) -> bool {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value()
            .element()
            .unwrap()
            .mathml_annotation_xml_integration_point
    }

    #[setter]
    fn set_mathml_annotation_xml_integration_point(
        &self,
        mathml_annotation_xml_integration_point: bool,
    ) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value()
            .element_mut()
            .unwrap()
            .mathml_annotation_xml_integration_point = mathml_annotation_xml_integration_point;
    }

    fn id(&self) -> Option<String> {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let elem = node.value().element().unwrap();

        elem.attrs.id().map(String::from)
    }

    fn class_list(&self) -> Vec<String> {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let elem = node.value().element().unwrap();

        elem.attrs.class().iter().map(|x| x.to_string()).collect()
    }

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::Bound<Self>,
        other: pyo3::Py<pyo3::PyAny>,
        cmp: pyo3::basic::CompareOp,
    ) -> pyo3::PyResult<bool> {
        if matches!(cmp, pyo3::basic::CompareOp::Eq)
            && std::ptr::addr_eq(self_.as_ptr(), other.as_ptr())
        {
            return Ok(true);
        }

        match cmp {
            pyo3::basic::CompareOp::Eq => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let elem = node.value().element().unwrap();

        format!(
            "Element(name={}, attrs={}, template={}, integration_point={})",
            super::qualname::repr_qualname(&elem.name),
            repr_attrlist(elem),
            elem.template,
            elem.mathml_annotation_xml_integration_point
        )
    }
}

/// A processing instruction node
#[pyo3::pyclass(name = "ProcessingInstruction", module = "markupever._rustlib", frozen)]
pub struct PyProcessingInstruction(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyProcessingInstruction {
    #[new]
    fn new(treedom: &super::tree::PyTreeDom, data: String, target: String) -> pyo3::PyResult<Self> {
        let val =
            ::treedom::interface::ProcessingInstructionInterface::new(data.into(), target.into());

        let mut dom = treedom.dom.lock();
        let node = dom.orphan(val.into());

        Ok(Self(NodeGuard::from_nodemut(treedom.dom.clone(), node)))
    }

    #[getter]
    fn target(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value()
            .processing_instruction()
            .unwrap()
            .target
            .to_string()
    }

    #[setter]
    fn set_target(&self, target: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().processing_instruction_mut().unwrap().target = target.into();
    }

    #[getter]
    fn data(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value()
            .processing_instruction()
            .unwrap()
            .data
            .to_string()
    }

    #[setter]
    fn set_data(&self, data: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().processing_instruction_mut().unwrap().data = data.into();
    }

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::Py<pyo3::PyAny>> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::Bound<Self>,
        other: pyo3::Py<pyo3::PyAny>,
        cmp: pyo3::basic::CompareOp,
    ) -> pyo3::PyResult<bool> {
        if matches!(cmp, pyo3::basic::CompareOp::Eq)
            && std::ptr::addr_eq(self_.as_ptr(), other.as_ptr())
        {
            return Ok(true);
        }

        match cmp {
            pyo3::basic::CompareOp::Eq => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.get().0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let pi = node.value().processing_instruction().unwrap();

        format!(
            "ProcessingInstruction(data={:?}, target={:?})",
            &*pi.data, &*pi.target
        )
    }
}
