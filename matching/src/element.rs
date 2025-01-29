use treedom::markup5ever::{namespace_url, ns};

#[derive(Debug, Clone)]
pub struct SelectableNode(treedom::Node);

impl From<treedom::Node> for SelectableNode {
    fn from(value: treedom::Node) -> Self {
        Self(value)
    }
}

impl SelectableNode {
    pub fn new(node: treedom::Node) -> Self {
        node.into()
    }

    pub fn into_node(self) -> treedom::Node {
        self.0
    }
}

impl selectors::Element for SelectableNode {
    type Impl = crate::_impl::ParserImplementation;

    fn opaque(&self) -> selectors::OpaqueElement {
        selectors::OpaqueElement::new(self)
    }

    fn parent_element(&self) -> Option<Self> {
        let mut parent = self.0.parent()?;
        while parent.value().element().is_none() {
            parent = parent.into_parent()?;
        }

        Some(parent.into())
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn is_part(&self, _name: &<Self::Impl as selectors::SelectorImpl>::Identifier) -> bool {
        false
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.0.eq_by(&other.0, |x, y| {
            x.element().unwrap().name == y.element().unwrap().name
        })
    }

    fn imported_part(
        &self,
        _name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::Identifier> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        let mut prev_sibling = self.0.prev_sibling()?;
        while prev_sibling.value().element().is_none() {
            prev_sibling = prev_sibling.into_prev_sibling()?;
        }

        Some(prev_sibling.into())
    }

    fn next_sibling_element(&self) -> Option<Self> {
        let mut next_sibling = self.0.next_sibling()?;
        while next_sibling.value().element().is_none() {
            next_sibling = next_sibling.into_next_sibling()?;
        }

        Some(next_sibling.into())
    }

    fn first_element_child(&self) -> Option<Self> {
        let mut front = self.0.first_children()?;
        if front.value().element().is_some() {
            return Some(front.into());
        }

        if Some(front.index()) == self.0.last_children_index() {
            return None;
        }

        while front.value().element().is_none() {
            front = front.into_next_sibling()?;
        }

        Some(front.into())
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.0.value().element().unwrap().name.ns == ns!(html)
    }

    fn has_local_name(
        &self,
        local_name: &<Self::Impl as selectors::SelectorImpl>::BorrowedLocalName,
    ) -> bool {
        self.0.value().element().unwrap().name.local == *local_name
    }

    fn has_namespace(
        &self,
        ns: &<Self::Impl as selectors::SelectorImpl>::BorrowedNamespaceUrl,
    ) -> bool {
        self.0.value().element().unwrap().name.ns == *ns
    }

    fn attr_matches(
        &self,
        ns: &selectors::attr::NamespaceConstraint<
            &<Self::Impl as selectors::SelectorImpl>::NamespaceUrl,
        >,
        local_name: &<Self::Impl as selectors::SelectorImpl>::LocalName,
        operation: &selectors::attr::AttrSelectorOperation<
            &<Self::Impl as selectors::SelectorImpl>::AttrValue,
        >,
    ) -> bool {
        let val = self.0.value();
        let elem = val.element().unwrap();

        elem.attrs.iter().any(|(key, val)| {
            !matches!(*ns, selectors::attr::NamespaceConstraint::Specific(url) if *url != key.ns)
                && local_name.0 == key.local
                && operation.eval_str(val)
        })
    }

    fn match_non_ts_pseudo_class(
        &self,
        _pc: &<Self::Impl as selectors::SelectorImpl>::NonTSPseudoClass,
        _context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &<Self::Impl as selectors::SelectorImpl>::PseudoElement,
        _context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn is_link(&self) -> bool {
        &self.0.value().element().unwrap().name.local == "link"
    }

    fn is_html_slot_element(&self) -> bool {
        true
    }

    fn has_id(
        &self,
        id: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        match self.0.value().element().unwrap().attrs.id() {
            Some(val) => case_sensitivity.eq(val.as_bytes(), id.content.as_bytes()),
            None => false,
        }
    }

    fn has_class(
        &self,
        name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        self.0
            .value()
            .element()
            .unwrap()
            .attrs
            .classes()
            .any(|c| case_sensitivity.eq(c.as_bytes(), name.content.as_bytes()))
    }

    fn has_custom_state(
        &self,
        _name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        let tree = unsafe { self.0.tree() };
        let lock = tree.lock();

        for item in lock.vec_iter() {
            if unsafe { item.as_ref().value().text().is_some() } {
                return false;
            }
        }

        true
    }

    fn is_root(&self) -> bool {
        self.0.value().document().is_some()
    }

    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {}

    fn add_element_unique_hashes(&self, _filter: &mut selectors::bloom::BloomFilter) -> bool {
        false
    }
}
