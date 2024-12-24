/// A [`String`] with ToCss impl
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ToCssString(pub String);

impl cssparser::ToCss for ToCssString {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl AsRef<str> for ToCssString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> From<&'a str> for ToCssString {
    fn from(value: &'a str) -> Self {
        Self(value.into())
    }
}

/// A [`markup5ever::LocalName`] with ToCss impl
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct ToCssLocalName(pub markup5ever::LocalName);

impl cssparser::ToCss for ToCssLocalName {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        dest.write_str(&self.0)
    }
}

impl precomputed_hash::PrecomputedHash for ToCssLocalName {
    fn precomputed_hash(&self) -> u32 {
        self.0.precomputed_hash()
    }
}

impl<'a> From<&'a str> for ToCssLocalName {
    fn from(value: &'a str) -> Self {
        Self(value.into())
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct NonTSPseudoClass;

impl selectors::parser::NonTSPseudoClass for NonTSPseudoClass {
    type Impl = SelectorImpl;

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

#[derive(Clone, PartialEq, Eq)]
pub struct PseudoElement;

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = SelectorImpl;
}

impl cssparser::ToCss for PseudoElement {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        dest.write_str("")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorImpl;

impl selectors::SelectorImpl for SelectorImpl {
    type ExtraMatchingData<'a> = ();

    type AttrValue = ToCssString;
    type LocalName = ToCssLocalName;
    type Identifier = ToCssLocalName;
    type NamespacePrefix = ToCssLocalName;
    type NamespaceUrl = markup5ever::Namespace;
    type BorrowedLocalName = ToCssLocalName;
    type BorrowedNamespaceUrl = markup5ever::Namespace;
    type NonTSPseudoClass = NonTSPseudoClass;
    type PseudoElement = PseudoElement;
}
