import markupever
from markupever import dom


def test_has_matches_ancestor_and_not_has_does_not():
    doc = markupever.parse("<div class='box'><span>a</span><p>b</p></div>")

    assert [e.name.local for e in doc.select("div:has(span)")] == ["div"]
    assert [e.name.local for e in doc.select("div:has(> span)")] == ["div"]
    assert list(doc.select("div:not(:has(span))")) == []


def test_scope_on_a_document_is_the_root_element():
    doc = markupever.parse("<!DOCTYPE html><html><body><p>x</p></body></html>")

    assert [e.name.local for e in doc.select(":scope")] == ["html"]


def test_select_on_an_element_does_not_return_that_element():
    outer = list(
        markupever.parse("<div class='box'><div class='box'></div></div>").select("div.box")
    )[0]

    assert [element.class_list for element in outer.select("div")] == [["box"]]
    assert [element.class_list for element in outer.select(":scope")] == [["box"]]
    assert [element.class_list for element in outer.select(":scope.box")] == [["box"]]


def test_scope_child_is_relative_to_the_element_select_was_called_on():
    doc = markupever.parse("<!DOCTYPE html><html><body><p>x</p></body></html>")
    body = list(doc.select("body"))[0]

    assert [e.name.local for e in body.select(":scope > p")] == ["p"]
    assert [e.name.local for e in body.select(":scope")] == ["body"]


def test_root_matches_the_document_element():
    doc = markupever.parse("<!DOCTYPE html><html><body><p>x</p></body></html>")

    assert [e.name.local for e in doc.select(":root")] == ["html"]


def test_nth_of_type_ignores_prefix():
    xml = markupever.parse(
        '<r xmlns:a="urn:ns" xmlns:b="urn:ns"><a:item/><b:item/><a:item/></r>',
        "xml",
    )

    assert [e.name.prefix for e in xml.select("item:nth-of-type(1)")] == ["a"]
    assert [e.name.prefix for e in xml.select("item:nth-of-type(2)")] == ["b"]
    assert [e.name.prefix for e in xml.select("item:nth-of-type(3)")] == ["a"]


def test_id_selector_prefers_the_unnamespaced_attribute():
    xml = markupever.parse('<div xml:id="a" id="b"/>', "xml")
    element = list(xml.select("div"))[0]

    assert element.id == "b"
    assert [e.name.local for e in xml.select("#b")] == ["div"]
    assert list(xml.select("#a")) == []
    assert [e.name.local for e in xml.select("[id=b]")] == ["div"]


def test_xml_id_is_the_fallback_when_there_is_no_id_attribute():
    xml = markupever.parse('<div xml:id="a"/>', "xml")
    element = list(xml.select("div"))[0]

    assert element.id == "a"
    assert [e.name.local for e in xml.select("#a")] == ["div"]


def test_class_list_keeps_source_order_and_duplicates():
    element = list(markupever.parse("<div class='b a b c'>").select("div"))[0]

    assert element.class_list == ["b", "a", "b", "c"]


def test_quirks_mode_class_and_id_selectors_are_case_insensitive():
    parser = markupever.Parser(markupever.HtmlOptions())
    parser.process("<div class='Foo' id='Bar'>x</div>")
    parser.finish()

    assert parser.quirks_mode == markupever.QUIRKS_MODE_FULL
    document = parser.into_dom()
    assert [element.class_list for element in document.select(".foo")] == [["Foo"]]
    assert [element.id for element in document.select("#bar")] == ["Bar"]


def test_fragment_reports_the_quirks_mode_it_was_given():
    options = markupever.HtmlOptions(
        quirks_mode=markupever.QUIRKS_MODE_FULL,
        full_document=False,
    )
    parser = markupever.Parser(options)
    parser.process("<p>x<table>")
    parser.finish()

    assert parser.quirks_mode == markupever.QUIRKS_MODE_FULL
    serialized = parser.into_dom().root().serialize()
    assert "<p>x<table></table></p>" in serialized


def test_standards_mode_class_selectors_stay_case_sensitive():
    parser = markupever.Parser(markupever.HtmlOptions())
    parser.process("<!DOCTYPE html><div class='Foo'>")
    parser.finish()

    assert parser.quirks_mode == markupever.QUIRKS_MODE_OFF
    document = parser.into_dom()
    assert list(document.select(".foo")) == []
    assert [element.class_list for element in document.select(".Foo")] == [["Foo"]]


def test_xml_xhtml_names_stay_case_sensitive():
    xhtml = markupever.parse(
        '<html xmlns="http://www.w3.org/1999/xhtml"><SPAN>A</SPAN><span>B</span></html>',
        "xml",
    )

    assert [element.name.local for element in xhtml.select("SPAN")] == ["SPAN"]
    assert [element.name.local for element in xhtml.select("span")] == ["span"]


def test_empty_ignores_zero_length_text():
    document = dom.TreeDom()
    root = document.root()
    div = root.create_element("div")
    div.create_text("")
    root.create_element("p")

    assert [e.name.local for e in document.select("div:empty")] == ["div"]
    assert [e.name.local for e in document.select("p:empty")] == ["p"]
    assert list(document.select("div:empty"))[0] is not None

    whitespace = markupever.parse("<div> </div><p></p>")
    names = [e.name.local for e in whitespace.select("div:empty")]
    assert "div" not in names
