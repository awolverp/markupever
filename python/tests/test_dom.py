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
    _test_rustlib_node_convert(_rustlib.ProcessingInstruction, xmarkup.dom.ProcessingInstruction, dom, "name", "data")

    with pytest.raises(TypeError):
        xmarkup.dom.BaseNode("invalid type")

    xmarkup.dom.Doctype(_rustlib.Doctype(dom._raw, "m", "", ""))

    with pytest.raises(TypeError):
        xmarkup.dom.Element(_rustlib.Doctype(dom._raw, "m", "", ""))

    with pytest.raises(TypeError):
        xmarkup.dom.BaseNode._wrap(1)
