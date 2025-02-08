// **TreeDom**:
// - new
// - namespaces
// - with_capacity
// - root
// - append
// - prepend
// - insert_before
// - insert_after
// - detach
// - reparent_append
// - reparent_prepend
//
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
// - hash
//
// **Document** extends **Node**
//
// **Doctype** extends **Node**:
// - name
// - public_id
// - system_id
//
// **Comment** extends **Node**:
// - contents
//
// **Text** extends **Node**:
// - contents
//
// **ElementAttrs** extends **list**:
// - ...
//
// **Element** extends **Node**:
// - name
// - attrs
// - template
// - mathml_annotation_xml_integration_point
//
// **ProcessingInstruction** extends **Node**:
// - data
// - target
use pyo3::types::PyAnyMethods;
use std::sync::Arc;

#[pyo3::pyclass(name = "TreeDom", module = "xmarkup._rustlib", frozen)]
pub struct PyTreeDom {
    pub(super) tree: Arc<parking_lot::Mutex<::treedom::TreeDom>>,
}

impl PyTreeDom {
    pub fn from_treedom(dom: ::treedom::TreeDom) -> Self {
        Self {
            tree: Arc::new(parking_lot::Mutex::new(dom)),
        }
    }

    pub fn from_arc_mutex(dom: Arc<parking_lot::Mutex<::treedom::TreeDom>>) -> Self {
        Self { tree: dom }
    }
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
        let mut ns = ::treedom::NamespaceMap::new();

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
            ::treedom::ego_tree::Tree::new(::treedom::data::Document.into())
        } else {
            ::treedom::ego_tree::Tree::with_capacity(::treedom::data::Document.into(), capacity)
        };

        Ok(Self::from_treedom(::treedom::TreeDom::new(dom, ns)))
    }

    fn namespaces<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<pyo3::Bound<'a, pyo3::PyAny>> {
        use pyo3::types::{PyDict, PyDictMethods};

        let dict = PyDict::new(py);

        let dom = self.tree.lock();

        for (key, val) in dom.namespaces().iter() {
            dict.set_item(key.to_string(), val.to_string())?;
        }

        Ok(dict.into_any())
    }

    fn root(&self) -> super::nodes::PyDocument {
        let root_id = self.tree.lock().root().id();
        super::nodes::PyDocument(super::nodes::NodeGuard::new(
            self.tree.clone(),
            root_id,
            super::nodes::NodeGuardType::Document,
        ))
    }

    fn __str__(&self) -> String {
        let dom = self.tree.lock();
        format!("{}", dom)
    }

    fn __repr__(&self) -> String {
        let dom = self.tree.lock();
        format!("{:#?}", dom)
    }
}

// pub trait NodeImplementation: Sized {
//     /// Mustn't lock the tree
//     fn from_nodemut(
//         tree: Arc<parking_lot::Mutex<::treedom::TreeDom>>,
//         node: ::treedom::ego_tree::NodeMut<'_, ::treedom::data::NodeData>,
//     ) -> Self;
// }

// #[pyo3::pyclass(name = "Document", module = "xmarkup._rustlib", frozen)]
// pub struct PyDocument {
//     tree: Arc<parking_lot::Mutex<::treedom::TreeDom>>,
//     id: ::treedom::ego_tree::NodeId,
// }

// impl NodeImplementation for PyDocument {
//     fn from_nodemut(
//         tree: Arc<parking_lot::Mutex<treedom::TreeDom>>,
//         mut node: treedom::ego_tree::NodeMut<'_, treedom::data::NodeData>,
//     ) -> Self {
//         assert!(node.value().is_document());
//         Self {
//             tree,
//             id: node.id(),
//         }
//     }
// }

// #[pyo3::pymethods]
// impl PyDocument {
//     #[new]
//     fn new(treedom: pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<Self> {
//         {
//             let runtime_warning = treedom
//                 .py()
//                 .get_type::<pyo3::exceptions::PyRuntimeWarning>();

//             pyo3::PyErr::warn(
//                 treedom.py(),
//                 &runtime_warning,
//                 pyo3::ffi::c_str!("This is not recommended to orphan a Document to a tree. this may cause undefined errors in runtime."),
//                 1,
//             )?;
//         }

//         let treedom = treedom
//             .extract::<pyo3::PyRef<'_, PyTreeDom>>()
//             .map_err(|_| {
//                 pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
//                     "expected TreeDom for treedom, got {}",
//                     unsafe { crate::tools::get_type_name(treedom.py(), treedom.as_ptr()) }
//                 ))
//             })?;

//         let mut tree = treedom.tree.lock();
//         let slf_ = tree.tree_mut().orphan(::treedom::data::Document.into());

//         Ok(Self::from_nodemut(treedom.tree.clone(), slf_))
//     }

//     fn tree(&self) -> PyTreeDom {
//         PyTreeDom::from_arc_mutex(self.tree.clone())
//     }

//     fn __hash__(&self) -> u64 {
//         let tree = self.tree.lock();
//         let node = tree.get(self.id).unwrap();

//         let mut state = std::hash::DefaultHasher::new();
//         std::hash::Hash::hash(node.value(), &mut state);
//         state.finish()
//     }
// }
