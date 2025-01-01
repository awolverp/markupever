import pytest
from xmarkup import _rustlib


_HTML = """<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>

<body class="h-screen bg-zinc-900 text-white font-sans">
    <span class="opacity-50 md:opacity-100 duration-1000 z-[1] semi-circle bg-orange-500 rotate-90 fixed top-[60%] left-[-3rem]"></span>

    <header
        class="flex flex-row justify-between bg-zinc-op fixed w-full px-8 py-4 top-0 right-[50%] left-[50%] translate-x-[-50%] items-center md:w-3/4 md:rounded-2xl md:top-3 duration-500">

        <div>
            <img id="1" src="favicon.ico" alt="ICO" class="w-7 h-7 inline">
            <img id="2" src="favicon.ico" alt="ICO" class="w-7 h-7 inline">
            <p class="inline font-extrabold text-lg md:text-xl">Shine Frequency</p>
        </div>

        <div>
            <p class="font-mono text-sm md:text-lg">shinefrequency@gmail.com</p>
        </div>
    </header>

    <div class="mt-24 md:mt-28"></div>

    <div id="calendar" class="flex flex-row mx-auto justify-center gap-4">
        <div class="bg-zinc-700 text-center rounded-sm z-50 p-4 md:p-8">
            <p class="text-5xl md:text-7xl font-sans font-bold">14</p>
            <p class="text-base md:text-lg font-sans font-light">Days</p>
        </div>
        <div class="bg-zinc-700 text-center rounded-sm z-50 p-4 md:p-8">
            <p class="text-5xl md:text-7xl font-sans font-bold">08</p>
            <p class="text-base md:text-lg font-sans font-light">Hours</p>
        </div>
    </div>

</body>
</html>"""

_XML = """<bookstore>  
<book category="COOKING">  
<title lang="en">Everyday Italian</title>  
<author>Giada De Laurentiis</author>  
<year>2005</year>  
<price>30.00</price>  
</book>  
<book category="CHILDREN">  
<title lang="en">Harry Potter</title>  
<author>J K. Rowling</author>  
<year>2005</year>  
<price>29.99</price>  
</book>  
<book category="WEB">  
<title lang="en">Learning XML</title>  
<author>Erik T. Ray</author>  
<year>2003</year>  
<price>39.95</price>  
</book>  
</bookstore>"""


def test_html():
    html = _rustlib.Html(_HTML.encode("utf-8"), _rustlib.QUIRKS_MODE_OFF)

    assert not html.errors
    assert html.lineno == _HTML.count("\n") + 1
    assert html.quirks_mode == _rustlib.QUIRKS_MODE_OFF
    assert isinstance(html.root, _rustlib.Node)
    assert isinstance(html.root.data(), _rustlib.DocumentData)

    assert html.serialize() == html.root.serialize_html()

    # test parents() & tree()
    last_node = None
    for n in html.root.tree():
        assert isinstance(n, _rustlib.Node)
        last_node = n
    
    for p in last_node.parents():
        assert isinstance(p, _rustlib.Node)


def test_xml():
    xml = _rustlib.Xml(_XML.encode("utf-8"))

    assert not xml.errors
    assert xml.quirks_mode == _rustlib.QUIRKS_MODE_OFF
    assert isinstance(xml.root, _rustlib.Node)
    assert isinstance(xml.root.data(), _rustlib.DocumentData)

    assert xml.serialize() == xml.root.serialize_xml()


def test_qualname():
    qualname = _rustlib.QualName("name")
    assert qualname.local == "name"
    assert qualname.namespace == ""
    assert qualname.prefix is None

    with pytest.raises(AttributeError):
        # QualName properties is read-only
        qualname.local = "newname"

    qualname = _rustlib.QualName("div", "html", None)
    assert qualname.local == "div"
    assert qualname.namespace == "http://www.w3.org/1999/xhtml"
    assert qualname.prefix is None

    qualname = _rustlib.QualName("span", "xhtml", None)
    assert qualname.local == "span"
    assert qualname.namespace == "http://www.w3.org/1999/xhtml"
    assert qualname.prefix is None

    qualname = _rustlib.QualName("test", "*", "prefix")
    assert qualname.local == "test"
    assert qualname.namespace == "*"
    assert qualname.prefix == "prefix"

    qualname = _rustlib.QualName("test", "custom-ns", None)
    assert qualname.local == "test"
    assert qualname.namespace == "custom-ns"
    assert qualname.prefix is None

    assert _rustlib.QualName("test") == _rustlib.QualName("test")
    assert _rustlib.QualName("test") != _rustlib.QualName("test", "html")


