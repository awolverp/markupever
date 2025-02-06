//! **TreeDom**:
//! - new
//! - namespaces
//! - with_capacity
//! - root
//! - append
//! - prepend
//! - insert_before
//! - insert_after
//! - detach
//! - reparent_append
//! - reparent_prepend
//!
//! **Node**:
//! - new
//! - tree
//! - parent
//! - prev_sibling
//! - next_sibling
//! - first_child
//! - last_child
//! - has_sibling
//! - has_children
//! - serialize
//! - copy
//! - hash
//!
//! **Document** extends **Node**
//!
//! **Doctype** extends **Node**:
//! - name
//! - public_id
//! - system_id
//!
//! **Comment** extends **Node**:
//! - contents
//!
//! **Text** extends **Node**:
//! - contents
//!
//! **ElementAttrs** extends **list**:
//! - ...
//!
//!
//! **Element** extends **Node**:
//! - name
//! - attrs
//! - template
//! - mathml_annotation_xml_integration_point
//!
//! **ProcessingInstruction** extends **Node**:
//! - data
//! - target
//!
use std::sync::Arc;

#[pyo3::pyclass(name = "TreeDom", module = "xmarkup._rustlib", frozen)]
pub struct PyTreeDom {
    dom: Arc<parking_lot::Mutex<treedom::TreeDom>>,
}

#[pyo3::pymethods]
impl PyTreeDom {
    #[new]
    #[classmethod]
    #[pyo3(signature=(*, namespaces=None))]
    fn new(
        cls: pyo3::Bound<'_, pyo3::types::PyType>,
        namespaces: Option<pyo3::PyObject>,
    ) -> pyo3::PyResult<Self> {
        Self::with_capacity(cls, 0, namespaces)
    }

    #[classmethod]
    #[pyo3(signature=(capacity, *, namespaces=None))]
    fn with_capacity(
        cls: pyo3::Bound<'_, pyo3::types::PyType>,
        capacity: usize,
        namespaces: Option<pyo3::PyObject>,
    ) -> pyo3::PyResult<Self> {
        use pyo3::types::PyAnyMethods;

        let mut ns = treedom::NamespaceMap::new();

        if let Some(namespaces) = namespaces {
            let namespaces = namespaces
                .bind(cls.py())
                .downcast::<pyo3::types::PyDict>()
                .map_err(|_| {
                    pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                        "expected dict[str, str] for namespaces, got {}",
                        unsafe { crate::tools::get_type_name(cls.py(), namespaces.as_ptr()) }
                    ))
                })?;

            for (key, val) in pyo3::types::PyDictMethods::iter(namespaces) {
                let key = key.downcast::<pyo3::types::PyString>().map_err(|_| {
                    pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                        "expected dict[str, str] for namespaces, but found a key with type {} (keys must be strings)",
                        unsafe { crate::tools::get_type_name(cls.py(), key.as_ptr()) }
                    ))
                }).map(|x| pyo3::types::PyStringMethods::to_string_lossy(x).into_owned())?;

                let val = val.downcast::<pyo3::types::PyString>().map_err(|_| {
                    pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                        "expected dict[str, str] for namespaces, but found a value with type {} (values must be strings)",
                        unsafe { crate::tools::get_type_name(cls.py(), val.as_ptr()) }
                    ))
                }).map(|x| pyo3::types::PyStringMethods::to_string_lossy(x).into_owned())?;

                ns.insert(key.into(), val.into());
            }
        }

        let dom = if capacity == 0 {
            treedom::ego_tree::Tree::new(treedom::data::Document.into())
        } else {
            treedom::ego_tree::Tree::with_capacity(treedom::data::Document.into(), capacity)
        };

        Ok(Self {
            dom: Arc::new(parking_lot::Mutex::new(treedom::TreeDom::new(
                dom,
                Vec::new(),
                treedom::markup5ever::interface::QuirksMode::NoQuirks,
                ns,
                0,
            ))),
        })
    }

    fn namespaces<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<pyo3::Bound<'a, pyo3::PyAny>> {
        use pyo3::types::{PyDict, PyDictMethods};

        let dict = PyDict::new(py);

        let dom = self.dom.lock();

        for (key, val) in dom.namespaces().iter() {
            dict.set_item(key.to_string(), val.to_string())?;
        }

        Ok(dict.into_any())
    }

    fn __str__(&self) -> String {
        let dom = self.dom.lock();
        format!("{}", dom)
    }

    fn __repr__(&self) -> String {
        let dom = self.dom.lock();
        format!("{:#?}", dom)
    }
}
