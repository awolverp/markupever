#[derive(Debug, Clone)]
pub struct CssParserKindError<'a>(pub selectors::parser::SelectorParseErrorKind<'a>);

impl<'a> From<selectors::parser::SelectorParseErrorKind<'a>> for CssParserKindError<'a> {
    fn from(value: selectors::parser::SelectorParseErrorKind<'a>) -> Self {
        Self(value)
    }
}
