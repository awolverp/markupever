use super::NonTSPseudoClass;
use super::PseudoElement;
use super::ToCssLocalName;
use super::ToCssString;
use super::_impl::SelectorImpl;
use crate::core::arcdom::Node;
use crate::core::arcdom::NodesIterator;
use markup5ever::{namespace_url, ns};

impl selectors::Element for Node {
    type Impl = SelectorImpl;

    fn opaque(&self) -> selectors::OpaqueElement {
        selectors::OpaqueElement::new(self)
    }

    fn parent_element(&self) -> Option<Self> {
        self.parents().find(|x| x.is_element())
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

    fn is_part(&self, _name: &ToCssLocalName) -> bool {
        false
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.as_element().unwrap().name == other.as_element().unwrap().name
    }

    fn imported_part(&self, _name: &ToCssLocalName) -> Option<ToCssLocalName> {
        None
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        let parent = self.parent();

        if parent.is_none() {
            return None;
        }

        let parent = parking_lot::MappedMutexGuard::map(parent, |x| unsafe {
            x.as_mut().unwrap_unchecked()
        });

        // drop guard, clone and upgrade parent
        let parent = parent.clone().upgrade().expect("dangling weak pointer");

        let p_children = parent.children();
        let index = p_children
            .position(self)
            .expect("have parent but couldn't find in parent's children!");

        if index == 0 {
            return None;
        }

        p_children.vec[..index]
            .iter()
            .find(|x| x.is_element())
            .cloned()
    }

    fn next_sibling_element(&self) -> Option<Self> {
        let parent = self.parent();

        if parent.is_none() {
            return None;
        }

        let parent = parking_lot::MappedMutexGuard::map(parent, |x| unsafe {
            x.as_mut().unwrap_unchecked()
        });

        // drop guard, clone and upgrade parent
        let parent = parent.clone().upgrade().expect("dangling weak pointer");

        let p_children = parent.children();
        let index = p_children
            .position(self)
            .expect("have parent but couldn't find in parent's children!");

        if index == p_children.len() - 1 {
            return None;
        }

        p_children.vec[index + 1..]
            .iter()
            .find(|x| x.is_element())
            .cloned()
    }

    fn first_element_child(&self) -> Option<Self> {
        self.children().iter().find(|x| x.is_element()).cloned()
    }

    fn is_html_element_in_html_document(&self) -> bool {
        self.as_element().unwrap().name.ns == ns!(html)
    }

    fn has_local_name(&self, local_name: &ToCssLocalName) -> bool {
        self.as_element().unwrap().name.local == local_name.0
    }

    fn has_namespace(&self, ns: &markup5ever::Namespace) -> bool {
        &self.as_element().unwrap().name.ns == ns
    }

    fn attr_matches(
        &self,
        ns: &selectors::attr::NamespaceConstraint<&markup5ever::Namespace>,
        local_name: &ToCssLocalName,
        operation: &selectors::attr::AttrSelectorOperation<&ToCssString>,
    ) -> bool {
        let elem = self.as_element().unwrap();

        elem.attrs.iter().any(|(key, val)| {
            !matches!(*ns, selectors::attr::NamespaceConstraint::Specific(url) if *url != key.ns)
                && local_name.0 == key.local
                && operation.eval_str(val)
        })
    }

    fn match_non_ts_pseudo_class(
        &self,
        _pc: &NonTSPseudoClass,
        _context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn match_pseudo_element(
        &self,
        _pe: &PseudoElement,
        _context: &mut selectors::context::MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn is_link(&self) -> bool {
        &self.as_element().unwrap().name.local == "link"
    }

    fn is_html_slot_element(&self) -> bool {
        true
    }

    fn has_id(
        &self,
        id: &ToCssLocalName,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        match self.as_element().unwrap().id() {
            Some(val) => case_sensitivity.eq(val.as_bytes(), id.0.as_bytes()),
            None => false,
        }
    }

    fn has_class(
        &self,
        name: &ToCssLocalName,
        case_sensitivity: selectors::attr::CaseSensitivity,
    ) -> bool {
        self.as_element()
            .unwrap()
            .classes()
            .any(|c| case_sensitivity.eq(c.as_bytes(), name.0.as_bytes()))
    }

    fn has_custom_state(&self, _name: &ToCssLocalName) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        !self
            .children()
            .iter()
            .any(|c| c.is_element() || c.is_text())
    }

    fn is_root(&self) -> bool {
        self.is_document()
    }

    fn apply_selector_flags(&self, _flags: selectors::matching::ElementSelectorFlags) {}

    fn add_element_unique_hashes(&self, _filter: &mut selectors::bloom::BloomFilter) -> bool {
        false
    }
}

pub struct Parser;

impl<'i> selectors::parser::Parser<'i> for Parser {
    type Impl = SelectorImpl;
    type Error = super::errors::CssParserKindError<'i>;

    fn parse_is_and_where(&self) -> bool {
        true
    }

    fn parse_has(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectExprGroup(selectors::SelectorList<SelectorImpl>);

impl SelectExprGroup {
    pub fn new(
        content: &'_ str,
    ) -> Result<Self, cssparser::ParseError<'_, super::errors::CssParserKindError>> {
        let mut parser_input = cssparser::ParserInput::new(content);
        let mut parser = cssparser::Parser::new(&mut parser_input);

        let sl = selectors::SelectorList::parse(
            &Parser,
            &mut parser,
            selectors::parser::ParseRelative::No,
        )?;

        Ok(Self(sl))
    }

    pub fn matches(
        &self,
        node: &Node,
        scope: Option<Node>,
        caches: &mut selectors::context::SelectorCaches,
    ) -> bool {
        let mut ctx = selectors::matching::MatchingContext::new(
            selectors::matching::MatchingMode::Normal,
            None,
            caches,
            selectors::matching::QuirksMode::NoQuirks,
            selectors::matching::NeedsSelectorFlags::No,
            selectors::matching::MatchingForInvalidation::No,
        );
        ctx.scope_element = scope.map(|x| selectors::Element::opaque(&x));
        self.0
            .slice()
            .iter()
            .any(|s| selectors::matching::matches_selector(s, 0, None, node, &mut ctx))
    }
}

pub struct Select {
    inner: NodesIterator,
    expr: SelectExprGroup,
    caches: selectors::context::SelectorCaches,
}

impl Select {
    pub fn new(
        iterator: NodesIterator,
        expr: &str,
    ) -> Result<Select, cssparser::ParseError<'_, super::errors::CssParserKindError<'_>>> {
        let expr = SelectExprGroup::new(expr)?;

        Ok(Select {
            inner: iterator,
            expr,
            caches: Default::default(),
        })
    }
}

impl Iterator for Select {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result: Option<Node> = None;

        for node in &mut self.inner {
            if node.is_element() && self.expr.matches(&node, None, &mut self.caches) {
                result = Some(node.clone());
                break;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use crate::core::arcdom::parse_html;

    use super::*;

    #[test]
    fn test_parsing() {
        let _ = SelectExprGroup::new("#id").unwrap();
        let _ = SelectExprGroup::new("div#id").unwrap();
        let _ = SelectExprGroup::new("div.cls").unwrap();
        let _ = SelectExprGroup::new(".cls").unwrap();
        let _ = SelectExprGroup::new(".title div.cls nav.pad").unwrap();
        let _ = SelectExprGroup::new("#table .row-1 div").unwrap();
        let _ = SelectExprGroup::new("a:has(href)").unwrap();
        let _ = SelectExprGroup::new(":root").unwrap();
        let _ = SelectExprGroup::new(".title, div.m").unwrap();
    }

    #[test]
    fn test_invalid_expr() {
        let _ = SelectExprGroup::new("<bad expr>").unwrap_err();
        let _ = SelectExprGroup::new("a:child-nth(1)").unwrap_err();
    }

    #[test]
    fn test_select() {
        let tree = parse_html(
            r#"<div class="title">
                        <nav class="navbar">
                            <p id="title">Hello World</p><p id="text">Hello World</p>
                        </nav>
                        <nav class="nav2"><p>World</p></nav>
                        </div>"#,
            markup5ever::interface::QuirksMode::NoQuirks,
            true,
            false,
        );

        for res in Select::new(tree.root.iter(), "div.title").unwrap() {
            let elem = res.as_element().unwrap();
            assert_eq!(&*elem.name.local, "div");
            assert_eq!(
                elem.classes().collect::<Vec<_>>(),
                &[&markup5ever::LocalName::from("title")]
            );
        }

        for res in Select::new(tree.root.iter(), "nav.navbar p").unwrap() {
            let elem = res.as_element().unwrap();
            assert_eq!(&*elem.name.local, "p");
            assert!(elem.id().is_some());
        }

        for res in Select::new(tree.root.iter(), "nav.nav2 p").unwrap() {
            let elem = res.as_element().unwrap();
            assert_eq!(&*elem.name.local, "p");
            assert!(elem.id().is_none());
        }
    }
}
