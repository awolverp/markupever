from markupever import _rustlib
import markupever
import pytest


def test_treedom():
    dom = markupever.dom.TreeDom()

    assert dom.namespaces() == {}
    assert len(dom) == 1

    _ = str(dom)
    _ = repr(dom)

    lst = list(dom)

    assert dom == dom
    assert dom != 1
    assert len(lst) == 1
    assert isinstance(lst[0], markupever.dom.Document)


def _test_rustlib_node_convert(typ, expected, dom, *args, **kwargs) -> markupever.dom.BaseNode:
    instance = markupever.dom.BaseNode._wrap(typ(dom._raw, *args, **kwargs))
    assert isinstance(instance, expected)
    return instance


def test_basenode_init():
    dom = markupever.dom.TreeDom()

    assert isinstance(dom.root(), markupever.dom.Document)

    _test_rustlib_node_convert(_rustlib.Doctype, markupever.dom.Doctype, dom, "name", "", "")
    _test_rustlib_node_convert(_rustlib.Comment, markupever.dom.Comment, dom, "content")
    _test_rustlib_node_convert(_rustlib.Text, markupever.dom.Text, dom, "content")
    _test_rustlib_node_convert(
        _rustlib.Element, markupever.dom.Element, dom, "name", [], False, False
    )
    _test_rustlib_node_convert(
        _rustlib.ProcessingInstruction, markupever.dom.ProcessingInstruction, dom, "name", "data"
    )

    with pytest.raises(TypeError):
        markupever.dom.BaseNode("invalid type")

    markupever.dom.Doctype(_rustlib.Doctype(dom._raw, "m", "", ""))

    with pytest.raises(TypeError):
        markupever.dom.Element(_rustlib.Doctype(dom._raw, "m", "", ""))

    with pytest.raises(TypeError):
        markupever.dom.BaseNode._wrap(1)


def test_connect_node():
    dom = markupever.dom.TreeDom()
    root = dom.root()

    html = root.create_element(
        markupever.dom.QualName("html", "html"), {"lang": "en"}, False, False
    )
    assert isinstance(html, markupever.dom.Element)
    assert html.parent == root
    assert html.name == "html"

    head = html.create_element("head")
    assert isinstance(head, markupever.dom.Element)
    assert head.parent == html
    assert head.name == "head"

    meta_viewport = head.create_element(
        markupever.dom.QualName("meta", "html"),
        [("name", "viewport"), ("content", "width=device-width, initial-scale=1.0")],
    )
    assert isinstance(meta_viewport, markupever.dom.Element)
    assert meta_viewport.parent == head
    assert meta_viewport.name == markupever.dom.QualName("meta", "html")

    meta_charset = meta_viewport.create_element(
        markupever.dom.QualName("meta", "html"),
        {"charset": "UTF-8"},
        ordering=markupever.dom.Ordering.BEFORE,
    )
    assert isinstance(meta_charset, markupever.dom.Element)
    assert meta_charset.parent == head
    assert meta_charset.name == "meta"
    assert meta_charset.next_sibling == meta_viewport

    body = head.create_element("body", {"class": "bg-dark"}, ordering=markupever.dom.Ordering.AFTER)
    assert isinstance(body, markupever.dom.Element)
    assert body.parent == html
    assert body.name == "body"

    with pytest.raises(ValueError):
        root.create_doctype("html", ordering=markupever.dom.Ordering.AFTER)

    with pytest.raises(ValueError):
        root.create_doctype("html", ordering=10)

    doctype = root.create_doctype("html", ordering=markupever.dom.Ordering.PREPEND)
    assert isinstance(doctype, markupever.dom.Doctype)
    assert doctype.parent == root
    assert doctype.name == "html"
    assert doctype.next_sibling == html
    assert html.prev_sibling == doctype

    doctype.name = "HTML"
    doctype.public_id = "public"
    doctype.system_id = "system"
    assert doctype.name == "HTML"
    assert doctype.public_id == "public"
    assert doctype.system_id == "system"

    assert dom.namespaces() == {"": "http://www.w3.org/1999/xhtml"}

    p = body.create_element(
        markupever.dom.QualName("p", "namespace1", "ns1"),
        {"class": "font-sans"},
        ordering=markupever.dom.Ordering.APPEND,
    )
    assert isinstance(p, markupever.dom.Element)
    assert p.parent == body
    assert p.name == "p"

    assert dom.namespaces() == {"": "http://www.w3.org/1999/xhtml", "ns1": "namespace1"}

    comment = p.create_comment("content")
    assert isinstance(comment, markupever.dom.Comment)
    assert comment.parent == p
    assert comment.content == "content"
    assert comment == "content"

    comment.content += "testme"
    assert comment == "contenttestme"

    assert p.text() == ""

    text = p.create_text("\ncontent")
    assert isinstance(text, markupever.dom.Text)
    assert text.parent == p
    assert text.content == "\ncontent"
    assert text == "\ncontent"

    text.content += " 1"

    p.create_text("\ncontent 2")

    assert p.text() == "\ncontent 1\ncontent 2"
    assert p.text(strip=True) == "content 1content 2"
    assert p.text(seperator="\t", strip=True) == "content 1\tcontent 2"

    assert text.has_siblings
    assert p.has_children
    assert p.tree() == dom

    assert (
        root.serialize()
        == '<!DOCTYPE HTML PUBLIC "public" SYSTEM "system"><html lang="en"><head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0"></head><body class="bg-dark"><p class="font-sans"><!--contenttestme-->\ncontent 1\ncontent 2</p></body></html>'
    )

    with pytest.raises(ValueError):
        root.create_comment("content", ordering=markupever.dom.Ordering.AFTER)

    with pytest.raises(ValueError):
        root.create_comment("content", ordering=markupever.dom.Ordering.BEFORE)

    comment = root.create_comment("content")
    assert isinstance(comment, markupever.dom.Comment)
    assert comment.content == "content"
    assert comment == "content"

    text = root.create_text("content")
    assert isinstance(text, markupever.dom.Text)
    assert text.content == "content"
    assert text == "content"

    pi = root.create_processing_instruction("data", "target")
    assert isinstance(pi, markupever.dom.ProcessingInstruction)
    assert pi.data == "data"
    assert pi.target == "target"

    pi.data = "md"
    pi.target = "mt"
    assert pi.data == "md"
    assert pi.target == "mt"

    assert root.first_child == doctype
    assert root.last_child == pi


