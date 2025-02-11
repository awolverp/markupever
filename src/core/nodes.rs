// **Node**:
// - new - does orphan
// - tree
// - parent
// - prev_sibling
// - next_sibling
// - first_child
// - last_child
// - has_sibling
// - has_children
// - richcmp
//
// **ElementAttrs** extends **list**:
// - ...
//
// **Element** extends **Node**:
// - name
// - attrs
// - template
// - mathml_annotation_xml_integration_point
use std::sync::Arc;

use pyo3::types::PyAnyMethods;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeGuardType {
    Document,
    Doctype,
    Comment,
    Text,
    Element,
    Pi,
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
pub(super) struct NodeGuard {
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

    pub fn from_pyobject(object: &pyo3::Bound<'_, pyo3::PyAny>) -> Result<Self, ()> {
        use pyo3::type_object::PyTypeInfo;

        if PyDocument::is_exact_type_of(object) {
            let x = unsafe {
                object
                    .extract::<pyo3::PyRef<'_, PyDocument>>()
                    .unwrap_unchecked()
            };
            Ok(x.0.clone())
        } else if PyDoctype::is_exact_type_of(object) {
            let x = unsafe {
                object
                    .extract::<pyo3::PyRef<'_, PyDoctype>>()
                    .unwrap_unchecked()
            };
            Ok(x.0.clone())
        } else if PyComment::is_exact_type_of(object) {
            let x = unsafe {
                object
                    .extract::<pyo3::PyRef<'_, PyComment>>()
                    .unwrap_unchecked()
            };
            Ok(x.0.clone())
        } else if PyText::is_exact_type_of(object) {
            let x = unsafe {
                object
                    .extract::<pyo3::PyRef<'_, PyText>>()
                    .unwrap_unchecked()
            };
            Ok(x.0.clone())
        } else if PyElement::is_exact_type_of(object) {
            let x = unsafe {
                object
                    .extract::<pyo3::PyRef<'_, PyElement>>()
                    .unwrap_unchecked()
            };
            Ok(x.0.clone())
        } else if PyProcessingInstruction::is_exact_type_of(object) {
            let x = unsafe {
                object
                    .extract::<pyo3::PyRef<'_, PyProcessingInstruction>>()
                    .unwrap_unchecked()
            };
            Ok(x.0.clone())
        } else {
            Err(())
        }
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

    pub fn into_any(self, py: pyo3::Python<'_>) -> pyo3::PyObject {
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

        let tree = self.tree.lock();

        let g1 = tree.get(self.id).unwrap();
        let g2 = tree.get(other.id).unwrap();

        g1.value() == g2.value()
    }
}
impl Eq for NodeGuard {}

macro_rules! _create_richcmp_notimplemented {
    ($token:expr, $selfobj:expr) => {
        unsafe {
            Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                format!(
                    "'{}' not implemented for '{}'",
                    $token,
                    crate::tools::get_type_name($selfobj.py(), $selfobj.as_ptr()),
                ),
            ))
        }
    };
}

#[pyo3::pyclass(name = "Document", module = "xmarkup._rustlib", frozen)]
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

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::PyRef<'_, Self>,
        other: pyo3::PyObject,
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

                Ok(self_.0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                _create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                _create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                _create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                _create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        String::from("[document]")
    }
}

