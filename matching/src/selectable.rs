use treedom::markup5ever::{namespace_url, ns};

/// A selectable [`treedom::ego_tree::NodeRef`]
#[derive(Debug, Clone)]
pub struct CssNodeRef<'a>(treedom::NodeRef<'a>);

impl<'a> CssNodeRef<'a> {
    pub fn new(node: treedom::NodeRef<'a>) -> Option<Self> {
        if node.value().is_element() {
            Some(Self(node))
        } else {
            None
        }
    }

    /// # Safety
    /// `node` value must be element
    pub unsafe fn new_unchecked(node: treedom::NodeRef<'a>) -> Self {
        Self(node)
    }

    pub fn into_node(self) -> treedom::NodeRef<'a> {
        self.0
    }
}

impl selectors::Element for CssNodeRef<'_> {
    type Impl = crate::_impl::ParserImplementation;

    fn opaque(&self) -> selectors::OpaqueElement {
        selectors::OpaqueElement::new(self)
    }

    fn parent_element(&self) -> Option<Self> {
        self.0
            .ancestors()
            .find(|x| x.value().is_element())
            .map(|x| unsafe { CssNodeRef::new_unchecked(x) })
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
        self.0.value().element().unwrap().name == other.0.value().element().unwrap().name
    }

    fn imported_part(
        &self,
        _name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::Identifier> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.0
            .prev_siblings()
            .find(|sibling| sibling.value().is_element())
            .map(|x| unsafe { Self::new_unchecked(x) })
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.0
            .next_siblings()
            .find(|sibling| sibling.value().is_element())
            .map(|x| unsafe { Self::new_unchecked(x) })
    }

    fn first_element_child(&self) -> Option<Self> {
        self.0
            .children()
            .find(|sibling| sibling.value().is_element())
            .map(|x| unsafe { Self::new_unchecked(x) })
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
            .class()
            .iter()
            .any(|c| case_sensitivity.eq(c.as_bytes(), name.content.as_bytes()))
    }

    fn has_custom_state(
        &self,
        _name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        !self.0.children().any(|x| {
            let v = x.value();
            v.is_element() || v.is_text()
        })
    }

    fn is_root(&self) -> bool {
        self.0.value().is_document()
    }

    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {}

    fn add_element_unique_hashes(&self, _filter: &mut selectors::bloom::BloomFilter) -> bool {
        false
    }
}
