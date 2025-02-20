use super::CssNodeRef;
use super::_impl;

#[derive(Debug, Clone)]
pub struct CssParserKindError<'a>(pub selectors::parser::SelectorParseErrorKind<'a>);

impl<'a> From<selectors::parser::SelectorParseErrorKind<'a>> for CssParserKindError<'a> {
    fn from(value: selectors::parser::SelectorParseErrorKind<'a>) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for CssParserKindError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

struct Parser<'i>(Option<&'i treedom::NamespaceMap>);

impl<'i> selectors::parser::Parser<'i> for Parser<'i> {
    type Impl = _impl::ParserImplementation;
    type Error = CssParserKindError<'i>;

    fn parse_is_and_where(&self) -> bool {
        true
    }

    fn parse_has(&self) -> bool {
        true
    }

    fn parse_nth_child_of(&self) -> bool {
        true
    }

    fn namespace_for_prefix(
        &self,
        prefix: &<Self::Impl as selectors::SelectorImpl>::NamespacePrefix,
    ) -> Option<<Self::Impl as selectors::SelectorImpl>::NamespaceUrl> {
        if let Some(x) = self.0 {
            if x.is_empty() {
                return None;
            }

            x.get(&prefix.0).map(treedom::markup5ever::Namespace::from)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ExpressionGroup(selectors::SelectorList<_impl::ParserImplementation>);

impl ExpressionGroup {
    pub fn new<'a>(
        content: &'a str,
        namespaces: Option<&'a treedom::NamespaceMap>,
    ) -> Result<Self, cssparser::ParseError<'a, CssParserKindError<'a>>> {
        let mut parser_input = cssparser::ParserInput::new(content);
        let mut parser = cssparser::Parser::new(&mut parser_input);

        let sl = selectors::SelectorList::parse(
            &Parser(namespaces),
            &mut parser,
            selectors::parser::ParseRelative::No,
        )?;

        Ok(Self(sl))
    }

    pub fn matches<'a>(
        &self,
        node: CssNodeRef<'a>,
        scope: Option<CssNodeRef<'a>>,
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
            .any(|s| selectors::matching::matches_selector(s, 0, None, &node, &mut ctx))
    }
}

pub struct Select<'a> {
    inner: treedom::iter::Descendants<'a, treedom::interface::Interface>,
    expr: ExpressionGroup,
    caches: selectors::context::SelectorCaches,
}

impl<'a> Select<'a> {
    pub fn new<'b>(
        desc: treedom::iter::Descendants<'a, treedom::interface::Interface>,
        expr: &'b str,
        namespaces: Option<&'b treedom::NamespaceMap>,
    ) -> Result<Self, cssparser::ParseError<'b, CssParserKindError<'b>>> {
        let expr = ExpressionGroup::new(expr, namespaces)?;

        Ok(Self {
            inner: desc,
            expr,
            caches: Default::default(),
        })
    }
}

impl<'a> Iterator for Select<'a> {
    type Item = treedom::NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result: Option<Self::Item> = None;

        for node in &mut self.inner {
            if node.value().is_element()
                && self.expr.matches(
                    unsafe { CssNodeRef::new_unchecked(node) },
                    None,
                    &mut self.caches,
                )
            {
                result = Some(node);
                break;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tendril::TendrilSink;

    #[test]
    fn test_parsing() {
        let _ = ExpressionGroup::new("#id", None).unwrap();
        let _ = ExpressionGroup::new("div#id", None).unwrap();
        let _ = ExpressionGroup::new("div.cls", None).unwrap();
        let _ = ExpressionGroup::new(".cls", None).unwrap();
        let _ = ExpressionGroup::new(".title div.cls nav.pad", None).unwrap();
        let _ = ExpressionGroup::new("#table .row-1 div", None).unwrap();
        let _ = ExpressionGroup::new("a:has(href)", None).unwrap();
        let _ = ExpressionGroup::new(":root", None).unwrap();
        let _ = ExpressionGroup::new(".title, div.m", None).unwrap();
        let _ = ExpressionGroup::new("a:nth-child(1)", None).unwrap();
        let _ = ExpressionGroup::new("[href]", None).unwrap();
        let _ = ExpressionGroup::new("[type=\"text\"]", None).unwrap();
        let _ = ExpressionGroup::new("div > li", None).unwrap();
        let _ = ExpressionGroup::new("div + li", None).unwrap();
        let _ = ExpressionGroup::new("div ~ li", None).unwrap();
        let _ = ExpressionGroup::new("a.externals[href]", None).unwrap();
        let _ = ExpressionGroup::new("a[href^=\"https\"]", None).unwrap();
        let _ = ExpressionGroup::new("[href*=ht]", None).unwrap();
        let _ = ExpressionGroup::new("[href~=ht]", None).unwrap();
        let _ = ExpressionGroup::new("[href|=ht]", None).unwrap();
        let _ = ExpressionGroup::new("[src$=\".png\"]", None).unwrap();
        let _ = ExpressionGroup::new("a:empty", None).unwrap();
        let _ = ExpressionGroup::new("a:only-child", None).unwrap();
        let _ = ExpressionGroup::new("a:only-of-type", None).unwrap();
    }

    #[test]
    fn test_invalid_expr() {
        let _ = ExpressionGroup::new("<bad expr>", None).unwrap_err();
        let _ = ExpressionGroup::new("a:child-nth(1)", None).unwrap_err();
        let _ = ExpressionGroup::new("div..example", None).unwrap_err();
        let _ = ExpressionGroup::new("div - li", None).unwrap_err();
    }

    const HTML: &'static str = r#"<div class="title">
        <nav class="navbar">
            <p id="title">Hello World</p><p id="text">Hello World</p>
        </nav>
        <nav class="nav2"><p>World</p></nav>
        </div>"#;

    fn get_attr<'a>(
        attrs: &'a treedom::interface::ElementAttributes,
        key: &str,
    ) -> Option<&'a treedom::atomic::AtomicTendril> {
        attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    #[test]
    fn test_select_1() {
        let tree = treedom::ParserSink::parse_html(true, Default::default(), Default::default());
        let dom = tree.one(HTML).into_dom();

        for res in Select::new(dom.root().descendants(), "div.title", None).unwrap() {
            let elem = res.value().element().unwrap();
            assert_eq!(&*elem.name.local, "div");
            assert_eq!(
                get_attr(&elem.attrs, "class"),
                Some(&"title".to_owned().into())
            )
        }

        for res in Select::new(dom.root().descendants(), "nav.navbar p", None).unwrap() {
            let elem = res.value().element().unwrap();
            assert_eq!(&*elem.name.local, "p");
            get_attr(&elem.attrs, "id").unwrap();
        }

        for res in Select::new(dom.root().descendants(), "nav.nav2 p", None).unwrap() {
            let elem = res.value().element().unwrap();
            assert_eq!(&*elem.name.local, "p");
            assert!(get_attr(&elem.attrs, "id").is_none());
        }
    }
}
