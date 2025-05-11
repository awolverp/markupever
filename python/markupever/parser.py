from .dom import TreeDom
from . import _rustlib
import typing


class Parser:
    __slots__ = ("__raw", "__state")

    def __init__(self, options: typing.Union[_rustlib.HtmlOptions, _rustlib.XmlOptions, typing.Literal["html"], typing.Literal["xml"]] = "html"):
        """
        An HTML/XML parser, ready to receive unicode input.

        This is very easy to use and allows you to stream input using `.process()` method; By this way
        you are don't worry about memory usages of huge inputs.

        for `options`, If your input is a HTML document, pass a `HtmlOptions`;
        If your input is a XML document, pass `XmlOptions`.
        """
        if isinstance(options, str):
            if options == "html":
                options = _rustlib.HtmlOptions()
            elif options == "xml":
                options = _rustlib.XmlOptions()
            else:
                raise ValueError(f"invalid parser options: {options!r}")
        
        self.__raw = _rustlib.Parser(options)

        # 0 - processing
        # 1 - finished
        # 2 - converted
        self.__state = 0

    def writable(self) -> bool:
        """
        Same as `Parser.is_finished`.

        This function exists to make `Parser` like a `BytesIO` and `StringIO`.
        You can pass the `Parser` to each function which needs a writable buffer or IO.
        """
        return self.is_finished

    def write(self, content: typing.Union[str, bytes]) -> int:
        """
        Same as `Parser.process`.

        This function exists to make `Parser` like a `BytesIO` and `StringIO`.
        You can pass the `Parser` to each function which needs a writable buffer or IO.
        """
        self.__raw.process(content)
        return len(content)

    def process(self, content: typing.Union[str, bytes]) -> "Parser":
        """
        Processes an input.

        `content` must be `str` or `bytes`.

        Raises `RuntimeError` if `.finish()` method is called.
        """
        self.__raw.process(content)
        return self

    def finish(self) -> "Parser":
        """
        Finishes the parser and marks it as finished.

        Raises `RuntimeError` if is already finished.
        """
        self.__raw.finish()
        self.__state = 1
        return self

    def into_dom(self) -> TreeDom:
        """Converts the self into `TreeDom`. after calling this method, this object is unusable and you cannot use it."""
        dom = TreeDom(raw=self.__raw.into_dom())
        self.__state = 2
        return dom

    def errors(self) -> typing.List[str]:
        """
        Returns the errors which are detected while parsing.
        """
        return self.__raw.errors()

    @property
    def quirks_mode(self) -> int:
        """
        Returns the quirks mode (always is QUIRKS_MODE_OFF for XML).

        See quirks mode on [wikipedia](https://en.wikipedia.org/wiki/Quirks_mode) for more information.
        """
        return self.__raw.quirks_mode()

    @property
    def lineno(self) -> int:
        """Returns the line count of the parsed content (always is `1` for XML)."""
        return self.__raw.lineno()

    @property
    def is_finished(self) -> bool:
        """Returns `True` if the parser is marked as finished"""
        return self.__state != 0

    @property
    def is_converted(self) -> bool:
        """Returns `True` if the parser is converted to `TreeDom` and now is unusable."""
        return self.__state == 2

    def __repr__(self) -> str:
        return repr(self.__raw)


def parse(
    content: typing.Union[str, bytes],
    options: typing.Union[_rustlib.HtmlOptions, _rustlib.XmlOptions, typing.Literal["html"], typing.Literal["xml"]] = "html",
) -> TreeDom:
    """
    Parses HTML or XML content and returns the parsed document tree.

    Args:
        content: The HTML or XML content to parse, either as a string or bytes.
        options: Parsing options that specify whether to parse as HTML or XML.

    Returns:
        A TreeDom object representing the parsed document tree.
    """
    parser = Parser(options)
    parser.process(content)
    return parser.finish().into_dom()


def parse_file(
    path: typing.Union[str, typing.TextIO, typing.BinaryIO],
    options: typing.Union[_rustlib.HtmlOptions, _rustlib.XmlOptions, typing.Literal["html"], typing.Literal["xml"]] = "html",
    *,
    chunk_size: int = 10240,
) -> TreeDom:
    """
    Parses an HTML or XML file and returns the parsed document tree.

    Args:
        path: A file path, file-like object, or Path object to be parsed.
        options: HTML or XML parsing options that control the parsing behavior.
        chunk_size: Size of chunks to read from the file during parsing (default is 10240 bytes).

    Returns:
        A TreeDom object representing the parsed document tree.

    The function supports parsing files of different types (string paths, Path objects,
    file-like objects) and handles file opening and closing automatically.
    """
    from pathlib import Path

    close = False

    if isinstance(path, Path):
        path = str(path)

    if isinstance(path, str):
        path = open(path, "rb")
        close = True

    try:
        parser = Parser(options)

        while True:
            content = path.read(chunk_size)
            if not content:
                break

            parser.process(content)

        return parser.finish().into_dom()
    finally:
        if close:
            path.close()
