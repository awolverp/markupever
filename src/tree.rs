use pyo3::types::PyAnyMethods;
use std::sync::Arc;

#[pyo3::pyclass(name = "TreeDom", module = "markupever._rustlib", frozen)]
pub struct PyTreeDom {
    pub(super) dom: Arc<parking_lot::Mutex<::treedom::IDTreeDOM>>,
}

impl PyTreeDom {
    #[inline(always)]
    pub fn from_treedom(dom: ::treedom::IDTreeDOM) -> Self {
        Self {
            dom: Arc::new(parking_lot::Mutex::new(dom)),
        }
    }

    #[inline(always)]
    pub fn from_arc_mutex(dom: Arc<parking_lot::Mutex<::treedom::IDTreeDOM>>) -> Self {
        Self { dom }
    }

    #[inline]
    fn add_new_namespace(
        &self,
        mut lock: ::parking_lot::MutexGuard<'_, ::treedom::IDTreeDOM>,
        id: ::treedom::NodeId,
    ) {
        let child = lock.get(id).unwrap();

        if let Some(elem) = child.value().element() {
            if let Some(prefix) = elem.name.prefix.clone() {
                let ns = elem.name.ns.clone();

                lock.namespaces_mut().insert(prefix, ns);
            } else if lock.namespaces().is_empty() && !elem.name.ns.is_empty() {
                let ns = elem.name.ns.clone();

                lock.namespaces_mut()
                    .insert(::treedom::markup5ever::Prefix::from(""), ns);
            }
        }
    }

    #[inline]
    fn remove_old_namespace(
        &self,
        mut lock: ::parking_lot::MutexGuard<'_, ::treedom::IDTreeDOM>,
        id: ::treedom::NodeId,
    ) {
        let child = lock.get(id).unwrap();

        if let Some(elem) = child.value().element() {
            if let Some(prefix) = elem.name.prefix.clone() {
                lock.namespaces_mut().remove(&prefix);
            }
        }
    }
}

#[pyo3::pymethods]
impl PyTreeDom {
    /// Creates a new [`PyTreeDom`]
    #[new]
    #[classmethod]
    #[pyo3(signature=(*, namespaces=None))]
    fn new(
        cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        namespaces: Option<pyo3::PyObject>,
    ) -> pyo3::PyResult<Self> {
        Self::with_capacity(cls, 0, namespaces)
    }

    /// Creates a new [`PyTreeDom`] with the specified capacity.
    #[classmethod]
    #[pyo3(signature=(capacity, *, namespaces=None))]
    fn with_capacity(
        cls: &pyo3::Bound<'_, pyo3::types::PyType>,
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
            ::treedom::IDTreeDOM::new(::treedom::interface::DocumentInterface, ns)
        } else {
            ::treedom::IDTreeDOM::with_capacity(
                ::treedom::interface::DocumentInterface,
                ns,
                capacity,
            )
        };

