use super::docdata;
use super::elementdata;
use super::utils::{get_node_from_pyobject, make_repr};
use crate::core::arcdom;
use crate::core::matching;

/// Children vector of a node
#[pyo3::pyclass(name = "NodeChildren", module = "markupselect._rustlib", frozen)]
pub struct PyNodeChildren {
    node: arcdom::Node,
    len: std::sync::atomic::AtomicUsize,
    index: std::sync::atomic::AtomicUsize,
}

#[pyo3::pymethods]
impl PyNodeChildren {
    #[new]
    pub(super) fn new() -> pyo3::PyResult<Self> {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Use Node.children() method; don't use this constructor directly.",
        ))
    }

    /// Returns `len(self)` - length of the attributes vector.
    pub(super) fn __len__(&self) -> usize {
        self.node.children().len()
    }

    /// Returns `bool(self)` - `true` if the vector is not empty
    pub(super) fn __bool__(&self) -> bool {
        !self.node.children().is_empty()
    }

    /// Clears the attributes vector
    pub(super) fn clear(&self) {
        self.node.children().clear();
    }

    pub(super) fn append(&self, py: pyo3::Python<'_>, node: pyo3::PyObject) -> pyo3::PyResult<()> {
        let n = get_node_from_pyobject(node.bind(py))?;

        self.node
            .children()
            .push(n)
            .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string()))
    }

    pub(super) fn pop(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let n = self.node.children().pop().ok_or_else(|| {
            pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>("pop from empty children")
        })?;

        let n = PyNode(n);
        Ok(pyo3::Py::new(py, n)?.into_any())
    }

    /// Returns `self[index]`
    pub(super) fn __getitem__(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let children = self.node.children();

        let n = match children.get(index) {
            Some(x) => PyNode(x.clone()),
            None => {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                    "out of range",
                ))
            }
        };

        Ok(pyo3::Py::new(py, n)?.into_any())
    }

    /// Performs `self[index] = Node`
    pub(super) fn __setitem__(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
        value: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let node = get_node_from_pyobject(value.bind(py))?;
        let mut children = self.node.children();

        if index >= children.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        children
            .insert(index + 1, node)
            .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string()))?;
        children.remove(index);

        Ok(())
    }

    /// Performs `del self[index]`
    pub(super) fn __delitem__(&self, index: usize) -> pyo3::PyResult<()> {
        let mut children = self.node.children();

        if index >= children.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        children.remove(index);
        Ok(())
    }

    /// Return first index of value.
    ///
    /// Raises ValueError if the value is not present.
    #[pyo3(signature=(value, start=0))]
    pub(super) fn index(
        &self,
        py: pyo3::Python<'_>,
        value: pyo3::PyObject,
        start: usize,
    ) -> pyo3::PyResult<usize> {
        let node = get_node_from_pyobject(value.bind(py))?;
        let children = self.node.children();

        let mut iter = children.iter();

        if start > 0 {
            iter.skip(start - 1)
                .position(|x| x.ptr_eq(&node))
                .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(()))
        } else {
            iter.position(|x| x.ptr_eq(&node))
                .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(()))
        }
    }

    /// Inserts a child at position `index`.
    ///
    /// Returns error if the child has parent for itself.
    /// Also returns error if child cycle be detected.
    pub(super) fn insert(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
        value: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let node = get_node_from_pyobject(value.bind(py))?;
        let mut children = self.node.children();

        if index > children.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        children
            .insert(index, node)
            .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(x.to_string()))
    }

    /// Returns `iter(self)`
    ///
    /// Note that you cannot have multiple `iter(self)` in a same time.
    /// each one must be done before creating next one.
    pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::PyRef<'_, Self>> {
        if slf.len.load(std::sync::atomic::Ordering::Relaxed) != 0 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "you can only call PyNodeChildren's __iter__() once in a time.",
            ));
        }

        slf.index.store(0, std::sync::atomic::Ordering::Relaxed);
        slf.len
            .store(slf.__len__(), std::sync::atomic::Ordering::Relaxed);
        Ok(slf)
    }

    /// Returns `next(self)`
    pub fn __next__(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        let children = slf.node.children();

        if slf.len.load(std::sync::atomic::Ordering::Relaxed) != children.len() {
            std::mem::drop(children);
            slf.len.store(0, std::sync::atomic::Ordering::Relaxed);
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "node attrs size changed during iteration",
            ));
        }

        if slf.index.load(std::sync::atomic::Ordering::Relaxed) >= children.len() {
            std::mem::drop(children);
            slf.len.store(0, std::sync::atomic::Ordering::Relaxed);
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()));
        }

        let n = &children[slf.index.load(std::sync::atomic::Ordering::Relaxed)];
        let n = PyNode(n.clone());

        std::mem::drop(children);
        slf.index.store(
            slf.index.load(std::sync::atomic::Ordering::Relaxed) + 1,
            std::sync::atomic::Ordering::Relaxed,
        );
        Ok(pyo3::Py::new(py, n)?.into_ptr())
    }
}

