from xmarkup import _rustlib
import xmarkup
import pytest


def test_treedom():
    dom = xmarkup.dom.TreeDom()

    assert dom.namespaces() == {}
    assert len(dom) == 1

    _ = str(dom)
    _ = repr(dom)

    lst = list(dom)

    assert len(lst) == 1
    assert isinstance(lst[0], xmarkup.dom.Document)


def _test_rustlib_node_convert(typ, expected, dom, *args, **kwargs) -> xmarkup.dom.BaseNode:
    instance = xmarkup.dom.BaseNode._wrap(typ(dom._raw, *args, **kwargs))
    assert isinstance(instance, expected)
    return instance


def test_basenode_init():
    dom = xmarkup.dom.TreeDom()

    assert isinstance(dom.root(), xmarkup.dom.Document)

    _test_rustlib_node_convert(_rustlib.Doctype, xmarkup.dom.Doctype, dom, "name", "", "")
    _test_rustlib_node_convert(_rustlib.Comment, xmarkup.dom.Comment, dom, "content")
    _test_rustlib_node_convert(_rustlib.Text, xmarkup.dom.Text, dom, "content")
    _test_rustlib_node_convert(_rustlib.Element, xmarkup.dom.Element, dom, "name", [], False, False)
    _test_rustlib_node_convert(
        _rustlib.ProcessingInstruction, xmarkup.dom.ProcessingInstruction, dom, "name", "data"
    )

    with pytest.raises(TypeError):
        xmarkup.dom.BaseNode("invalid type")

    xmarkup.dom.Doctype(_rustlib.Doctype(dom._raw, "m", "", ""))

    with pytest.raises(TypeError):
        xmarkup.dom.Element(_rustlib.Doctype(dom._raw, "m", "", ""))

    with pytest.raises(TypeError):
        xmarkup.dom.BaseNode._wrap(1)


def test_connect_node():
    dom = xmarkup.dom.TreeDom()
    root = dom.root()

    html = root.create_element(xmarkup.dom.QualName("html", "html"), {"lang": "en"}, False, False)
    assert isinstance(html, xmarkup.dom.Element)
    assert html.parent == root
    assert html.name == "html"

    head = html.create_element("head")
    assert isinstance(head, xmarkup.dom.Element)
    assert head.parent == html
    assert head.name == "head"

    meta_viewport = head.create_element(
        xmarkup.dom.QualName("meta", "html"),
        [("name", "viewport"), ("content", "width=device-width, initial-scale=1.0")],
    )
    assert isinstance(meta_viewport, xmarkup.dom.Element)
    assert meta_viewport.parent == head
    assert meta_viewport.name == xmarkup.dom.QualName("meta", "html")

    meta_charset = meta_viewport.create_element(
        xmarkup.dom.QualName("meta", "html"),
        {"charset": "UTF-8"},
        ordering=xmarkup.dom.Ordering.BEFORE,
    )
    assert isinstance(meta_charset, xmarkup.dom.Element)
    assert meta_charset.parent == head
    assert meta_charset.name == "meta"
    assert meta_charset.next_sibling == meta_viewport

    body = head.create_element("body", {"class": "bg-dark"}, ordering=xmarkup.dom.Ordering.AFTER)
    assert isinstance(body, xmarkup.dom.Element)
    assert body.parent == html
    assert body.name == "body"

    with pytest.raises(ValueError):
        root.create_doctype("html", ordering=xmarkup.dom.Ordering.AFTER)

    with pytest.raises(ValueError):
        root.create_doctype("html", ordering=10)

    doctype = root.create_doctype("html", ordering=xmarkup.dom.Ordering.PREPEND)
    assert isinstance(doctype, xmarkup.dom.Doctype)
    assert doctype.parent == root
    assert doctype.name == "html"
    assert doctype.next_sibling == html
    assert html.prev_sibling == doctype

    assert dom.namespaces() == {"": "http://www.w3.org/1999/xhtml"}

    p = body.create_element(
        xmarkup.dom.QualName("p", "namespace1", "ns1"),
        {"class": "font-sans"},
        ordering=xmarkup.dom.Ordering.APPEND,
    )
    assert isinstance(p, xmarkup.dom.Element)
    assert p.parent == body
    assert p.name == "p"

    assert dom.namespaces() == {"": "http://www.w3.org/1999/xhtml", "ns1": "namespace1"}

    comment = p.create_comment("content")
    assert isinstance(comment, xmarkup.dom.Comment)
    assert comment.parent == p
    assert comment.content == "content"
    assert comment == "content"

    assert p.text() == ""

    text = p.create_text("\ncontent 1")
    assert isinstance(text, xmarkup.dom.Text)
    assert text.parent == p
    assert text.content == "\ncontent 1"
    assert text == "\ncontent 1"

    p.create_text("\ncontent 2")

    assert p.text() == "\ncontent 1\ncontent 2"
    assert p.text(strip=True) == "content 1content 2"
    assert p.text(seperator="\t", strip=True) == "content 1\tcontent 2"

    assert text.has_siblings
    assert p.has_children
    assert p.tree() == dom

    assert (
        root.serialize()
        == '<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head><body class="bg-dark"><p class="font-sans"><!--content-->\ncontent 1\ncontent 2</p></body></html>'
    )

    with pytest.raises(ValueError):
        root.create_comment("content", ordering=xmarkup.dom.Ordering.AFTER)

    with pytest.raises(ValueError):
        root.create_comment("content", ordering=xmarkup.dom.Ordering.BEFORE)

    comment = root.create_comment("content")
    assert isinstance(comment, xmarkup.dom.Comment)
    assert comment.content == "content"
    assert comment == "content"

    text = root.create_text("content")
    assert isinstance(text, xmarkup.dom.Text)
    assert text.content == "content"
    assert text == "content"

    pi = root.create_processing_instruction("data", "target")
    assert isinstance(pi, xmarkup.dom.ProcessingInstruction)
    assert pi.data == "data"
    assert pi.target == "target"

    assert root.first_child == doctype
    assert root.last_child == pi
