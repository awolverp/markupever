mod node;
mod tree;

pub use node::{
    Children, CommentData, DoctypeData, DocumentData, ElementData, FragmentData, Node,
    NodesIterator, ProcessingInstructionData, TextData, WeakNode,
};
pub use tree::{ErrorWithLine, TreeBuilder};

use markup5ever::{local_name, namespace_url};
use tendril::TendrilSink;

fn get_html_parser(
    quirks_mode: markup5ever::interface::QuirksMode,
    exact_errors: bool,
    is_fragment: bool,
) -> html5ever::Parser<TreeBuilder> {
    let opts = html5ever::ParseOpts {
        tokenizer: html5ever::tokenizer::TokenizerOpts {
            exact_errors,
            ..Default::default()
        },
        tree_builder: html5ever::tree_builder::TreeBuilderOpts {
            exact_errors,
            quirks_mode,
            ..Default::default()
        },
    };

    if is_fragment {
        html5ever::driver::parse_fragment(
            TreeBuilder::new(Node::new(DocumentData)),
            opts,
            html5ever::QualName::new(
                None,
                namespace_url!("http://www.w3.org/1999/xhtml"),
                local_name!("body"),
            ),
            Vec::new(),
        )
    } else {
        html5ever::driver::parse_document(TreeBuilder::default(), opts)
    }
}

/// Parse HTML content using [`TreeBuilder`]
pub fn parse_html<Doc>(
    document: Doc,
    quirks_mode: markup5ever::interface::QuirksMode,
    exact_errors: bool,
    is_fragment: bool,
) -> TreeBuilder
where
    Doc: Into<tendril::StrTendril>,
{
    let parser = get_html_parser(quirks_mode, exact_errors, is_fragment);
    parser.one(document)
}

/// Parse UTF-8 HTML content using [`TreeBuilder`]
pub fn parse_html_utf8<Doc>(
    document: Doc,
    quirks_mode: markup5ever::interface::QuirksMode,
    exact_errors: bool,
    is_fragment: bool,
) -> TreeBuilder
where
    Doc: Into<tendril::ByteTendril>,
{
    let parser = get_html_parser(quirks_mode, exact_errors, is_fragment);
    parser.from_utf8().one(document)
}

/// Serialize a [`Node`](struct@super::node::Node) into HTML
pub fn serialize_html<Wr>(writer: Wr, node: &Node) -> std::io::Result<()>
where
    Wr: std::io::Write,
{
    html5ever::serialize::serialize(writer, node, html5ever::serialize::SerializeOpts::default())
}

fn get_xml_parser(exact_errors: bool) -> xml5ever::driver::XmlParser<TreeBuilder> {
    let opts = xml5ever::driver::XmlParseOpts {
        tokenizer: xml5ever::tokenizer::XmlTokenizerOpts {
            exact_errors,
            ..Default::default()
        },
        tree_builder: xml5ever::tree_builder::XmlTreeBuilderOpts {},
    };

    xml5ever::driver::parse_document(TreeBuilder::default(), opts)
}

/// Parse XML content using [`TreeBuilder`]
pub fn parse_xml<Doc>(document: Doc, exact_errors: bool) -> TreeBuilder
where
    Doc: Into<tendril::StrTendril>,
{
    let parser = get_xml_parser(exact_errors);
    parser.one(document)
}

/// Parse UTF-8 XML content using [`TreeBuilder`]
pub fn parse_xml_utf8<Doc>(document: Doc, exact_errors: bool) -> TreeBuilder
where
    Doc: Into<tendril::ByteTendril>,
{
    let parser = get_xml_parser(exact_errors);
    parser.from_utf8().one(document)
}

/// Serialize a [`Node`](struct@Node) into XML
pub fn serialize_xml<Wr>(writer: Wr, node: &Node) -> std::io::Result<()>
where
    Wr: std::io::Write,
{
    xml5ever::serialize::serialize(writer, node, xml5ever::serialize::SerializeOpts::default())
}

pub fn quirks_mode_from_u8(value: u8) -> markup5ever::interface::QuirksMode {
    match value {
        0 => markup5ever::interface::QuirksMode::Quirks,
        1 => markup5ever::interface::QuirksMode::LimitedQuirks,
        _ => markup5ever::interface::QuirksMode::NoQuirks,
    }
}

pub fn quirks_mode_to_u8(value: markup5ever::interface::QuirksMode) -> u8 {
    match value {
        markup5ever::interface::QuirksMode::Quirks => 0,
        markup5ever::interface::QuirksMode::LimitedQuirks => 1,
        markup5ever::interface::QuirksMode::NoQuirks => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let _ = parse_html(
            "<div><p>Hello</p> World</div>",
            markup5ever::interface::QuirksMode::NoQuirks,
            true,
            false,
        );

        let _ = parse_html(
            "<body><p>Hello</p> World</body>",
            markup5ever::interface::QuirksMode::LimitedQuirks,
            false,
            true,
        );

        let _ = parse_xml("<div><p>Hello</p> World</div>", true);

        let _ = parse_xml("<body><p>Hello</p> World</body>", false);
    }

    #[test]
    fn test_serialize() {
        let h = parse_html(
            "<body><p>Hello</p> World</body>",
            markup5ever::interface::QuirksMode::LimitedQuirks,
            false,
            true,
        );

        let mut w = Vec::new();
        serialize_html(&mut w, &h.root).unwrap();
    }
}
