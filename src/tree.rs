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

/// Checks that `child` can be inserted as a child of `parent`: a node can't be inserted into
/// itself or one of its descendants, which would make it its own ancestor.
fn check_insert_into(
    tree: &::treedom::IDTreeDOM,
    parent: ::treedom::NodeId,
    child: ::treedom::NodeId,
) -> pyo3::PyResult<()> {
    let parent = tree.get(parent).unwrap();

    if parent.id() == child || parent.ancestors().any(|ancestor| ancestor.id() == child) {
        return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Cannot insert a node into itself or its descendant",
        ));
    }

    Ok(())
}

/// Checks that `new_sibling` can be inserted as a sibling of `sibling`.
fn check_insert_beside(
    tree: &::treedom::IDTreeDOM,
    sibling: ::treedom::NodeId,
    new_sibling: ::treedom::NodeId,
) -> pyo3::PyResult<()> {
    let parent = tree.get(sibling).unwrap().parent().ok_or_else(|| {
        pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Cannot insert a sibling of a node that has no parent",
        )
    })?;

    check_insert_into(tree, parent.id(), new_sibling)
}

#[pyo3::pymethods]
impl PyTreeDom {
    /// Creates a new [`PyTreeDom`]
    #[new]
    #[classmethod]
    #[pyo3(signature=(*, namespaces=None))]
    fn new(
        cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        namespaces: Option<
            std::collections::HashMap<pyo3::pybacked::PyBackedStr, pyo3::pybacked::PyBackedStr>,
        >,
    ) -> Self {
        Self::with_capacity(cls, 0, namespaces)
    }

    /// Creates a new [`PyTreeDom`] with the specified capacity.
    #[classmethod]
    #[pyo3(signature=(capacity, *, namespaces=None))]
    fn with_capacity(
        _cls: &pyo3::Bound<'_, pyo3::types::PyType>,
        capacity: usize,
        namespaces: Option<
            std::collections::HashMap<pyo3::pybacked::PyBackedStr, pyo3::pybacked::PyBackedStr>,
        >,
    ) -> Self {
        let ns = namespaces
            .into_iter()
            .flatten()
            .map(|(key, val)| ((*key).into(), (*val).into()))
            .collect();

        let dom = if capacity == 0 {
            ::treedom::IDTreeDOM::new(::treedom::interface::DocumentInterface, ns)
        } else {
            ::treedom::IDTreeDOM::with_capacity(
                ::treedom::interface::DocumentInterface,
                ns,
                capacity,
            )
        };

        Self {
            dom: Arc::new(parking_lot::Mutex::new(dom)),
        }
    }

    /// Returns the available namespaces in DOM as a `dict`.
    fn namespaces(&self) -> std::collections::HashMap<String, String> {
        let dom = self.dom.lock();
        dom.namespaces()
            .iter()
            .map(|(key, val)| (key.to_string(), val.to_string()))
            .collect()
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
        &self,
        parent: crate::nodes::PyNodeRef,
        child: crate::nodes::PyNodeRef,
    ) -> pyo3::PyResult<()> {
        let parent = parent.as_node_guard();
        let child = child.as_node_guard();

        if !Arc::ptr_eq(&self.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        if parent.id == child.id {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Cannot append node as a child to itself",
            ));
        }

        let mut tree = self.dom.lock();
        check_insert_into(&tree, parent.id, child.id)?;

        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.append_id(child.id);

        self.add_new_namespace(tree, child.id);