#[pyo3::pyclass(name = "Doctype", module = "xmarkup._rustlib", frozen)]
pub struct PyDoctype(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyDoctype {
    #[new]
    fn new(
        treedom: &pyo3::Bound<'_, pyo3::PyAny>,
        name: String,
        public_id: String,
        system_id: String,
    ) -> pyo3::PyResult<Self> {
        let treedom = treedom
            .extract::<pyo3::PyRef<'_, super::tree::PyTreeDom>>()
            .map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected TreeDom for treedom, got {}",
                    unsafe { crate::tools::get_type_name(treedom.py(), treedom.as_ptr()) }
                ))
            })?;

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

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::PyRef<'_, Self>,
        other: pyo3::PyObject,
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

                Ok(self_.0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                _create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                _create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                _create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                _create_richcmp_notimplemented!(">=", self_)
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

#[pyo3::pyclass(name = "Comment", module = "xmarkup._rustlib", frozen)]
pub struct PyComment(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyComment {
    #[new]
    fn new(treedom: &pyo3::Bound<'_, pyo3::PyAny>, contents: String) -> pyo3::PyResult<Self> {
        let treedom = treedom
            .extract::<pyo3::PyRef<'_, super::tree::PyTreeDom>>()
            .map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected TreeDom for treedom, got {}",
                    unsafe { crate::tools::get_type_name(treedom.py(), treedom.as_ptr()) }
                ))
            })?;

        let val = ::treedom::interface::CommentInterface::new(contents.into());

        let mut dom = treedom.dom.lock();
        let node = dom.orphan(val.into());

        Ok(Self(NodeGuard::from_nodemut(treedom.dom.clone(), node)))
    }

    #[getter]
    fn contents(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value().comment().unwrap().contents.to_string()
    }

    #[setter]
    fn set_contents(&self, contents: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().comment_mut().unwrap().contents = contents.into();
    }

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::PyRef<'_, Self>,
        other: pyo3::PyObject,
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

                Ok(self_.0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                _create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                _create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                _create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                _create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let comment = node.value().comment().unwrap();

        format!("Comment(contents={:?})", &*comment.contents)
    }
}

#[pyo3::pyclass(name = "Text", module = "xmarkup._rustlib", frozen)]
pub struct PyText(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyText {
    #[new]
    fn new(treedom: &pyo3::Bound<'_, pyo3::PyAny>, contents: String) -> pyo3::PyResult<Self> {
        let treedom = treedom
            .extract::<pyo3::PyRef<'_, super::tree::PyTreeDom>>()
            .map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected TreeDom for treedom, got {}",
                    unsafe { crate::tools::get_type_name(treedom.py(), treedom.as_ptr()) }
                ))
            })?;

        let val = ::treedom::interface::TextInterface::new(contents.into());

        let mut dom = treedom.dom.lock();
        let node = dom.orphan(val.into());

        Ok(Self(NodeGuard::from_nodemut(treedom.dom.clone(), node)))
    }

    #[getter]
    fn contents(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        node.value().text().unwrap().contents.to_string()
    }

    #[setter]
    fn set_contents(&self, contents: String) {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();
        node.value().text_mut().unwrap().contents = contents.into();
    }

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::PyRef<'_, Self>,
        other: pyo3::PyObject,
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

                Ok(self_.0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                _create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                _create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                _create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                _create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __repr__(&self) -> String {
        let tree = self.0.tree.lock();
        let node = tree.get(self.0.id).unwrap();
        let text = node.value().text().unwrap();

        format!("Text(contents={:?})", &*text.contents)
    }
}

#[pyo3::pyclass(name = "AttrsList", module = "xmarkup._rustlib", mapping, frozen)]
pub struct PyAttrsList(pub(super) NodeGuard);

