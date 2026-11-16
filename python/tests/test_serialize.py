import markupever
from markupever import dom


def test_pretty_print_keeps_a_newline_before_the_outer_close_tag():
    outer = list(markupever.parse("<div id=o><div id=i></div></div>").select("#o"))[0]

    assert outer.serialize(indent=2) == '<div id="o">\n  <div id="i"></div>\n</div>'


def test_doctype_with_public_and_system_ids_omits_the_system_keyword():
    serialized = (
        markupever.parse(
            '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd"><p>x'
        )
        .root()
        .serialize()
    )

    assert serialized.startswith(
        '<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd">'
    )


def test_system_only_doctype_keeps_the_system_keyword():
    serialized = markupever.parse('<!DOCTYPE html SYSTEM "sys.dtd"><p>').root().serialize()

    assert serialized.startswith('<!DOCTYPE html SYSTEM "sys.dtd">')


def test_doctype_quotes_are_escaped():
    document = dom.TreeDom()
    root = document.root()
    doctype = root.create_doctype('html"><script>', 'a"b', "c")

    assert doctype.name == 'html"><script>'
    assert root.serialize().startswith(
        '<!DOCTYPE html&quot;&gt;&lt;script&gt; PUBLIC "a&quot;b" "c">'
    )
