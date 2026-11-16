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
