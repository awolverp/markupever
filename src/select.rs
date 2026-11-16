struct PySelectInner {
    traverse: crate::iter::PyTraverse,
    expr: ::matching::ExpressionGroup,
    /// Element `select` was called on. The document node has no scope, so
    /// `:scope` falls through to `:root`.
    scope_id: Option<::treedom::NodeId>,
    /// Start element, yielded only when the selector matches it because it is
    /// `:scope`. `div.select("div")` does not return that div.
    start: Option<crate::nodes::NodeGuard>,
}

impl PySelectInner {
    fn new(node: crate::nodes::NodeGuard, expr: String) -> pyo3::PyResult<Self> {
        let scope_id =
            matches!(node.type_, crate::nodes::NodeGuardType::Element).then_some(node.id);
        let start = scope_id.map(|_| node.clone());

        let tree = node.tree.lock();
        let expr = ::matching::ExpressionGroup::new(&expr, Some(tree.namespaces()))
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        std::mem::drop(tree);

        Ok(Self {
            traverse: crate::iter::PyTraverse::from_nodeguard(node),
            expr,
            scope_id,
            start,
        })
    }

    /// True when `guard` matches this selector only because it is `:scope`.
    fn matches_only_as_scope(&self, guard: &crate::nodes::NodeGuard) -> bool {
        let Some(scope_id) = self.scope_id else {
            return false;
        };

        let tree = guard.tree.lock();
        let html_document = tree.html_document();
        let quirks = tree.quirks_mode();
        let scope = ::matching::CssNodeRef::new(
            tree.get(scope_id).expect("scope node is still in the tree"),
            html_document,
        )
        .map(|node| node.opaque_id());

        let here = self.expr.matches(
            ::matching::CssNodeRef::new_unchecked(tree.get(guard.id).unwrap(), html_document),
            scope,
            quirks,
            &mut Default::default(),
        );
        if !here {
            return false;
        }

        let elsewhere = self.expr.matches(
            ::matching::CssNodeRef::new_unchecked(tree.get(guard.id).unwrap(), html_document),
            Some(::matching::foreign_scope()),
            quirks,
            &mut Default::default(),
        );
        !elsewhere
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
        if let Some(guard) = self.start.take()
            && self.matches_only_as_scope(&guard)
        {
            return Some(guard);
        }

        while let Some(guard) = self.next_descendant() {
            if self.scope_id == Some(guard.id)
                || !matches!(guard.type_, crate::nodes::NodeGuardType::Element)
            {
                continue;
            }

            let tree = guard.tree.lock();
            let html_document = tree.html_document();
            let quirks = tree.quirks_mode();
            let node = tree.get(guard.id).unwrap();
            let scope = self.scope_id.and_then(|id| {
                ::matching::CssNodeRef::new(
                    tree.get(id).expect("scope node is still in the tree"),
                    html_document,
                )
                .map(|scope| scope.opaque_id())
            });

            if self.expr.matches(
                ::matching::CssNodeRef::new_unchecked(node, html_document),
                scope,
                quirks,
                &mut Default::default(),
            ) {
                std::mem::drop(tree);
                return Some(guard);
            }
        }

        None
    }
}

#[pyo3::pyclass(name = "Select", module = "markupever._rustlib", unsendable)]
pub struct PySelect {
    inner: PySelectInner,
}

#[pyo3::pymethods]
impl PySelect {
    #[new]
    fn new(node: crate::nodes::PyNodeRef, expression: String) -> pyo3::PyResult<Self> {
        let node = node.as_node_guard().clone();

        Ok(Self {
            inner: PySelectInner::new(node, expression)?,
        })
    }

    fn __iter__(self_: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
        self_
    }

    pub fn __next__(&mut self) -> pyo3::PyResult<crate::nodes::NodeGuard> {
        self.inner
            .next()
            .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()))
    }
}
