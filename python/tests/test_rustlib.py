import markupever._rustlib as rl
import pytest


def test_qualname():
    q = rl.QualName("div")
    assert q.local == "div"
    assert q.ns == ""
    assert q.prefix is None

    q = rl.QualName("div", "html")
    assert q.local == "div"
    assert q.ns == "http://www.w3.org/1999/xhtml"
    assert q.prefix is None

    q = rl.QualName("div", "https://namespace1.org", prefix="ns1")
    assert q.local == "div"
    assert q.ns == "https://namespace1.org"
    assert q.prefix == "ns1"

    assert hash(q) == hash(q.copy())

    q1 = rl.QualName("a")
    q2 = rl.QualName("b")

    assert q1 < q2
    assert q1 != q2

    assert q1 != 1
    assert q1 == "a"
    assert q1 != "b"

    with pytest.raises(TypeError):
        q1 >= 1

    repr(q1)


def test_options():
    repr(rl.HtmlOptions())
    repr(rl.XmlOptions())


def test_parser_generators():
    parser = rl.Parser(rl.HtmlOptions())
    parser.process(b"<html><p>Ali</p></html>")
    parser.finish()

    repr(parser)

    parser = rl.Parser(rl.HtmlOptions())
    parser.process("<html><p>Ali</p></html>")

    repr(parser)

    with pytest.raises(TypeError):
        parser.process(1)

    with pytest.raises(RuntimeError):
        parser.into_dom()

    parser.finish()

    with pytest.raises(RuntimeError):
        parser.process("")

    with pytest.raises(RuntimeError):
        parser.finish()

    parser.into_dom()
    with pytest.raises(RuntimeError):
        parser.into_dom()

    parser = rl.Parser(rl.XmlOptions())
    for c in ("<html>", b"Ali", b"</html>"):
        parser.process(c)
    parser.finish()

    assert isinstance(parser.errors(), list)
    assert parser.lineno() == 1
    assert parser.quirks_mode() == 2

    parser = rl.Parser(rl.HtmlOptions(full_document=False))
    for c in (b"<html><p>Ali</p>", "\n", "</html>"):
        parser.process(c)
    parser.finish()

    assert parser.lineno() == 2

    _ = parser.into_dom()

    repr(parser)

    with pytest.raises(RuntimeError):
        parser.errors()


def test_document():
    dom = rl.TreeDom()

    with pytest.raises(NotImplementedError):
        rl.Document(dom)

    assert isinstance(dom.root(), rl.Document)
    assert dom.root() == dom.root()

    repr(dom)


def test_doctype():
    dom = rl.TreeDom()
    doctype = rl.Doctype(dom, "html", "", system_id="hello")

    with pytest.raises(TypeError):
        rl.Doctype(doctype, "xml")

    with pytest.raises(TypeError):
        rl.Doctype(1, "xml")

    assert doctype.parent() is None  # make sure it is orphan

    assert doctype.name == "html"
    assert doctype.system_id == "hello"
    assert doctype.public_id == ""

    doctype.name = "xml"
    doctype.public_id = "test"

    assert doctype.name == "xml"
    assert doctype.system_id == "hello"
    assert doctype.public_id == "test"

    repr(doctype)


def test_comment():
    dom = rl.TreeDom()
    x = rl.Comment(dom, "test")

    with pytest.raises(TypeError):
        rl.Comment(x, "xml")

    with pytest.raises(TypeError):
        rl.Comment("", "xml")

    assert x.parent() is None  # make sure it is orphan

    assert x.content == "test"
    x.content = "I am comment"
    assert x.content == "I am comment"

    repr(x)


def test_text():
    dom = rl.TreeDom()
    x = rl.Text(dom, "test")

    with pytest.raises(TypeError):
        rl.Text(x, "xml")

    with pytest.raises(TypeError):
        rl.Text("", "xml")

    assert x.parent() is None  # make sure it is orphan

    assert x.content == "test"
    x.content = "I am text"
    assert x.content == "I am text"

    repr(x)


