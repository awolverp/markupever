from xmarkup._rustlib import QualName
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
