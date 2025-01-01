#[derive(Debug, Clone)]
pub struct CssParserKindError<'a>(pub selectors::parser::SelectorParseErrorKind<'a>);

impl<'a> From<selectors::parser::SelectorParseErrorKind<'a>> for CssParserKindError<'a> {
    fn from(value: selectors::parser::SelectorParseErrorKind<'a>) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for CssParserKindError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            self.0
        )
    }
}
