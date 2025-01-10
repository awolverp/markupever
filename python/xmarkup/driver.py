from . import node, _rustlib
import typing


class HtmlOptions:
    __slots__ = ("_options",)

    def __init__(
        self,
        full_document: bool = True,
        *,
        exact_errors: bool = False,
        discard_bom: bool = True,
        profile: bool = False,
        iframe_srcdoc: bool = False,
        drop_doctype: bool = False,
        quirks_mode: int = _rustlib.QUIRKS_MODE_OFF,
    ) -> None:
        """
        These are options for HTML parsing:

        - full_document: Is this a complete document? (means includes html, head, and body tag)
        - exact_errors: Report all parse errors described in the spec, at some performance penalty?
        - discard_bom: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream?
        - profile: Keep a record of how long we spent in each state? (records will printed in stdout)
        - iframe_srcdoc: Is this an `iframe srcdoc` document?
        - drop_doctype: Should we drop the DOCTYPE (if any) from the tree?
        - quirks_mode: A document's quirks mode, for compatibility with old browsers.
                       See [quirks mode on wikipedia](https://en.wikipedia.org/wiki/Quirks_mode) for more information.
        """
        self._options = _rustlib.RawHtmlOptions(
            full_document=full_document,
            exact_errors=exact_errors,
            discard_bom=discard_bom,
            profile=profile,
            iframe_srcdoc=iframe_srcdoc,
            drop_doctype=drop_doctype,
            quirks_mode=quirks_mode,
        )

    @property
    def full_document(self) -> bool:
        """Is this a complete document? (means includes html, head, and body tag)"""
        return self._options.full_document

    @property
    def exact_errors(self) -> bool:
        """Report all parse errors described in the spec, at some performance penalty?"""
        return self._options.exact_errors

    @property
    def discard_bom(self) -> bool:
        """Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream?"""
        return self._options.discard_bom

    @property
    def profile(self) -> bool:
        """Keep a record of how long we spent in each state? (records will printed in stdout)"""
        return self._options.profile

    @property
    def iframe_srcdoc(self) -> bool:
        """Is this an `iframe srcdoc` document?"""
        return self._options.iframe_srcdoc

    @property
    def drop_doctype(self) -> bool:
        """Should we drop the DOCTYPE (if any) from the tree?"""
        return self._options.drop_doctype

    @property
    def quirks_mode(self) -> int:
        """document's quirks mode, for compatibility with old browsers."""
        return self._options.quirks_mode


class XmlOptions:
    __slots__ = ("_options",)

    def __init__(
        self,
        *,
        exact_errors: bool = False,
        discard_bom: bool = True,
        profile: bool = False,
    ) -> None:
        """
        These are options for XML parsing:

        - exact_errors: Report all parse errors described in the spec, at some performance penalty?
        - discard_bom: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream?
        - profile: Keep a record of how long we spent in each state? (records will printed in stdout)
        """
        self._options = _rustlib.RawXmlOptions(
            exact_errors=exact_errors,
            discard_bom=discard_bom,
            profile=profile,
        )

    @property
    def exact_errors(self) -> bool:
        """Report all parse errors described in the spec, at some performance penalty?"""
        return self._options.exact_errors

    @property
    def discard_bom(self) -> bool:
        """Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream?"""
        return self._options.discard_bom

    @property
    def profile(self) -> bool:
        """Keep a record of how long we spent in each state? (records will printed in stdout)"""
        return self._options.profile


class Html:
    __slots__ = ("_parser",)

    def __init__(
        self,
        content: typing.Union[str, bytes],
        options: typing.Union[HtmlOptions, _rustlib.RawHtmlOptions] = HtmlOptions(),
    ) -> None:
        """
        Parses a HTML document into a tree of `Node`s.

        - content: HTML document content as bytes or str.
        - options: parsing options.

        Example::

            html = Html("... HTML ...", HtmlOptions(exact_errors=True))
            print(html.root) # Node(DocumentData)
        """
        if isinstance(options, HtmlOptions):
            options = options._options

        self._parser = _rustlib.RawHtml(content, options)

    @property
    def errors(self) -> typing.List[str]:
        """Parse errors list"""
        return self._parser.errors

    @property
    def quirks_mode(self) -> int:
        """document's quirks mode, for compatibility with old browsers."""
        return self._parser.quirks_mode

    @property
    def root(self) -> node.Node:
        """the root node of the parsed document. always is a `node.DocumentData`."""
        return node.Node(self._parser.root)

    def serialize(self) -> bytes:
        """Shorthand for `self.root.serialize_html()`"""
        return self.root.serialize_html()


class Xml:
    __slots__ = ("_parser",)

    def __init__(
        self,
        content: typing.Union[str, bytes],
        options: typing.Union[XmlOptions, _rustlib.RawXmlOptions] = XmlOptions(),
    ) -> None:
        """
        Parses a XML document into a tree of `Node`s.

        - content: XML document content as bytes or str.
        - options: parsing options.

        Example::

            xml = Xml("... XML ...", XmlOptions(exact_errors=True))
            print(xml.root) # Node(DocumentData)
        """
        if isinstance(options, XmlOptions):
            options = options._options

        self._parser = _rustlib.RawXml(content, options)

    @property
    def errors(self) -> typing.List[str]:
        """Parse errors list"""
        return self._parser.errors

    @property
    def root(self) -> node.Node:
        """the root node of the parsed document. always is a `node.DocumentData`."""
        return node.Node(self._parser.root)

    def serialize(self) -> bytes:
        """Shorthand for `self.root.serialize_xml()`"""
        return self.root.serialize_xml()
