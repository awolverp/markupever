use precomputed_hash::PrecomputedHash;
use treedom::markup5ever::ns;

use crate::_impl::CssTendril;

/// A selectable [`treedom::ego_tree::NodeRef`]
#[derive(Debug, Clone)]
pub struct CssNodeRef<'a>(treedom::NodeRef<'a>, bool);

/// Identity of a node for `:has()`, `:scope`, and the nth cache.
///
/// `ego_tree::NodeId` is a newtype over `NonZeroUsize` (the arena index plus
/// one). That bit pattern stays valid when the arena reallocates. The address
/// of a `CssNodeRef` does not: `parent_element` builds a new wrapper.
fn opaque_from_node_id(id: treedom::NodeId) -> selectors::OpaqueElement {
    const _: () = assert!(
        std::mem::size_of::<treedom::NodeId>() == std::mem::size_of::<std::num::NonZeroUsize>()
    );
    const _: () = assert!(
        std::mem::align_of::<treedom::NodeId>() == std::mem::align_of::<std::num::NonZeroUsize>()
    );

    let nz: std::num::NonZeroUsize = unsafe { std::mem::transmute(id) };
    selectors::OpaqueElement::from_non_null_ptr(
        std::ptr::NonNull::new(nz.get() as *mut ()).unwrap(),
    )
}

const NAMESPACE_SENTINEL: &str = "\0markupever-namespaces\0";

/// One selector namespace that means “any of these URIs”.
/// The selectors crate can store only one namespace per prefix.
pub(crate) fn namespace_sentinel(
    uris: &[treedom::markup5ever::Namespace],
) -> treedom::markup5ever::Namespace {
    let mut raw = String::from("\0markupever-namespaces");
    for uri in uris {
        raw.push('\0');
        raw.push_str(uri);
    }
    treedom::markup5ever::Namespace::from(raw)
}

pub(crate) fn namespace_sentinel_uris(ns: &treedom::markup5ever::Namespace) -> Option<Vec<&str>> {
    let text: &str = ns;
    text.strip_prefix(NAMESPACE_SENTINEL)
        .map(|rest| rest.split('\0').collect())
}

/// A scope id that is not an ego-tree node. Used to ask whether a selector
/// matches an element only because that element is `:scope`.
pub fn foreign_scope() -> selectors::OpaqueElement {
    selectors::OpaqueElement::from_non_null_ptr(
        std::ptr::NonNull::new(usize::MAX as *mut ()).unwrap(),
    )
}

impl<'a> CssNodeRef<'a> {
    pub fn new(node: treedom::NodeRef<'a>, html_document: bool) -> Option<Self> {
        if node.value().is_element() {
            Some(Self(node, html_document))
        } else {
            None
        }
    }

    /// # Safety
    /// `node` value must be element
    pub fn new_unchecked(node: treedom::NodeRef<'a>, html_document: bool) -> Self {
        Self(node, html_document)
    }

    fn wrap(&self, node: treedom::NodeRef<'a>) -> Option<Self> {
        Self::new(node, self.1)
    }

    pub fn into_node(self) -> treedom::NodeRef<'a> {
        self.0
    }

    pub fn opaque_id(&self) -> selectors::OpaqueElement {
        selectors::Element::opaque(self)
    }
}

impl selectors::Element for CssNodeRef<'_> {
    type Impl = crate::_impl::ParserImplementation;

    fn opaque(&self) -> selectors::OpaqueElement {
        opaque_from_node_id(self.0.id())
    }

    fn parent_element(&self) -> Option<Self> {
        self.0.ancestors().find_map(|node| self.wrap(node))
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
        let left = &self.0.value().element().unwrap().name;
        let right = &other.0.value().element().unwrap().name;
        left.ns == right.ns && left.local == right.local
    }

    fn imported_part(
        &self,
        _name: &<Self::Impl as selectors::SelectorImpl>::Identifier,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::Identifier> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.0.prev_siblings().find_map(|node| self.wrap(node))
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.0.next_siblings().find_map(|node| self.wrap(node))
    }

    fn first_element_child(&self) -> Option<Self> {
        self.0.children().find_map(|node| self.wrap(node))
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.1 && self.0.value().element().unwrap().name.ns == ns!(html)
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
        let element_ns = &self.0.value().element().unwrap().name.ns;
        if let Some(choices) = namespace_sentinel_uris(ns) {
            return choices.contains(&element_ns.as_str());
        }
        element_ns == ns
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
        &self.0.value().element().unwrap().name.local == "slot"
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
        !self.0.children().any(|child| {
            let value = child.value();
            if value.is_element() {
                return true;
            }
            // Selectors Level 3 ignores zero-length text. Whitespace still counts.
            value.text().is_some_and(|text| !text.contents.is_empty())
        })
    }

    fn is_root(&self) -> bool {
        self.0
            .parent()
            .is_some_and(|parent| parent.value().is_document())
    }

    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {}

    fn add_element_unique_hashes(&self, filter: &mut selectors::bloom::BloomFilter) -> bool {
        let elem = self.0.value().element().unwrap();

        // Same hashes the selector atoms store. A different function makes
        // the bloom filter reject elements that do match.
        filter.insert_hash(elem.name.ns.precomputed_hash());
        filter.insert_hash(elem.name.local.precomputed_hash());

        if let Some(id) = elem.attrs.id() {
            filter.insert_hash(CssTendril::from(id).precomputed_hash());
        }

        for class in elem.attrs.class().iter() {
            filter.insert_hash(CssTendril::from(class.as_ref()).precomputed_hash());
        }

        true
    }
}