/// Children vector of a node
#[pyo3::pyclass(name = "TreeIterator", module = "markupselect._rustlib")]
pub struct PyTreeIterator(arcdom::iter::TreeIterator);

#[pyo3::pymethods]
impl PyTreeIterator {
    #[new]
    pub fn new() -> pyo3::PyResult<Self> {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Use Node.tree() method; don't use this constructor directly.",
        ))
    }

    pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf
    }

    pub fn __next__(
        mut slf: pyo3::PyRefMut<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        match slf.0.next() {
            None => Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(())),
            Some(node) => {
                let node = PyNode(node);
                Ok(pyo3::Py::new(py, node)?.into_any())
            }
        }
    }
}

/// Children vector of a node
#[pyo3::pyclass(name = "ParentsIterator", module = "markupselect._rustlib")]
pub struct PyParentsIterator(arcdom::iter::ParentsIterator);

#[pyo3::pymethods]
impl PyParentsIterator {
    #[new]
    pub fn new() -> pyo3::PyResult<Self> {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Use Node.tree() method; don't use this constructor directly.",
        ))
    }

    pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf
    }

    pub fn __next__(
        mut slf: pyo3::PyRefMut<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        match slf.0.next() {
            None => Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(())),
            Some(node) => {
                let node = PyNode(node);
                Ok(pyo3::Py::new(py, node)?.into_any())
            }
        }
    }
}


/// Children vector of a node
#[pyo3::pyclass(name = "SelectExpr", module = "markupselect._rustlib")]
pub struct PySelectExpr(matching::Select);

#[pyo3::pymethods]
impl PySelectExpr {
    #[new]
    pub fn new() -> pyo3::PyResult<Self> {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Use Node.select() method; don't use this constructor directly.",
        ))
    }

    pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        slf
    }

    pub fn __next__(
        mut slf: pyo3::PyRefMut<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        match slf.0.next() {
            None => Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(())),
            Some(node) => {
                let node = PyNode(node);
                Ok(pyo3::Py::new(py, node)?.into_any())
            }
        }
    }
}


/// A node of DOM
#[pyo3::pyclass(name = "Node", module = "markupselect._rustlib", frozen)]
pub struct PyNode(pub arcdom::Node);

#[pyo3::pymethods]
impl PyNode {
    #[new]
    #[pyo3(signature=(data, /))]
    pub(super) fn new(py: pyo3::Python<'_>, data: pyo3::PyObject) -> pyo3::PyResult<Self> {
        let data = data.bind(py);

        Ok(Self(get_node_from_pyobject(data)?))
    }

