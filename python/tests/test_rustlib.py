import xmarkup._rustlib as rl
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
    assert q1 != "a"
    assert q1 != "b"

    with pytest.raises(TypeError):
        q1 >= 1


def test_options():
    _ = rl.HtmlOptions()
    _ = rl.XmlOptions()


def test_parser_generators():
    parser = rl.Parser(rl.HtmlOptions())
    parser.process(b"<html><p>Ali</p></html>")
    parser.finish()

    parser = rl.Parser(rl.HtmlOptions())
    parser.process("<html><p>Ali</p></html>")

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

    with pytest.raises(RuntimeError):
        parser.errors()


def test_document():
    dom = rl.TreeDom()

    with pytest.raises(NotImplementedError):
        rl.Document(dom)

    assert isinstance(dom.root(), rl.Document)
    assert dom.root() == dom.root()


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


def test_comment():
    dom = rl.TreeDom()
    x = rl.Comment(dom, "test")

    with pytest.raises(TypeError):
        rl.Comment(x, "xml")

    with pytest.raises(TypeError):
        rl.Comment("", "xml")

    assert x.parent() is None  # make sure it is orphan

    assert x.contents == "test"
    x.contents = "I am comment"
    assert x.contents == "I am comment"


def test_text():
    dom = rl.TreeDom()
    x = rl.Text(dom, "test")

    with pytest.raises(TypeError):
        rl.Text(x, "xml")

    with pytest.raises(TypeError):
        rl.Text("", "xml")

    assert x.parent() is None  # make sure it is orphan

    assert x.contents == "test"
    x.contents = "I am text"
    assert x.contents == "I am text"


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