        Ok(())
    }

    fn prepend(
        &self,
        parent: crate::nodes::PyNodeRef,
        child: crate::nodes::PyNodeRef,
    ) -> pyo3::PyResult<()> {
        let parent = parent.as_node_guard();
        let child = child.as_node_guard();

        if !Arc::ptr_eq(&self.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        if parent.id == child.id {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Cannot append node as a child to itself",
            ));
        }

        let mut tree = self.dom.lock();
        check_insert_into(&tree, parent.id, child.id)?;

        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.prepend_id(child.id);

        self.add_new_namespace(tree, child.id);

        Ok(())
    }

    fn insert_before(
        &self,
        sibling: crate::nodes::PyNodeRef,
        new_sibling: crate::nodes::PyNodeRef,
    ) -> pyo3::PyResult<()> {
        let sibling = sibling.as_node_guard();
        let new_sibling = new_sibling.as_node_guard();

        if !Arc::ptr_eq(&self.dom, &sibling.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given sibling is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self.dom, &new_sibling.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given new sibling is not for this dom",
            ));
        }

        if sibling.id == new_sibling.id {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Cannot insert node as a sibling of itself",
            ));
        }

        let mut tree = self.dom.lock();
        check_insert_beside(&tree, sibling.id, new_sibling.id)?;

        let mut sibling = tree.get_mut(sibling.id).unwrap();

        sibling.insert_id_before(new_sibling.id);

        self.add_new_namespace(tree, new_sibling.id);

        Ok(())
    }

    fn insert_after(
        &self,
        sibling: crate::nodes::PyNodeRef,
        new_sibling: crate::nodes::PyNodeRef,
    ) -> pyo3::PyResult<()> {
        let sibling = sibling.as_node_guard();
        let new_sibling = new_sibling.as_node_guard();

        if !Arc::ptr_eq(&self.dom, &sibling.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given sibling is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self.dom, &new_sibling.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given new sibling is not for this dom",
            ));
        }

        if sibling.id == new_sibling.id {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Cannot insert node as a sibling of itself",
            ));
        }

        let mut tree = self.dom.lock();
        check_insert_beside(&tree, sibling.id, new_sibling.id)?;

        let mut sibling = tree.get_mut(sibling.id).unwrap();

        sibling.insert_id_after(new_sibling.id);

        self.add_new_namespace(tree, new_sibling.id);

        Ok(())
    }

    fn detach(&self, node: crate::nodes::PyNodeRef) -> pyo3::PyResult<()> {
        let node = node.as_node_guard();

        if !Arc::ptr_eq(&self.dom, &node.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given node node is not for this dom",
            ));
        }

        let mut tree = self.dom.lock();
        let mut node = tree.get_mut(node.id).unwrap();

        node.detach();
        let id = node.id();
        let _ = node;

        self.remove_old_namespace(tree, id);

        Ok(())
    }

    fn reparent_append(
        &self,
        parent: crate::nodes::PyNodeRef,
        child: crate::nodes::PyNodeRef,
    ) -> pyo3::PyResult<()> {
        let parent = parent.as_node_guard();
        let child = child.as_node_guard();

        if !Arc::ptr_eq(&self.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        let mut tree = self.dom.lock();
        // Moving a node's children into the node itself or one of its descendants would make a
        // node its own ancestor.
        check_insert_into(&tree, parent.id, child.id)?;

        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.reparent_from_id_append(child.id);

        Ok(())
    }

    fn reparent_prepend(
        &self,
        parent: crate::nodes::PyNodeRef,
        child: crate::nodes::PyNodeRef,
    ) -> pyo3::PyResult<()> {
        let parent = parent.as_node_guard();
        let child = child.as_node_guard();

        if !Arc::ptr_eq(&self.dom, &parent.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent parent is not for this dom",
            ));
        }

        if !Arc::ptr_eq(&self.dom, &child.tree) {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "the given parent child is not for this dom",
            ));
        }

        let mut tree = self.dom.lock();
        // Moving a node's children into the node itself or one of its descendants would make a
        // node its own ancestor.
        check_insert_into(&tree, parent.id, child.id)?;

        let mut parent = tree.get_mut(parent.id).unwrap();

        parent.reparent_from_id_prepend(child.id);

        Ok(())
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

                if Arc::ptr_eq(&self_.get().dom, &other.dom) {
                    Ok(true)
                } else {
                    let t1 = self_.get().dom.lock();
                    let t2 = other.dom.lock();

                    Ok(*t1 == *t2)
                }
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(o) => o,
                    Err(_) => return Ok(false),
                };

                if Arc::ptr_eq(&self_.get().dom, &other.dom) {
                    Ok(false)
                } else {
                    let t1 = self_.get().dom.lock();
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
