enum EdgeSign {
    Open(crate::nodes::NodeGuard),
    Close(crate::nodes::NodeGuard),
}

/// An iterator which traverses a subtree.
#[pyo3::pyclass(name = "Traverse", module = "markupever._rustlib")]
pub struct PyTraverse {
    root: Option<crate::nodes::NodeGuard>,
    edge: Option<EdgeSign>,
}

impl PyTraverse {
    pub fn from_nodeguard(node: crate::nodes::NodeGuard) -> Self {
        Self {
            root: Some(node),
            edge: None,
        }
    }

    pub fn next_edge(&mut self) -> Option<(crate::nodes::NodeGuard, bool)> {
        match &self.edge {
            None => {
                if let Some(root) = &self.root {
                    self.edge = Some(EdgeSign::Open(root.clone()));
                }
            }
            Some(EdgeSign::Open(node)) => {
                if let Some(first_child) = node.first_child() {
                    self.edge = Some(EdgeSign::Open(first_child));
                } else {
                    self.edge = Some(EdgeSign::Close(node.clone()));
                }
            }
            Some(EdgeSign::Close(node)) => {
                if self.root.as_ref().is_some_and(|x| x.id == node.id) {
                    self.root = None;
                    self.edge = None;
                } else if let Some(next_sibling) = node.next_sibling() {
                    self.edge = Some(EdgeSign::Open(next_sibling));
                } else {
                    self.edge = node.parent().map(EdgeSign::Close);
                }
            }
        }

        match &self.edge {
            Some(EdgeSign::Open(x)) => Some((x.clone(), false)),
            Some(EdgeSign::Close(x)) => Some((x.clone(), true)),
            None => None,
        }
    }
}

#[pyo3::pymethods]
impl PyTraverse {
    #[new]
    fn new(node: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let node = crate::nodes::NodeGuard::from_pyobject(node).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "expected a node (such as Element, Text, Comment, ...) for node, got {}",
                unsafe { crate::tools::get_type_name(node.py(), node.as_ptr()) }
            ))
        })?;

        Ok(Self::from_nodeguard(node))
    }

    fn __iter__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        self_
    }

    pub fn __next__(mut self_: pyo3::PyRefMut<'_, Self>) -> pyo3::PyResult<(pyo3::PyObject, bool)> {
        let py = self_.py();
        match self_.next_edge() {
            Some((x, y)) => Ok((x.into_any(py), y)),
            None => Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(())),
        }
    }
}

/// An iterator over a node and its descendants.
#[pyo3::pyclass(name = "Descendants", module = "markupever._rustlib")]
pub struct PyDescendants(PyTraverse);

#[pyo3::pymethods]
impl PyDescendants {
    #[new]
    fn new(node: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let node = crate::nodes::NodeGuard::from_pyobject(node).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "expected a node (such as Element, Text, Comment, ...) for node, got {}",
                unsafe { crate::tools::get_type_name(node.py(), node.as_ptr()) }
            ))
        })?;

        Ok(Self(PyTraverse {
            root: Some(node),
            edge: None,
        }))
    }

    fn __iter__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        self_
    }

    fn __next__(mut self_: pyo3::PyRefMut<'_, Self>) -> pyo3::PyResult<pyo3::PyObject> {
        let py = self_.py();

        while let Some((node, is_close)) = self_.0.next_edge() {
            if is_close {
                continue;
            }

            return Ok(node.into_any(py));
        }

        Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()))
    }
}