        Ok(Self {
            dom: Arc::new(parking_lot::Mutex::new(dom)),
        })
    }

    /// Returns the available namespaces in DOM as a `dict`.
    fn namespaces<'a>(&self, py: pyo3::Python<'a>) -> pyo3::PyResult<pyo3::Bound<'a, pyo3::PyAny>> {
        use pyo3::types::{PyDict, PyDictMethods};

        let dict = PyDict::new(py);

        let dom = self.dom.lock();

        for (key, val) in dom.namespaces().iter() {
            dict.set_item(key.to_string(), val.to_string())?;
        }

        Ok(dict.into_any())
    }

    /// Returns the root node (always is PyDocument).
    fn root(&self) -> super::nodes::PyDocument {
        let root_id = self.dom.lock().root().id();
        super::nodes::PyDocument(super::nodes::NodeGuard::new(
            self.dom.clone(),
            root_id,
            super::nodes::NodeGuardType::Document,
        ))
    }

    fn append(
        self_: pyo3::PyRef<'_, Self>,
        parent: pyo3::PyObject,
        child: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let parent =
            super::nodes::NodeGuard::from_pyobject(parent.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for parent, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), parent.as_ptr()) }
                ))
            })?;

        let child =
            super::nodes::NodeGuard::from_pyobject(child.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for child, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), child.as_ptr()) }
                ))
            })?;

        if !Arc::ptr_eq(&self_.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self_.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        let mut tree = self_.dom.lock();
        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.append_id(child.id);

        self_.add_new_namespace(tree, child.id);

        Ok(())
    }

    fn prepend(
        self_: pyo3::PyRef<'_, Self>,
        parent: pyo3::PyObject,
        child: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let parent =
            super::nodes::NodeGuard::from_pyobject(parent.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for parent, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), parent.as_ptr()) }
                ))
            })?;

        let child =
            super::nodes::NodeGuard::from_pyobject(child.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for child, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), child.as_ptr()) }
                ))
            })?;

        if !Arc::ptr_eq(&self_.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self_.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        let mut tree = self_.dom.lock();
        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.prepend_id(child.id);

        self_.add_new_namespace(tree, child.id);

        Ok(())
    }

    fn insert_before(
        self_: pyo3::PyRef<'_, Self>,
        parent: pyo3::PyObject,
        child: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let parent =
            super::nodes::NodeGuard::from_pyobject(parent.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for parent, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), parent.as_ptr()) }
                ))
            })?;

        let child =
            super::nodes::NodeGuard::from_pyobject(child.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for child, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), child.as_ptr()) }
                ))
            })?;

        if !Arc::ptr_eq(&self_.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self_.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        let mut tree = self_.dom.lock();
        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.insert_id_before(child.id);

        self_.add_new_namespace(tree, child.id);

        Ok(())
    }

    fn insert_after(
        self_: pyo3::PyRef<'_, Self>,
        parent: pyo3::PyObject,
        child: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let parent =
            super::nodes::NodeGuard::from_pyobject(parent.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for parent, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), parent.as_ptr()) }
                ))
            })?;

        let child =
            super::nodes::NodeGuard::from_pyobject(child.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for child, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), child.as_ptr()) }
                ))
            })?;

        if !Arc::ptr_eq(&self_.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self_.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        let mut tree = self_.dom.lock();
        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.insert_id_after(child.id);

        self_.add_new_namespace(tree, child.id);

        Ok(())
    }

    fn detach(self_: pyo3::PyRef<'_, Self>, node: pyo3::PyObject) -> pyo3::PyResult<()> {
        let node = super::nodes::NodeGuard::from_pyobject(node.bind(self_.py())).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "expected an node (such as Element, Text, Comment, ...) for node, got {}",
                unsafe { crate::tools::get_type_name(self_.py(), node.as_ptr()) }
            ))
        })?;

        if !Arc::ptr_eq(&self_.dom, &node.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given node node is not for this dom",
            ));
        }

        let mut tree = self_.dom.lock();
        let mut node = tree.get_mut(node.id).unwrap();

        node.detach();
        let id = node.id();
        let _ = node;

        self_.remove_old_namespace(tree, id);

        Ok(())
    }

    fn reparent_append(
        self_: pyo3::PyRef<'_, Self>,
        parent: pyo3::PyObject,
        child: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let parent =
            super::nodes::NodeGuard::from_pyobject(parent.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for parent, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), parent.as_ptr()) }
                ))
            })?;

        let child =
            super::nodes::NodeGuard::from_pyobject(child.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for child, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), child.as_ptr()) }
                ))
            })?;

        if !Arc::ptr_eq(&self_.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self_.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        let mut tree = self_.dom.lock();
        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.reparent_from_id_append(child.id);

        Ok(())
    }

    fn reparent_prepend(
        self_: pyo3::PyRef<'_, Self>,
        parent: pyo3::PyObject,
        child: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let parent =
            super::nodes::NodeGuard::from_pyobject(parent.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for parent, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), parent.as_ptr()) }
                ))
            })?;

        let child =
            super::nodes::NodeGuard::from_pyobject(child.bind(self_.py())).map_err(|_| {
                pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                    "expected an node (such as Element, Text, Comment, ...) for child, got {}",
                    unsafe { crate::tools::get_type_name(self_.py(), child.as_ptr()) }
                ))
            })?;

        if !Arc::ptr_eq(&self_.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self_.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        let mut tree = self_.dom.lock();
        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.reparent_from_id_prepend(child.id);

        Ok(())
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

                if Arc::ptr_eq(&self_.dom, &other.dom) {
                    Ok(true)
                } else {
                    let t1 = self_.dom.lock();
                    let t2 = other.dom.lock();

                    Ok(*t1 == *t2)
                }
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                if Arc::ptr_eq(&self_.dom, &other.dom) {
                    Ok(false)
                } else {
                    let t1 = self_.dom.lock();
                    let t2 = other.dom.lock();

                    Ok(*t1 != *t2)
                }
            }
            pyo3::basic::CompareOp::Gt => {
                crate::nodes::create_richcmp_notimplemented!('>', self_)
            }
            pyo3::basic::CompareOp::Lt => {
                crate::nodes::create_richcmp_notimplemented!('<', self_)
            }
            pyo3::basic::CompareOp::Le => {
                crate::nodes::create_richcmp_notimplemented!("<=", self_)
            }
            pyo3::basic::CompareOp::Ge => {
                crate::nodes::create_richcmp_notimplemented!(">=", self_)
            }
        }
    }

    fn __len__(&self) -> usize {
        let dom = self.dom.lock();
        dom.values().len()
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
