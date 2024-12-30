import pytest
from xmarkup import _rustlib


_HTML = """<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>TailwindCss</title>

    <link rel="stylesheet" href="output.css">
</head>

<body class="h-screen bg-zinc-900 text-white font-sans">

    <span class="opacity-50 md:opacity-100 duration-1000 z-[1] semi-circle bg-orange-500 rotate-90 fixed top-[60%] left-[-3rem]"></span>
    <span class="opacity-50 md:opacity-100 duration-1000 z-[1] quarter-circle bg-white -rotate-90 fixed top-[calc(60%-3rem)] left-[5rem]"></span>
    <span class="opacity-50 md:opacity-100 duration-1000 z-[1] semi-circle bg-yellow-500 fixed top-[calc(60%+3rem)] left-[5rem]"></span>

    <span class="opacity-50 md:opacity-100 duration-1000 z-[1] circle-alike bg-white fixed top-[calc(30%-3rem)] right-[-3rem]"></span>
    <span class="opacity-50 md:opacity-100 duration-1000 z-[1] semi-circle bg-orange-500 fixed top-[calc(30%-3rem)] right-[8rem]"></span>
    <span class="opacity-50 md:opacity-100 duration-1000 z-[1] semi-circle bg-yellow-500 rotate-180 fixed top-[calc(30%+3rem)] right-[2rem]"></span>

    <header
        class="flex flex-row justify-between bg-zinc-op fixed w-full px-8 py-4 top-0 right-[50%] left-[50%] translate-x-[-50%] items-center md:w-3/4 md:rounded-2xl md:top-3 duration-500">

        <div>
            <img src="favicon.ico" alt="ICO" class="w-7 h-7 inline">
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
        <div class="bg-zinc-700 text-center rounded-sm z-50 p-4 md:p-8">
            <p class="text-5xl md:text-7xl font-sans font-bold">07</p>
            <p class="text-base md:text-lg font-sans font-light">Mins</p>
        </div>
        <div class="bg-zinc-700 text-center rounded-sm z-50 p-4 md:p-8">
            <p class="text-5xl md:text-7xl font-sans font-bold">58</p>
            <p class="text-base md:text-lg font-sans font-light">Secs</p>
        </div>
    </div>

</body>

</html>
"""

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

    html.serialize()


def test_xml():
    xml = _rustlib.Xml(_XML.encode("utf-8"))

    assert not xml.errors
    assert xml.quirks_mode == _rustlib.QUIRKS_MODE_OFF
    assert isinstance(xml.root, _rustlib.Node)
    assert isinstance(xml.root.data(), _rustlib.DocumentData)

    xml.serialize()


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


def _construct_data(cls: type, *args, **kwargs) -> object:
    obj = cls(*args, **kwargs)

    assert isinstance(obj, cls)

    assert isinstance(obj.as_node(), _rustlib.Node)
    assert isinstance(_rustlib.Node(obj), _rustlib.Node)

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
    _ = _construct_data(_rustlib.DocumentData)
    _ = _construct_data(_rustlib.FragmentData)

    doctype = _construct_data(_rustlib.DoctypeData, "name", "public_id", "system_id")
    assert doctype.name == "name"
    assert doctype.public_id == "public_id"
    assert doctype.system_id == "system_id"
    _writable_properties(
        doctype,
        ("name", "public_id", "system_id"),
        [("html", "html"), ("xml", "xml"), ("xmlns", "xmlns"), ("test", "test")],
        (1, 2.4, []),
    )

    comment = _construct_data(_rustlib.CommentData, "this is a comment")
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

    text = _construct_data(_rustlib.TextData, "this is a text")
    assert text.contents == "this is a text"
    _writable_properties(
        text,
        ("contents",),
        [("test", "test"), ("w\na", "w\na")],
        (1, 2.4, []),
    )

    pi = _construct_data(_rustlib.ProcessingInstructionData, "data", "target")
    assert pi.data == "data"
    assert pi.target == "target"
    _writable_properties(
        pi,
        ("data", "target"),
        [("test", "test"), ("w\na", "w\na")],
        (1, 2.4, []),
    )

    element = _construct_data(_rustlib.ElementData, _rustlib.QualName("div"), [], False, True)
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
            (_rustlib.QualName("span", "html", "prefix"), _rustlib.QualName("span", "html", "prefix")),
        ],
        (1, [])
    )
    _writable_properties(
        element,
        ("template", "mathml_annotation_xml_integration_point"),
        [
            (False, False),
            (True, True),
        ],
        (1, [])
    )

    with pytest.raises(AttributeError):
        element.attrs = []

    element = _rustlib.ElementData("span", [ ("data-type", "3"), ("custom-attr", "val"), (_rustlib.QualName("qual", "xml"), "") ])
    assert element.name == _rustlib.QualName("span")
    assert len(element.attrs) == 3

    assert element.attrs[0] == (_rustlib.QualName("data-type"), "3")
    assert element.attrs[1] == (_rustlib.QualName("custom-attr"), "val")
    assert element.attrs[2] == (_rustlib.QualName("qual", "xml"), "")