def test_children():
    dom = markupever.dom.TreeDom()
    root = dom.root()

    testcases = [
        root.create_element("first"),
        root.create_text("second"),
        root.create_doctype("third"),
        root.create_processing_instruction("fourth", "target"),
        root.create_comment("fifth"),
        root.create_element("sixth"),
    ]

    for index, child in enumerate(root.children()):
        assert testcases[index] == child


def test_ancestors():
    dom = markupever.dom.TreeDom()
    root = dom.root()

    testcases = [root]
    parent = root
    for i in range(10):
        elem = parent.create_element(f"test{i}")
        testcases.append(elem)
        parent = elem

    testcases.pop()

    for index, ans in enumerate(parent.ancestors()):
        assert testcases[len(testcases) - (index + 1)] == ans


def test_siblings():
    dom = markupever.dom.TreeDom()
    root = dom.root()

    testcases = [
        root.create_element("first"),
        root.create_text("second"),
        root.create_doctype("third"),
        root.create_processing_instruction("fourth", "target"),
        root.create_comment("fifth"),
        root.create_element("sixth"),
    ]

    for index, sibling in enumerate(testcases[0].next_siblings()):
        assert testcases[index + 1] == sibling

    for index, sibling in enumerate(testcases[-1].prev_siblings()):
        assert testcases[len(testcases) - (index + 2)] == sibling

    # No need to test traverse - it's tested in Rust
    for edge in root.traverse():
        assert isinstance(edge.closed, bool)
        assert isinstance(edge.node, markupever.dom.BaseNode)
        repr(edge)


def test_detach():
    dom = markupever.dom.TreeDom()
    root = dom.root()

    with pytest.raises(ValueError):
        root.detach()

    with pytest.raises(ValueError):
        tmp_dom = markupever.dom.TreeDom()
        root.attach(tmp_dom.root())

    html = root.create_element("html")
    body = html.create_element("body")

    assert body.parent == html

    body.detach()
    assert body.parent is None

    root.attach(body)
    assert body.parent == root
    assert root.last_child == body

    html.attach(body)
    assert body.parent == html