def _construct_data(cls: type, is_name: str, *args, **kwargs) -> object:
    obj = cls(*args, **kwargs)

    assert isinstance(obj, cls)

    assert isinstance(obj.as_node(), _rustlib.Node)

    node = _rustlib.Node(obj)
    assert isinstance(node, _rustlib.Node)
    assert isinstance(node.data(), cls)

    assert node == obj

    assert getattr(node, is_name)() is True

    return obj


def _writable_properties(obj, attrnames, correct_cases: list[tuple], wrong_cases: tuple):
    for attrname in attrnames:
        for val, t in correct_cases:
            setattr(obj, attrname, val)
            assert getattr(obj, attrname) == t

        for wrong in wrong_cases:
            with pytest.raises((TypeError, ValueError)):
                setattr(obj, attrname, wrong)


def test_datas():
    _ = _construct_data(_rustlib.DocumentData, "is_document")
    _ = _construct_data(_rustlib.FragmentData, "is_fragment")

    doctype = _construct_data(_rustlib.DoctypeData, "is_doctype", "name", "public_id", "system_id")
    assert doctype.name == "name"
    assert doctype.public_id == "public_id"
    assert doctype.system_id == "system_id"
    _writable_properties(
        doctype,
        ("name", "public_id", "system_id"),
        [("html", "html"), ("xml", "xml"), ("xmlns", "xmlns"), ("test", "test")],
        (1, 2.4, []),
    )

    comment = _construct_data(_rustlib.CommentData, "is_comment", "this is a comment")
    assert comment.contents == "this is a comment"
    _writable_properties(
        comment,
        ("contents",),
        [
            ("test", "test"),
            ("w\na", "w\na"),
        ],
        (1, 2.4, []),
    )

    text = _construct_data(_rustlib.TextData, "is_text", "this is a text")
    assert text.contents == "this is a text"
    _writable_properties(
        text,
        ("contents",),
        [("test", "test"), ("w\na", "w\na")],
        (1, 2.4, []),
    )

    pi = _construct_data(
        _rustlib.ProcessingInstructionData, "is_processing_instruction", "data", "target"
    )
    assert pi.data == "data"
    assert pi.target == "target"
    _writable_properties(
        pi,
        ("data", "target"),
        [("test", "test"), ("w\na", "w\na")],
        (1, 2.4, []),
    )

    element = _construct_data(
        _rustlib.ElementData, "is_element", _rustlib.QualName("div"), [], False, True
    )
    assert element.name == _rustlib.QualName("div")
    assert len(element.attrs) == 0
    assert element.template is False
    assert element.mathml_annotation_xml_integration_point is True
    _writable_properties(
        element,
        ("name",),
        [
            ("span", _rustlib.QualName("span")),
            ("custom-name", _rustlib.QualName("custom-name")),
            (_rustlib.QualName("span", "html", None), _rustlib.QualName("span", "html", None)),
            (
                _rustlib.QualName("span", "html", "prefix"),
                _rustlib.QualName("span", "html", "prefix"),
            ),
        ],
        (1, []),
    )
    _writable_properties(
        element,
        ("template", "mathml_annotation_xml_integration_point"),
        [
            (False, False),
            (True, True),
        ],
        (1, []),
    )

    with pytest.raises(AttributeError):
        element.attrs = []

    element = _rustlib.ElementData(
        "span", [("id", "3"), ("custom-attr", "val"), (_rustlib.QualName("qual", "xml"), "")]
    )
    assert element.name == _rustlib.QualName("span")
    assert len(element.attrs) == 3

    assert element.attrs[0] == (_rustlib.QualName("id"), "3")
    assert element.attrs[1] == (_rustlib.QualName("custom-attr"), "val")
    assert element.attrs[2] == (_rustlib.QualName("qual", "xml"), "")

    assert element.id == "3"
    assert element.classes == []


