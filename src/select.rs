use std::sync::Arc;

struct PySelectInner {
    traverse: crate::iter::PyTraverse,
    expr: ::matching::ExpressionGroup,
}

impl PySelectInner {
    fn new(node: crate::nodes::NodeGuard, expr: String) -> pyo3::PyResult<Self> {
        let tree = node.tree.lock();
        let expr = ::matching::ExpressionGroup::new(&expr, Some(tree.namespaces()))
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        std::mem::drop(tree);

        Ok(Self {
            traverse: crate::iter::PyTraverse::from_nodeguard(node),
            expr,
        })
    }

    fn next_descendant(&mut self) -> Option<crate::nodes::NodeGuard> {
        while let Some((node, is_close)) = self.traverse.next_edge() {
            if is_close {
                continue;
            }

            return Some(node);
        }

        None
    }
}

impl Iterator for PySelectInner {
    type Item = crate::nodes::NodeGuard;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(guard) = self.next_descendant() {
            if !matches!(guard.type_, crate::nodes::NodeGuardType::Element) {
                continue;
            }

            let tree = guard.tree.lock();
            let node = tree.get(guard.id).unwrap();

            if self.expr.matches(
                unsafe { ::matching::CssNodeRef::new_unchecked(node) },
                None,
                &mut Default::default(),
            ) {
                std::mem::drop(tree);
                return Some(guard);
            }
        }

        None
    }
}

#[pyo3::pyclass(name = "Select", module = "markupever._rustlib", frozen)]
pub struct PySelect {
    inner: Arc<parking_lot::Mutex<PySelectInner>>,
}

#[pyo3::pymethods]
impl PySelect {
    #[new]
    fn new(node: &pyo3::Bound<'_, pyo3::PyAny>, expression: String) -> pyo3::PyResult<Self> {
        let node = crate::nodes::NodeGuard::from_pyobject(node).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "expected a node (such as Element, Text, Comment, ...) for node, got {}",
                unsafe { crate::tools::get_type_name(node.py(), node.as_ptr()) }
            ))
        })?;

        Ok(Self {
            inner: Arc::new(parking_lot::Mutex::new(PySelectInner::new(
                node, expression,
            )?)),
        })
    }

    fn __iter__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        self_
    }

    pub fn __next__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::PyObject> {
        let mut lock = self_.inner.lock();

        lock.next()
            .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()))
            .map(|x| x.into_any(self_.py()))
    }
}