def test_element():
    dom = rl.TreeDom()
    x = rl.Element(dom, "body", [], False, False)

    with pytest.raises(TypeError):
        rl.Element(x, "d", [], False, False)

    with pytest.raises(TypeError):
        rl.Element(b"", "d", [], False, False)

    assert x.parent() is None  # make sure it is orphan

    assert x.name == rl.QualName("body", "", None)
    assert isinstance(x.attrs, rl.AttrsList)
    assert x.template is False
    assert x.mathml_annotation_xml_integration_point is False

    x = rl.Element(dom, rl.QualName("div", "html", "ns"), [], False, True)

    assert x.name == rl.QualName("div", "html", "ns")
    assert isinstance(x.attrs, rl.AttrsList)
    assert x.template is False
    assert x.mathml_annotation_xml_integration_point is True

    with pytest.raises(TypeError):
        rl.Element(dom, rl.QualName("div", "html", "ns"), {}, False, True)

    rl.Element(dom, rl.QualName("div", "html", "ns"), [("a", "b")], False, True)
    rl.Element(dom, rl.QualName("div", "html", "ns"), [("a", "b"), ("c", "d")], False, True)
    rl.Element(
        dom, rl.QualName("div", "html", "ns"), [(rl.QualName("a"), "b"), ("c", "d")], False, True
    )
    rl.Element(
        dom, rl.QualName("div", "html", "ns"), ((rl.QualName("a"), "b"), ("c", "d")), False, True
    )

    with pytest.raises(TypeError):
        rl.Element(
            dom,
            rl.QualName("div", "html", "ns"),
            [(rl.QualName("a"), rl.QualName("b"))],
            False,
            True,
        )

    with pytest.raises(TypeError):
        rl.Element(dom, rl.QualName("div", "html", "ns"), [rl.QualName("a")], False, False)

    x = rl.Element(dom, rl.QualName("div", "html", "ns"), [], False, True)

    assert x.name == rl.QualName("div", "html", "ns")
    assert isinstance(x.attrs, rl.AttrsList)
    assert x.template is False
    assert x.mathml_annotation_xml_integration_point is True

    x.template = True
    x.mathml_annotation_xml_integration_point = False
    x.name = rl.QualName("html")
    x.attrs = []
    x.attrs = [(rl.QualName("a"), "b"), ("c", "d")]

    assert x.name == rl.QualName("html")
    assert isinstance(x.attrs, rl.AttrsList)
    assert x.template is True
    assert x.mathml_annotation_xml_integration_point is False

    x.name = "template"

    assert x.name == rl.QualName("template")

    repr(x)


def _get_attr(attrs: rl.AttrsList, name):
    for index, v in enumerate(attrs.items()):
        if v[0] == name:
            return index, v[1]

    return None


def test_element_attrs():
    dom = rl.TreeDom()
    x = rl.Element(dom, "body", [("class", "flex"), ("id", "main")], False, False)
    x = rl.Element(
        dom, "body", [(rl.QualName("class"), "flex"), (rl.QualName("id"), "main")], False, False
    )

    with pytest.raises(TypeError):
        rl.Element(dom, "wolf", (1, ""), False, False)

    with pytest.raises(TypeError):
        rl.Element(dom, "temple", {}, False, False)

    with pytest.raises(TypeError):
        rl.Element(dom, "hello", 0, False, False)

    assert len(x.attrs) == 2
    x.attrs.clear()
    assert len(x.attrs) == 0

    x.attrs.push("id", "panel")
    x.attrs.push(rl.QualName("class", "html"), "flex")

    assert _get_attr(x.attrs, "id") == (0, "panel")
    assert _get_attr(x.attrs, rl.QualName("id")) == (0, "panel")

    assert _get_attr(x.attrs, "class") == (1, "flex")
    assert _get_attr(x.attrs, rl.QualName("class", "html")) == (1, "flex")
    assert _get_attr(x.attrs, rl.QualName("class")) is None
    assert _get_attr(x.attrs, rl.QualName("class", "xml")) is None

    index, _ = _get_attr(x.attrs, "id")
    x.attrs.update_value(index, "x")

    assert _get_attr(x.attrs, "id") == (0, "x")

    with pytest.raises(IndexError):
        x.attrs.update_value(10, "x")

    x.attrs.remove(0)
    assert len(x.attrs) == 1
    assert _get_attr(x.attrs, "id") is None

    x.attrs.swap_remove(0)
    assert len(x.attrs) == 0
    assert _get_attr(x.attrs, "class") is None

    x = rl.Element(
        dom, "body", [("class", "flex"), ("class", "main"), ("class", "main")], False, False
    )

    assert len(x.attrs) == 3
    x.attrs.dedup()
    assert len(x.attrs) == 2

    assert _get_attr(x.attrs, "class") == (0, "flex")
    x.attrs.reverse()
    assert _get_attr(x.attrs, "class") == (0, "main")

    repr(x.attrs)