def test_element_attrs():
    element = _rustlib.ElementData(
        "span",
        [
            ("id", "par"),
            ("custom-attr", "val"),
            (_rustlib.QualName("class"), "table flex flex-col"),
        ],
    )
    assert len(element.attrs) == 3
    assert element.id == "par"
    assert element.classes.sort() == ["table", "flex", "flex-col"].sort()
    assert element.attrs[0] == (_rustlib.QualName("id"), "par")
    assert element.attrs[1] == (_rustlib.QualName("custom-attr"), "val")
    assert element.attrs[2] == (_rustlib.QualName("class"), "table flex flex-col")

    del element.attrs[0]  # del id
    assert element.id is None
    del element.attrs[1]  # del class
    assert element.classes == []

    element.attrs[0] = (_rustlib.QualName("data-type"), "3")
    assert element.attrs[0] == (_rustlib.QualName("data-type"), "3")

    element.attrs.append((_rustlib.QualName("id"), "newid"))
    element.attrs.append((_rustlib.QualName("class"), "mt-0 px-10"))
    assert len(element.attrs) == 3

    for qual, val in element.attrs:
        pass

    for qual, val in element.attrs:
        pass

    attrs = element.attrs
    iter(attrs)

    with pytest.raises(RuntimeError):
        iter(attrs)

    attrs.sort()
    assert element.attrs[0][0] == _rustlib.QualName("class")
    assert element.attrs[1][0] == _rustlib.QualName("data-type")
    assert element.attrs[2][0] == _rustlib.QualName("id")

    element.attrs.clear()
    assert len(element.attrs) == 0


def test_node_children():
    root = _rustlib.Node(_rustlib.DocumentData())

    assert len(root.children()) == 0

    root.children().append(_rustlib.CommentData("content"))

    assert len(root.children()) == 1

    with pytest.raises(RuntimeError):
        root.children().append(root)

    assert isinstance(root.children()[0].data(), _rustlib.CommentData)

    with pytest.raises(IndexError):
        root.children()[1]

    root.children()[0] = _rustlib.TextData("content")
    assert isinstance(root.children()[0].data(), _rustlib.TextData)

    root.children().append(_rustlib.Node(_rustlib.ElementData("div", [])))

    del root.children()[0]
    assert len(root.children()) == 1

    newroot = _rustlib.Node(_rustlib.DocumentData())
    newroot.children().append(root)

    with pytest.raises(ValueError):
        newroot.children().append(root)

    assert newroot.children()[0].children()[0].is_element()

    for n in newroot.children():
        for _ in n.children():
            pass

    children = newroot.children()
    children2 = newroot.children()

    children.clear()
    assert len(children2) == 0


def test_parent():
    root = _rustlib.DocumentData().as_node()
    element = _rustlib.ElementData("html", []).as_node()

    root.children().append(element)
    
    assert len(root.children()) == 1
    assert element.parent() is not None
    assert element.parent() == root

    element.unlink()
    assert len(root.children()) == 0
    assert element.parent() is None

    root.children().append(element)
    del root.children()[0]
    assert len(root.children()) == 0
    assert element.parent() is None


def _to_text(node):
    text = ""

    tree = node.tree()

    for n in tree:
        if n.is_text():
            text += n.data().contents.strip()
    
    return text


def test_select():
    html = _rustlib.Html(_HTML.encode("utf-8"), _rustlib.QUIRKS_MODE_OFF)

    for node in html.root.select("p"):
        data = node.data()
        assert isinstance(data, _rustlib.ElementData)
        assert data.name.local == "p"

    for node in html.root.select("header div > img:first-child"):
        data = node.data()
        assert isinstance(data, _rustlib.ElementData)
        assert data.name.local == "img"
        assert data.id == "1"
