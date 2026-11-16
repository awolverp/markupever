import markupever
from markupever import dom


def test_hand_built_html_document_serializes_as_html():
    document = dom.TreeDom()
    root = document.root()
    root.create_doctype("html", "pub", "sys")
    body = root.create_element("body")
    body.create_element("br")
    body.create_element("img", {"src": "a.png"})
    script = body.create_element("script")
    script.create_text("a < b")
    root.create_element("P")

    serialized = root.serialize()
    assert '<!DOCTYPE html PUBLIC "pub" "sys">' in serialized
    assert "<br>" in serialized
    assert "</br>" not in serialized
    assert '<img src="a.png">' in serialized
    assert "</img>" not in serialized
    assert "<script>a < b</script>" in serialized
    assert [element.name.local for element in document.select("p")] == ["p"]
    assert [element.name.local for element in document.select("P")] == ["p"]


def test_hand_built_tree_without_an_html_doctype_serializes_as_xml():
    document = dom.TreeDom()
    document.root().create_element("br")

    assert document.root().serialize() == "<br></br>"


def test_pretty_print_keeps_a_newline_before_the_outer_close_tag():
    outer = next(markupever.parse("<div id=o><div id=i></div></div>").select("#o"))

    assert outer.serialize(indent=2) == '<div id="o">\n  <div id="i"></div>\n</div>'


def test_doctype_with_public_and_system_ids_omits_the_system_keyword():
    serialized = (
        markupever.parse(
            '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd"><p>x'
        )
        .root()
        .serialize()
    )

    assert serialized.startswith(
        '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd">'
    )


def test_system_only_doctype_keeps_the_system_keyword():
    serialized = (
        markupever.parse('<!DOCTYPE html SYSTEM "sys.dtd"><p>').root().serialize()
    )

    assert serialized.startswith('<!DOCTYPE html SYSTEM "sys.dtd">')


def test_doctype_quotes_are_escaped():
    document = dom.TreeDom()
    root = document.root()
    doctype = root.create_doctype('html"><script>', 'a"b', "c")

    assert doctype.name == 'html"><script>'
    assert root.serialize().startswith(
        '<!DOCTYPE html&quot;&gt;&lt;script&gt; PUBLIC "a&quot;b" "c">'
    )