def test_pi():
    dom = rl.TreeDom()
    x = rl.ProcessingInstruction(dom, "d", target="t")

    with pytest.raises(TypeError):
        rl.ProcessingInstruction(x, "d", "t")

    with pytest.raises(TypeError):
        rl.ProcessingInstruction("", "d", "t")

    assert x.parent() is None  # make sure it is orphan

    assert x.data == "d"
    assert x.target == "t"

    x.data = "I am data"

    assert x.data == "I am data"
    assert x.target == "t"

    repr(x)


def test_append_prepend():
    # this test is enough because these methods uses ego_tree crate of Rust which is completely tested
    dom = rl.TreeDom()
    doctype = rl.Doctype(dom, "html", "", system_id="hello")

    dom.append(dom.root(), doctype)

    assert doctype.parent() == dom.root()
    assert dom.root().first_child() == doctype

    x = rl.Element(
        dom, rl.QualName("head", "mynamespace"), [("class", "flex"), ("id", "main")], False, False
    )

    assert dom.namespaces() == {}

    dom.prepend(dom.root(), x)

    assert dom.namespaces() == {"": "mynamespace"}

    assert x.parent() == dom.root()
    assert dom.root().first_child() == x

    y = rl.Element(
        dom,
        rl.QualName("head", "namespace2", "ns"),
        [("class", "flex"), ("id", "main")],
        False,
        False,
    )

    assert dom.namespaces() == {"": "mynamespace"}
    dom.insert_before(x, y)

    assert dom.namespaces() == {"": "mynamespace", "ns": "namespace2"}

    y = rl.Element(
        dom, rl.QualName("head", "namespace3"), [("class", "flex"), ("id", "main")], False, False
    )
    dom.insert_after(x, y)

    assert dom.namespaces() == {"": "mynamespace", "ns": "namespace2"}


def test_iterator():
    dom = rl.TreeDom()

    testcase = [
        rl.Doctype(dom, "one", "", ""),
        rl.Element(dom, "two", [], False, False),
        rl.Text(dom, "three"),
        rl.Comment(dom, "four"),
        rl.Element(dom, "five", [("class", "flex flex-row")], False, False),
        rl.Element(dom, rl.QualName("html", "html", None), [], False, False),
        rl.ProcessingInstruction(dom, "1", "2"),
    ]

    for index, node in enumerate(rl.iter.Iterator(dom)):
        if isinstance(node, rl.Document):
            continue

        assert testcase[index - 1] == node

        dom.append(dom.root(), node)

    for index, node in enumerate(rl.iter.Iterator(dom)):
        if isinstance(node, rl.Document):
            continue

        assert testcase[index - 1] == node


def test_ancestors():
    dom = rl.TreeDom()

    testcase = [
        rl.Doctype(dom, "one", "", ""),
        rl.Element(dom, "html", [(rl.QualName("lang", "", None), "en")], False, False),
        rl.Element(dom, "head", [(rl.QualName("lang", "", None), "en")], False, False),
        rl.Text(dom, "Hello World"),
    ]

    dom.append(dom.root(), testcase[0])
    dom.append(dom.root(), testcase[1])
    dom.append(testcase[1], testcase[2])
    dom.append(testcase[2], testcase[3])

    result = list(rl.iter.Ancestors(testcase[3]))

    assert isinstance(result[-1], rl.Document)
    assert result[-2] == testcase[1]


def test_children():
    dom = rl.TreeDom()

    testcase = [
        rl.Doctype(dom, "one", "", ""),
        rl.Element(dom, "html", [(rl.QualName("lang", "", None), "en")], False, False),
        rl.Element(dom, "head", [(rl.QualName("lang", "", None), "en")], False, False),
        rl.Text(dom, "Hello World"),
    ]

    for n in testcase:
        dom.append(dom.root(), n)

    result = list(rl.iter.Children(dom.root()))

    assert result == testcase


