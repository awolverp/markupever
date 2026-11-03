import markupever
import pytest


def test_parser():  # this is a copy of test_rustlib.test_parser for markupever.parser.Parser
    parser = markupever.Parser(markupever.HtmlOptions())
    parser.process(b"<html><p>Ali</p></html>")
    parser.finish()

    repr(parser)

    parser = markupever.Parser(markupever.HtmlOptions())
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

    parser = markupever.Parser(markupever.XmlOptions())
    for c in ("<html>", b"Ali", b"</html>"):
        parser.process(c)
    parser.finish()

    assert parser.is_finished
    assert isinstance(parser.errors(), list)
    assert parser.lineno == 1
    assert parser.quirks_mode == 2

    parser = markupever.Parser(markupever.HtmlOptions(full_document=False))
    for c in (b"<html><p>Ali</p>", "\n", "</html>"):
        parser.process(c)
    parser.finish()

    assert parser.lineno == 2

    _ = parser.into_dom()

    assert parser.is_converted

    with pytest.raises(RuntimeError):
        parser.errors()

    _ = markupever.Parser("html")
    _ = markupever.Parser("xml")

    with pytest.raises(ValueError):
        _ = markupever.Parser("invalid")


def test_parse_function():
    assert isinstance(
        markupever.parse("<html></html>", markupever.XmlOptions()),
        markupever.dom.TreeDom,
    )


def test_parse_file_function(tmp_path):
    import io

    file = io.BytesIO(b"<body></body>")
    assert isinstance(markupever.parse_file(file, markupever.XmlOptions()), markupever.dom.TreeDom)
    assert not file.closed
    file.close()

    file = io.StringIO("<body></body>")
    assert isinstance(markupever.parse_file(file, markupever.XmlOptions()), markupever.dom.TreeDom)
    assert not file.closed
    file.close()

    file = tmp_path / "file.html"

    with pytest.raises(FileNotFoundError):
        markupever.parse_file(str(file), markupever.HtmlOptions())

    file.write_bytes(b"<body></body>")
    assert isinstance(
        markupever.parse_file(str(file), markupever.HtmlOptions()),
        markupever.dom.TreeDom,
    )

    markupever.parse_file(file, markupever.HtmlOptions())


@pytest.mark.parametrize("kwargs", [{}, {"full_document": False}, {"fragment_context": "div"}])
def test_scripting_enabled_noscript(kwargs):
    html = "<body><noscript><p>Code <strong>1234</strong> &amp; more</p></noscript></body>"
    default = markupever.parse(html, markupever.HtmlOptions(**kwargs))
    assert default.select_one("noscript strong") is None

    enabled = markupever.HtmlOptions(scripting_enabled=True, **kwargs)
    assert enabled.scripting_enabled
    assert markupever.parse(html, enabled).serialize() == default.serialize()

    disabled = markupever.HtmlOptions(scripting_enabled=False, **kwargs)
    assert not disabled.scripting_enabled
    dom = markupever.parse(html, disabled)
    assert dom.select_one("noscript p").text() == "Code 1234 & more"
    assert dom.select_one("noscript strong").text() == "1234"


@pytest.mark.parametrize("scripting_enabled", [True, False])
def test_scripting_enabled_noscript_fragment(scripting_enabled):
    options = markupever.HtmlOptions(
        fragment_context="noscript", scripting_enabled=scripting_enabled
    )
    dom = markupever.parse("<strong>123 &amp; more</strong>", options)
    if scripting_enabled:
        assert dom.select_one("strong") is None
        assert dom.root().text() == "<strong>123 &amp; more</strong>"
    else:
        assert dom.select_one("strong").text() == "123 & more"


@pytest.mark.parametrize("scripting_enabled", [True, False])
def test_scripting_enabled_noscript_in_head(scripting_enabled):
    html = '<head><noscript><link rel="stylesheet" href="/style.css"></noscript></head>'
    dom = markupever.parse(html, markupever.HtmlOptions(scripting_enabled=scripting_enabled))
    assert (dom.select_one("head noscript link") is not None) is (not scripting_enabled)


def test_scripting_enabled_default():
    options = markupever.HtmlOptions(
        None, False, True, False, False, False, markupever.QUIRKS_MODE_OFF, None
    )
    assert options.scripting_enabled
    assert "scripting_enabled=true" in repr(options)


def test_fragment_context():
    options = markupever.HtmlOptions(fragment_context="tbody")
    assert not options.full_document
    assert options.fragment_context == markupever.dom.QualName("tbody", "html")
    assert "fragment_context=" in repr(options)

    # In a tbody context, table rows are kept rather than dropped, and the parsed nodes are the
    # children of the root
    dom = markupever.parse("<tr><td>x", options)
    (tr,) = dom.root().children()
    assert tr.name.local == "tr"
    (td,) = tr.children()
    assert td.name.local == "td"
    assert dom.serialize(indent=0) == "<tr><td>x</td></tr>"

    # In an svg context, elements are created in the SVG namespace
    options = markupever.HtmlOptions(fragment_context=markupever.dom.QualName("svg", "svg"))
    (path,) = markupever.parse("<path/>", options).root().children()
    assert path.name == markupever.dom.QualName("path", "svg")

    # Without a context, fragments are parsed in a body context
    options = markupever.HtmlOptions(full_document=False)
    assert options.fragment_context is None
    dom = markupever.parse("<tr><td>x<p>y", options)
    assert dom.serialize(indent=0) == "x<p>y</p>"

    assert markupever.HtmlOptions().full_document
    with pytest.raises(ValueError):
        markupever.HtmlOptions(full_document=True, fragment_context="td")


def test_meta_content_ending_in_charset():
    # html5ever 0.39 panicked extracting an encoding from this content attribute.
    for content in ("charset", "text/html; charset", "charset  "):
        dom = markupever.parse(f'<meta http-equiv="Content-Type" content="{content}"><p>x</p>')
        assert dom.select_one("p").text() == "x"


def test_fragment_context_serialize():
    # A fragment is serialized as the children of its context element, so that it round-trips.
    src = 'console.log("<br>") &amp;'
    for context, text, html in [
        ("script", src, src),
        ("style", src, src),
        ("xmp", src, src),
        ("plaintext", src, src),
        ("textarea", 'console.log("<br>") &', 'console.log("&lt;br&gt;") &amp;'),
        ("title", 'console.log("<br>") &', 'console.log("&lt;br&gt;") &amp;'),
    ]:
        options = markupever.HtmlOptions(fragment_context=context)
        dom = markupever.parse(src, options)
        assert dom.root().text() == text
        assert dom.serialize() == html
        assert dom.root().serialize(include_self=False) == html
        assert markupever.parse(dom.serialize(), options).root().text() == text

    # Text in other contexts is escaped.
    dom = markupever.parse("a &lt; b", markupever.HtmlOptions(full_document=False))
    assert dom.serialize() == "a &lt; b"

    # Descendants of the fragment's nodes are serialized as usual.
    dom = markupever.parse("<p>a &lt; b</p>", markupever.HtmlOptions(fragment_context="div"))
    assert dom.serialize() == "<p>a &lt; b</p>"