#[pyo3::pyclass(name = "Element", module = "xmarkup._rustlib", frozen)]
pub struct PyElement(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyElement {
    #[new]
    fn new(
        treedom: &pyo3::Bound<'_, pyo3::PyAny>,
        name: pyo3::PyObject,
        attrs: Vec<(pyo3::PyObject, pyo3::PyObject)>,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> pyo3::PyResult<Self> {
        let treedom = treedom
            .extract::<pyo3::PyRef<'_, super::tree::PyTreeDom>>()
            .map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected TreeDom for treedom, got {}",
                    unsafe { crate::tools::get_type_name(treedom.py(), treedom.as_ptr()) }
                ))
            })?;

        let name = crate::tools::qualname_from_pyobject(treedom.py(), &name).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "expected QualName or str for name, got {}",
                unsafe { crate::tools::get_type_name(treedom.py(), name.as_ptr()) }
            ))
        })?;

        let mut attributes = Vec::with_capacity(attrs.len());

        for (key, val) in attrs.into_iter() {
            let key = match crate::tools::qualname_from_pyobject(treedom.py(), &key) {
                Ok(x) => x,
                Err(_) => {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        format!("expected QualName or str for attrs #1, got {}", unsafe {
                            crate::tools::get_type_name(treedom.py(), key.as_ptr())
                        }),
                    ))
                }
            };

            let val = unsafe {
                if pyo3::ffi::PyUnicode_Check(val.as_ptr()) == 1 {
                    val.bind(treedom.py())
                        .extract::<String>()
                        .unwrap_unchecked()
                } else {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        format!(
                            "expected str for attrs #2, got {}",
                            crate::tools::get_type_name(treedom.py(), val.as_ptr())
                        ),
                    ));
                }
            };

            attributes.push((key, treedom::atomic::AtomicTendril::from(val)));
        }

        let val = ::treedom::interface::ElementInterface::new(
            name,
            attributes.into_iter(),
            template,
            mathml_annotation_xml_integration_point,
        );

        let mut dom = treedom.dom.lock();
        let node = dom.orphan(val.into());

        Ok(Self(NodeGuard::from_nodemut(treedom.dom.clone(), node)))
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
    fn set_name(&self, name: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();

        let name =
            crate::tools::qualname_from_pyobject(name.py(), name.as_unbound()).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected QualName or str for name, got {}",
                    unsafe { crate::tools::get_type_name(name.py(), name.as_ptr()) }
                ))
            })?;

        node.value().element_mut().unwrap().name = name;
        Ok(())
    }

    #[getter]
    fn attrs(&self) -> PyAttrsList {
        PyAttrsList(self.0.clone())
    }

    #[setter]
    fn set_attrs(
        &self,
        py: pyo3::Python<'_>,
        attrs: Vec<(pyo3::PyObject, pyo3::PyObject)>,
    ) -> pyo3::PyResult<()> {
        let mut tree = self.0.tree.lock();
        let mut node = tree.get_mut(self.0.id).unwrap();

        let mut attributes = Vec::with_capacity(attrs.len());

        for (key, val) in attrs.into_iter() {
            let key = match crate::tools::qualname_from_pyobject(py, &key) {
                Ok(x) => x,
                Err(_) => {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        format!("expected QualName or str for attrs #1, got {}", unsafe {
                            crate::tools::get_type_name(py, key.as_ptr())
                        }),
                    ))
                }
            };

            let val = unsafe {
                if pyo3::ffi::PyUnicode_Check(val.as_ptr()) == 1 {
                    val.bind(py).extract::<String>().unwrap_unchecked()
                } else {
                    return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        format!(
                            "expected str for attrs #2, got {}",
                            crate::tools::get_type_name(py, val.as_ptr())
                        ),
                    ));
                }
            };

            attributes.push((key, treedom::atomic::AtomicTendril::from(val)));
        }

        node.value()
            .element_mut()
            .unwrap()
            .attrs
            .replace(attributes.into_iter());
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

    fn tree(&self) -> super::tree::PyTreeDom {
        self.0.tree()
    }

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::PyRef<'_, Self>,
        other: pyo3::PyObject,
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

                Ok(self_.0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                _create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                _create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                _create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                _create_richcmp_notimplemented!(">=", self_)
            }
        }
    }
}

#[pyo3::pyclass(name = "ProcessingInstruction", module = "xmarkup._rustlib", frozen)]
pub struct PyProcessingInstruction(pub(super) NodeGuard);

#[pyo3::pymethods]
impl PyProcessingInstruction {
    #[new]
    fn new(
        treedom: &pyo3::Bound<'_, pyo3::PyAny>,
        data: String,
        target: String,
    ) -> pyo3::PyResult<Self> {
        let treedom = treedom
            .extract::<pyo3::PyRef<'_, super::tree::PyTreeDom>>()
            .map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected TreeDom for treedom, got {}",
                    unsafe { crate::tools::get_type_name(treedom.py(), treedom.as_ptr()) }
                ))
            })?;

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

    fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.parent().map(move |x| x.into_any(py))
    }

    fn prev_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.prev_sibling().map(move |x| x.into_any(py))
    }

    fn next_sibling(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.next_sibling().map(move |x| x.into_any(py))
    }

    fn first_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.first_child().map(move |x| x.into_any(py))
    }

    fn last_child(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
        self.0.last_child().map(move |x| x.into_any(py))
    }

    fn has_children(&self) -> bool {
        self.0.has_children()
    }

    fn has_siblings(&self) -> bool {
        self.0.has_siblings()
    }

    fn __richcmp__(
        self_: pyo3::PyRef<'_, Self>,
        other: pyo3::PyObject,
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

                Ok(self_.0 == other.0)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                Ok(self_.0 != other.0)
            }
            pyo3::basic::CompareOp::Gt => {
                _create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                _create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                _create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                _create_richcmp_notimplemented!(">=", self_)
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
