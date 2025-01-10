from xmarkup.driver import Html, HtmlOptions, Xml, XmlOptions
from xmarkup import node


def test_html():
    html = Html(b"<div><p>Ali</p></div>", HtmlOptions(full_document=False))

    assert isinstance(html.root, node.Node)
    assert isinstance(html.errors, list)
    assert isinstance(html.quirks_mode, int)


def test_xml():
    xml = Xml(b"<div><p>Ali</p></div>", XmlOptions(exact_errors=True))

    assert isinstance(xml.root, node.Node)
    assert isinstance(xml.errors, list)
    assert isinstance(xml.quirks_mode, int)


def test_node():
    n1 = node.Node(node.DocumentData())

    _ = node.Node(n1)
    _ = node.Node(n1._node)
    element = node.Node(node.ElementData(node.QualName("div"), []))

    assert isinstance(element.data(), node.ElementData)
    assert element.is_element()
    assert isinstance(element.copy(), node.Node)
    assert isinstance(element.children(), node.Children)
    assert element.parent() is None
    assert isinstance(element.parents(), node.ParentsIterator)
    assert isinstance(element.tree(), node.TreeIterator)
    assert isinstance(element.select("div"), node.SelectExpr)
    assert element == node.ElementData(node.QualName("div"), [])
    element.__repr__()


def test_children_and_parent():
    root = node.Node(node.DocumentData())
    assert len(root.children()) == 0

    root.children().append(node.CommentData("comment1"))

    children = node.Children(root._node)
    assert len(children) == 1

    assert isinstance(children[0], node.Node)

    text = node.TextData("content")
    root.children()[0] = text

    assert node.Node(text).parent() == root

    children.pop()
    assert len(root.children()) == 0

    for n in root.children():
        assert isinstance(n, node.Node)

        for _ in n.children():
            pass


def test_tree_and_parents():
    xml = Xml(b"<div><p>Ali</p><p>Wolf</p></div>")

    last = None
    for n in xml.root.tree():
        assert isinstance(n, node.Node)
        last = n

    assert last.data() == node.TextData("Wolf")

    first = None
    for n in last.parents():
        assert isinstance(n, node.Node)
        first = n

    assert first == xml.root


def test_select():
    html = Html(b"<div><p>Ali</p><p>Wolf</p></div>", HtmlOptions(exact_errors=True))

    for n in html.root.select("div > p"):
        assert isinstance(n, node.Node)
        assert n.data().name.local == "p"