def test_traverse():
    dom = rl.TreeDom()

    testcase = [
        rl.Doctype(dom, "one", "", ""),
        rl.Element(dom, "html", [(rl.QualName("lang", "", None), "en")], False, False),
        rl.Element(dom, "head", [(rl.QualName("lang", "", None), "en")], False, False),
        rl.Text(dom, "Hello World"),
    ]

    dom.append(dom.root(), testcase[0])
    dom.append(dom.root(), testcase[1])
    dom.append(testcase[1], testcase[2])
    dom.append(testcase[2], testcase[3])

    expected = [
        (testcase[0], False),
        (testcase[0], True),
        (testcase[1], False),
        (testcase[2], False),
        (testcase[3], False),
        (testcase[3], True),
        (testcase[2], True),
        (testcase[1], True),
    ]
    result = list(rl.iter.Traverse(dom.root()))[1:-1]

    assert result == expected


def _get_text(node) -> str:
    s = ""

    for n in rl.iter.Descendants(node):
        if isinstance(n, rl.Text):
            s += n.content

    return s.strip()


def test_select():
    p = rl.Parser(rl.HtmlOptions())
    p.process(
        """<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Document</title>
</head>
<body>
    <div id="header" data-role="banner">
        <h1 title="main heading">Welcome</h1>
        <p>This is paragraph 1</p>
        <p>This is paragraph 2</p>
    </div>
    <ul class="menu">
        <li class="example">Item 1</li>
        <li>Item 2</li>
        <li>Item 3</li>
    </ul>
    <a href="https://example.com" class="external">External Link</a>
    <input type="text" required>
    <input type="checkbox" checked>
    <div class="box"></div>
    <p lang="en">English</p>
    <p lang="fr">Texte en</p>
</body>
</html>"""
    )
    p.finish()
    d = p.into_dom()

    is_ok = False
    for node in rl.Select(d.root(), '[href*="example"]'):
        assert node.name == "a"

        _, v = _get_attr(node.attrs, "href")
        assert "example" in v
        is_ok = True

    assert is_ok

    is_ok = False
    for node in rl.Select(d.root(), "div p:nth-of-type(2)"):
        assert node.name == "p"

        assert _get_text(node) == "This is paragraph 2"
        is_ok = True

    assert is_ok

    is_ok = False
    for node in rl.Select(d.root(), "div:empty"):
        assert node.name == "div"

        assert node.class_list() == ["box"]

        is_ok = True

    assert is_ok

    is_ok = False
    for node in rl.Select(d.root(), "div[data-role] p"):
        assert node.name == "p"
        assert _get_text(node).startswith("This is paragraph")

        is_ok = True

    assert is_ok


def test_serialize():
    parser = rl.Parser(rl.HtmlOptions(full_document=True))
    for c in ("<html>", b"<body>Ali</body>", b"</html>"):
        parser.process(c)
    parser.finish()

    dom = parser.into_dom()

    assert rl.serialize(dom.root()) == b"<html><head></head><body>Ali</body></html>"
    assert (
        rl.serialize(dom.root(), False)
        == b'<html xmlns="http://www.w3.org/1999/xhtml"><head></head><body>Ali</body></html>'
    )

    parser = rl.Parser(rl.HtmlOptions(full_document=False))
    for c in ("<hello>", b"Ali", b"</hello>"):
        parser.process(c)
    parser.finish()

    dom = parser.into_dom()

    assert rl.serialize(dom.root(), True) == b"<html><hello>Ali</hello></html>"
    assert (
        rl.serialize(dom.root(), False)
        == b'<html xmlns="http://www.w3.org/1999/xhtml"><hello>Ali</hello></html>'
    )

    parser = rl.Parser(rl.XmlOptions())
    for c in ("<hello>", b"Ali", b"</hello>"):
        parser.process(c)
    parser.finish()

    dom = parser.into_dom()

    assert rl.serialize(dom.root()) == b"<hello>Ali</hello>"
    assert rl.serialize(dom.root(), False) == b"<hello>Ali</hello>"