def test_select():
    dom = markupever.parse(
        """<div class="title">
        <nav class="navbar">
            <p id="title">Hello World</p><p id="text">Hello World</p>
        </nav>
        <nav class="nav2"><p>World</p></nav>
        </div>""",
        markupever.HtmlOptions(),
    )

    count = 0
    for tag in dom.select("p:nth-child(1)"):
        count += 1
        assert isinstance(tag, markupever.dom.Element)
        assert tag.name == "p"

    assert count == 2

    count = 0
    for tag in dom.select("p:nth-child(1)", limit=1):
        count += 1
        assert isinstance(tag, markupever.dom.Element)
        assert tag.name == "p"

    assert count == 1

    count = 0
    for tag in dom.select("p"):
        count += 1
        assert isinstance(tag, markupever.dom.Element)
        assert tag.name == "p"

    assert count == 3

    count = 0
    for tag in dom.select("p", limit=1):
        count += 1
        assert isinstance(tag, markupever.dom.Element)
        assert tag.name == "p"
        assert tag.id == "title"

    assert count == 1

    count = 0
    for tag in dom.select("p", offset=3):
        count += 1
        assert isinstance(tag, markupever.dom.Element)
        assert tag.name == "p"
        assert tag.id is None

    assert count == 1

    tag = dom.select_one("nav.nav2")
    assert tag.name == "nav"
    assert tag.class_list == ["nav2"]
    assert tag.text() == "World"

    tag = dom.select_one("nav.nav2", offset=3)
    assert tag is None


def test_element():
    dom = markupever.dom.TreeDom()
    root = dom.root()

    html = root.create_element(
        "html", {"lang": "en", "class": "hello world"}, mathml_annotation_xml_integration_point=True
    )
    assert html.class_list == ["hello", "world"]
    assert html.template is False
    assert html.mathml_annotation_xml_integration_point is True

    html.name = markupever.dom.QualName("tag", "html")
    html.template = True
    html.mathml_annotation_xml_integration_point = False

    assert html.template is True
    assert html.mathml_annotation_xml_integration_point is False
    assert html.name == "tag"

    html.attrs["class"] = "hello man"
    assert html.class_list == ["hello", "man"]

    assert len(html.attrs) == 2
    del html.attrs[0]
    assert len(html.attrs) == 1

    assert html.class_list == ["hello", "man"]
    assert html.id is None

    html.attrs = [("id", "markup"), ("data-role", "button")]

    assert html.class_list == []
    html.attrs.append("class", "btn border")
    assert "btn" in html.class_list
    assert "border" in html.class_list
    assert html.attrs[0] == (markupever.dom.QualName("id"), "markup")
    assert html.id == "markup"
    assert html.attrs["id"] == "markup"
    assert html.attrs[markupever.dom.QualName("id")] == "markup"

    with pytest.raises(IndexError):
        html.attrs[10]

    with pytest.raises(KeyError):
        html.attrs["ali"]

    html.attrs.insert(0, "onclick", "alert")
    assert html.attrs[0] == (markupever.dom.QualName("onclick"), "alert")

    html.attrs = [("id", "id1"), ("id", "id2"), ("data-role", "button")]
    assert html.id == "id1"

    assert html.attrs.get("id") == "id1"
    assert html.attrs.get("id", start=1) == "id2"
    assert html.attrs.get("dt", start=1, default="h") == "h"

    assert html.attrs.index("id") == 0
    assert html.attrs.index("id", start=1) == 1
    assert html.attrs.index(("id", "id1")) == 0
    assert html.attrs.index(("id", "id2")) == 1

    with pytest.raises(ValueError):
        html.attrs.index(("id", "id3"))

    html.attrs.reverse()

    html.attrs.pop()
    assert len(html.attrs) == 2

    html.attrs.remove("data-role")
    assert len(html.attrs) == 1

    assert list(html.attrs) == [_rustlib.QualName("id")]
    assert list(html.attrs.values()) == ["id2"]

    with pytest.raises(KeyError):
        del html.attrs["a"]

    html.attrs["a"] = "b"
    del html.attrs["a"]

    html.attrs = {"a": "b"}

    assert "a" in html.attrs
    assert _rustlib.QualName("a") in html.attrs
    assert (_rustlib.QualName("a"), "b") in html.attrs

    repr(html.attrs)

    html.attrs.clear()
    assert len(html.attrs) == 0

    html.attrs.extend({"b": "c", "d": "e"})
    assert len(html.attrs) == 2
