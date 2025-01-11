from xmarkup.driver import Html, HtmlOptions, Xml, XmlOptions
from xmarkup import nodes


def test_html():
    html = Html(b"<div><p>Ali</p></div>", HtmlOptions(full_document=False))

    assert isinstance(html.root, nodes.Node)
    assert isinstance(html.errors, list)
    assert isinstance(html.quirks_mode, int)


def test_xml():
    xml = Xml(b"<div><p>Ali</p></div>", XmlOptions(exact_errors=True))

    assert isinstance(xml.root, nodes.Node)
    assert isinstance(xml.errors, list)


def test_node():
    n1 = nodes.Node(nodes.DocumentData())

    _ = nodes.Node(n1)
    _ = nodes.Node(n1._node)
    element = nodes.Node(nodes.ElementData(nodes.QualName("div"), []))

    assert isinstance(element.data(), nodes.ElementData)
    assert element.is_element()
    assert isinstance(element.copy(), nodes.Node)
    assert isinstance(element.children(), nodes.Children)
    assert element.parent() is None
    assert isinstance(element.parents(), nodes.ParentsIterator)
    assert isinstance(element.tree(), nodes.TreeIterator)
    assert isinstance(element.select("div"), nodes.Matching)
    assert element == nodes.ElementData(nodes.QualName("div"), [])
    element.__repr__()


def test_children_and_parent():
    root = nodes.Node(nodes.DocumentData())
    assert len(root.children()) == 0

    root.children().append(nodes.CommentData("comment1"))

    children = nodes.Children(root._node)
    assert len(children) == 1

    assert isinstance(children[0], nodes.Node)

    text = nodes.TextData("content")
    root.children()[0] = text

    assert nodes.Node(text).parent() == root

    children.pop()
    assert len(root.children()) == 0

    for n in root.children():
        assert isinstance(n, nodes.Node)

        for _ in n.children():
            pass


def test_tree_and_parents():
    xml = Xml(b"<div><p>Ali</p><p>Wolf</p></div>")

    last = None
    for n in xml.root.tree():
        assert isinstance(n, nodes.Node)
        last = n

    assert last.data() == nodes.TextData("Wolf")

    first = None
    for n in last.parents():
        assert isinstance(n, nodes.Node)
        first = n

    assert first == xml.root


def test_select():
    html = Html(b"<div><p>Ali</p><p>Wolf</p></div>", HtmlOptions(exact_errors=True))

    for n in html.root.select("div > p"):
        assert isinstance(n, nodes.Node)
        assert n.data().name.local == "p"