    /// Returns the node data as `Py*Data` classes
    pub(super) fn data(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let data = self.0.as_enum();

        let result = match &*data {
            arcdom::NodeData::Document(..) => {
                let r = pyo3::Py::new(py, docdata::PyDocumentData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Doctype(..) => {
                let r = pyo3::Py::new(py, docdata::PyDoctypeData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Text(..) => {
                let r = pyo3::Py::new(py, docdata::PyTextData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Comment(..) => {
                let r = pyo3::Py::new(py, docdata::PyCommentData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Element(..) => {
                let r = pyo3::Py::new(py, elementdata::PyElementData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::ProcessingInstruction(..) => {
                let r = pyo3::Py::new(py, docdata::PyProcessingInstructionData(self.0.clone()))?;
                r.into_any()
            }
        };

        Ok(result)
    }

    /// Returns `True` if the node is a document
    pub(super) fn is_document(&self) -> bool {
        self.0.is_document()
    }

    /// Returns `True` if the node is a doctype
    pub(super) fn is_doctype(&self) -> bool {
        self.0.is_doctype()
    }

    /// Returns `True` if the node is a comment
    pub(super) fn is_comment(&self) -> bool {
        self.0.is_comment()
    }

    /// Returns `True` if the node is a text
    pub(super) fn is_text(&self) -> bool {
        self.0.is_text()
    }

    /// Returns `True` if the node is an element
    pub(super) fn is_element(&self) -> bool {
        self.0.is_element()
    }

    /// Returns `True` if the node is a processing instruction
    pub(super) fn is_processing_instruction(&self) -> bool {
        self.0.is_processing_instruction()
    }

    /// Returns the parent if exist
    pub(super) fn parent(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<Option<pyo3::PyObject>> {
        let parent = self.0.parent();

        if parent.is_none() {
            return Ok(None);
        }

        let parent = unsafe {
            parent
                .clone()
                .unwrap_unchecked()
                .upgrade()
                .expect("dangling weak pointer")
        };

        Ok(Some(
            pyo3::Py::new(py, PyNode(parent)).map(|x| x.into_any())?,
        ))
    }

    /// Copies the `self` and returns a new one
    pub(super) fn copy(&self) -> Self {
        Self(arcdom::Node::copy(&self.0))
    }

    pub(super) fn children(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let children = PyNodeChildren {
            node: self.0.clone(),
            index: std::sync::atomic::AtomicUsize::new(0),
            len: std::sync::atomic::AtomicUsize::new(0),
        };

        Ok(pyo3::Py::new(py, children)?.into_any())
    }

    #[pyo3(signature=(include_self=true))]
    pub(super) fn tree(
        &self,
        py: pyo3::Python<'_>,
        include_self: bool,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let obj = {
            if include_self {
                PyTreeIterator(self.0.clone().into_tree())
            } else {
                PyTreeIterator(self.0.tree())
            }
        };

        Ok(pyo3::Py::new(py, obj)?.into_any())
    }

    #[pyo3(signature=(include_self=true))]
    pub(super) fn parents(
        &self,
        py: pyo3::Python<'_>,
        include_self: bool,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let obj = {
            if include_self {
                PyParentsIterator(self.0.clone().into_parents())
            } else {
                PyParentsIterator(self.0.parents())
            }
        };

        Ok(pyo3::Py::new(py, obj)?.into_any())
    }

    #[pyo3(signature=(include_self=true))]
    pub(super) fn serialize_html(&self, include_self: bool) -> pyo3::PyResult<Vec<u8>> {
        let mut writer = Vec::new();

        self.0
            .serialize_html(&mut writer, include_self)
            .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(x.to_string()))?;

        Ok(writer)
    }

    #[pyo3(signature=(include_self=true))]
    pub(super) fn serialize_xml(&self, include_self: bool) -> pyo3::PyResult<Vec<u8>> {
        let mut writer = Vec::new();

        self.0
            .serialize_xml(&mut writer, include_self)
            .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(x.to_string()))?;

        Ok(writer)
    }

    pub(super) fn __eq__(
        &self,
        py: pyo3::Python<'_>,
        other: pyo3::PyObject,
    ) -> pyo3::PyResult<bool> {
        let other = get_node_from_pyobject(other.bind(py))?;
        Ok(self.0.eq(&other))
    }

    pub(super) fn __repr__(&self) -> String {
        let data = self.0.as_enum();
        format!("Node({})", make_repr(&data))
    }

    pub(super) fn select(&self, py: pyo3::Python<'_>, expr: String) -> pyo3::PyResult<pyo3::PyObject> {
        let expr = matching::Select::new(self.0.tree(), &expr).map_err(|err| {
            pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(err.to_string())
        })?;

        Ok(pyo3::Py::new(py, PySelectExpr(expr))?.into_any())
    }
}
