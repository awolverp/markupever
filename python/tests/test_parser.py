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


def test_parse_function():
    assert isinstance(
        markupever.parse("<html></html>", markupever.XmlOptions()), markupever.dom.TreeDom
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
        markupever.parse_file(str(file), markupever.HtmlOptions()), markupever.dom.TreeDom
    )

    markupever.parse_file(file, markupever.HtmlOptions())
