use std::hash::Hasher;

use precomputed_hash::PrecomputedHash;

/// A StrTendril which implemented [`cssparser::ToCss`] and [`precomputed_hash::PrecomputedHash`]
#[derive(Clone, PartialEq, Eq)]
pub struct CssTendril {
    pub content: ::treedom::atomic::AtomicTendril,
    hash: u32,
}

impl cssparser::ToCss for CssTendril {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.content)
    }
}

impl PrecomputedHash for CssTendril {
    fn precomputed_hash(&self) -> u32 {
        self.hash
    }
}

impl<'a> From<&'a str> for CssTendril {
    fn from(value: &'a str) -> Self {
        use std::hash::Hash;

        let c = treedom::atomic::AtomicTendril::from(value);

        let mut state = std::hash::DefaultHasher::new();
        c.hash(&mut state);
        let state = state.finish();

        Self {
            content: c,
            hash: ((state >> 32) ^ state) as u32,
        }
    }
}

impl AsRef<str> for CssTendril {
    fn as_ref(&self) -> &str {
        &self.content
    }
}

unsafe impl Sync for CssTendril {}

/// A [`treedom::markup5ever::LocalName`] which implemented [`cssparser::ToCss`]
#[derive(Clone, PartialEq, Eq)]
pub struct CssLocalName(pub treedom::markup5ever::LocalName);

impl cssparser::ToCss for CssLocalName {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.0)
    }
}

impl PrecomputedHash for CssLocalName {
    fn precomputed_hash(&self) -> u32 {
        self.0.precomputed_hash()
    }
}

impl<'a> From<&'a str> for CssLocalName {
    fn from(value: &'a str) -> Self {
        Self(treedom::markup5ever::LocalName::from(value))
    }
}

impl std::borrow::Borrow<treedom::markup5ever::LocalName> for CssLocalName {
    fn borrow(&self) -> &treedom::markup5ever::LocalName {
        &self.0
    }
}

/// A [`treedom::markup5ever::Prefix`] which implemented [`cssparser::ToCss`]
#[derive(Default, Clone, PartialEq, Eq)]
pub struct CssNamespacePrefix(pub treedom::markup5ever::Prefix);

impl cssparser::ToCss for CssNamespacePrefix {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.0)
    }
}

impl<'a> From<&'a str> for CssNamespacePrefix {
    fn from(value: &'a str) -> Self {
        Self(treedom::markup5ever::Prefix::from(value))
    }
}

/// A NonTSPseudoClass which implemented [`cssparser::ToCss`]
#[derive(PartialEq, Eq, Clone)]
pub struct NonTSPseudoClass;

impl selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = ParserImplementation;

    fn is_active_or_hover(&self) -> bool {
        false
    }

    fn is_user_action_state(&self) -> bool {
        false
    }
}

impl cssparser::ToCss for NonTSPseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        dest.write_str("")
    }
}

/// A PseudoElement which implemented [`cssparser::ToCss`]
#[derive(Clone, PartialEq, Eq)]
pub struct PseudoElement;

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = ParserImplementation;
}

impl cssparser::ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        dest.write_str("")
    }
}

/// This struct defined the parser implementation in regards of pseudo-classes/elements
#[derive(Debug, Clone)]
pub struct ParserImplementation;

impl selectors::parser::SelectorImpl for ParserImplementation {
    type ExtraMatchingData<'a> = ();
    type AttrValue = CssTendril;
    type BorrowedLocalName = treedom::markup5ever::LocalName;
    type BorrowedNamespaceUrl = treedom::markup5ever::Namespace;
    type Identifier = CssTendril;
    type LocalName = CssLocalName;
    type NamespacePrefix = CssNamespacePrefix;
    type NamespaceUrl = treedom::markup5ever::Namespace;
    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}
