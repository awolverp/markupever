from xmarkup._rustlib import QualName, HtmlOptions, XmlOptions, Parser
import pytest


def test_qualname():
    q = QualName("div")
    assert q.local == "div"
    assert q.ns == ""
    assert q.prefix is None

    q = QualName("div", "html")
    assert q.local == "div"
    assert q.ns == "http://www.w3.org/1999/xhtml"
    assert q.prefix is None

    q = QualName("div", "https://namespace1.org", prefix="ns1")
    assert q.local == "div"
    assert q.ns == "https://namespace1.org"
    assert q.prefix == "ns1"

    assert hash(q) == hash(q.copy())

    q1 = QualName("a")
    q2 = QualName("b")

    assert q1 < q2
    assert q1 != q2

    assert q1 != 1
    assert q1 != "a"
    assert q1 != "b"

    with pytest.raises(TypeError):
        q1 >= 1


def test_options():
    _ = HtmlOptions()
    _ = XmlOptions()


def test_parser():
    def _yield(contents: tuple):
        for i in contents:
            yield i

    _ = Parser(_yield((b"<html><p>Ali</p></html>",)), HtmlOptions())
    _ = Parser(_yield("<html><p>Ali</p></html>"), HtmlOptions())
    _ = Parser(_yield(("<html>", b"Ali", b"</html>")), XmlOptions())

    with pytest.raises(TypeError):
        _ = Parser(_yield(("Ali", 3, b"A")), XmlOptions())

    with pytest.raises(TypeError):
        _ = Parser(_yield, XmlOptions())

    with pytest.raises(TypeError):
        _ = Parser(("Ali", 3, b"A"), XmlOptions())

    with pytest.raises(TypeError):
        _ = Parser(b"<html><p>Ali</p></html>", HtmlOptions())

    parser = Parser(_yield((b"<html><p>Ali</p></html>",)), HtmlOptions(full_document=False))
    assert isinstance(parser.errors(), list)
    assert parser.lineno() == 1
    assert parser.quirks_mode() == 2

    parser = Parser(
        _yield((b"<html><p>Ali</p>", "\n", "</html>")), HtmlOptions(full_document=False)
    )
    assert parser.lineno() == 2

    _ = parser.into_dom()

    with pytest.raises(RuntimeError):
        parser.errors()
