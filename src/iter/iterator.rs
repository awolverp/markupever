use pyo3::types::PyAnyMethods;
use std::sync::atomic;
use std::sync::Arc;

/// Iterates the nodes in insert order - don't matter which are orphan which not
#[pyo3::pyclass(name = "Iterator", module = "markupever._rustlib", frozen)]
pub struct PyIterator {
    dom: Arc<parking_lot::Mutex<::treedom::IDTreeDOM>>,
    index: atomic::AtomicUsize,
}

#[pyo3::pymethods]
impl PyIterator {
    #[new]
    fn new(dom: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let dom = dom
            .extract::<pyo3::PyRef<'_, crate::tree::PyTreeDom>>()
            .map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected TreeDom for dom, got {}",
                    unsafe { crate::tools::get_type_name(dom.py(), dom.as_ptr()) }
                ))
            })?;

        Ok(Self {
            dom: dom.dom.clone(),
            index: atomic::AtomicUsize::new(0),
        })
    }

    fn __iter__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        self_
    }

    fn __next__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = {
            let tree = self_.dom.lock();

            // NOTE:
            // Unfortunately the ego_tree crate does not let us to use directly usize for getting nodes.
            match tree
                .nodes()
                .nth(self_.index.load(atomic::Ordering::Relaxed))
            {
                Some(x) => crate::nodes::NodeGuard::from_noderef(self_.dom.clone(), x),
                None => return Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(())),
            }
        };

        self_.index.fetch_add(1, atomic::Ordering::Relaxed);
        Ok(node.into_any(self_.py()))
    }
}

macro_rules! axis_iterators {
    (
        $(
            #[$m:meta]
            $name:ident($f:path) as $pyname:expr;
        )*
    ) => {
        $(
            #[$m]
            #[pyo3::pyclass(name = $pyname, module = "markupever._rustlib")]
            pub struct $name {
                guard: Option<crate::nodes::NodeGuard>,
            }

            #[pyo3::pymethods]
            impl $name {
                #[new]
                fn new(node: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
                    let node = crate::nodes::NodeGuard::from_pyobject(node).map_err(|_| {
                        pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                            "expected a node (such as Element, Text, Comment, ...) for node, got {}",
                            unsafe { crate::tools::get_type_name(node.py(), node.as_ptr()) }
                        ))
                    })?;

                    Ok(Self { guard: $f(&node) })
                }

                fn __iter__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
                    self_
                }

                fn __next__(mut self_: pyo3::PyRefMut<'_, Self>) -> pyo3::PyResult<pyo3::PyObject> {
                    let node = self_.guard.take();
                    self_.guard = node.as_ref().and_then($f);

                    node.map(|x| x.into_any(self_.py()))
                        .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()))
                }
            }
        )*
    };
}

axis_iterators! {
    /// Iterates over ancestors (parents).
    PyAncestors(crate::nodes::NodeGuard::parent) as "Ancestors";

    /// Iterates over previous siblings.
    PyPrevSiblings(crate::nodes::NodeGuard::prev_sibling) as "PrevSiblings";

    /// Iterates over next siblings.
    PyNextSiblings(crate::nodes::NodeGuard::next_sibling) as "NextSiblings";

    /// Iterates over first children.
    PyFirstChildren(crate::nodes::NodeGuard::first_child) as "FirstChildren";

    /// Iterates over last children.
    PyLastChildren(crate::nodes::NodeGuard::last_child) as "LastChildren";
}

#[pyo3::pyclass(name = "Children", module = "markupever._rustlib")]
pub struct PyChildren {
    front: Option<crate::nodes::NodeGuard>,
    back: Option<crate::nodes::NodeGuard>,
}

#[pyo3::pymethods]
impl PyChildren {
    #[new]
    fn new(node: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
        let node = crate::nodes::NodeGuard::from_pyobject(node).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "expected a node (such as Element, Text, Comment, ...) for node, got {}",
                unsafe { crate::tools::get_type_name(node.py(), node.as_ptr()) }
            ))
        })?;

        let front = node.first_child();
        let back = node.last_child();

        Ok(Self { front, back })
    }

    fn __iter__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        self_
    }

    fn __next__(mut self_: pyo3::PyRefMut<'_, Self>) -> pyo3::PyResult<pyo3::PyObject> {
        let mut is_same = false;

        if let (Some(x), Some(y)) = (&self_.front, &self_.back) {
            if x.id == y.id {
                is_same = true;
            }
        }

        let node = {
            if is_same {
                let node = self_.front.take();
                self_.back = None;
                node
            } else {
                let node = self_.front.take();
                self_.front = node
                    .as_ref()
                    .and_then(crate::nodes::NodeGuard::next_sibling);
                node
            }
        };

        node.map(|x| x.into_any(self_.py()))
            .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()))
    }
}
