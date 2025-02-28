from .dom import TreeDom
from . import _rustlib
import typing


class Parser:
    __slots__ = ("__raw", "__state")

    def __init__(self, options: typing.Union[_rustlib.HtmlOptions, _rustlib.XmlOptions]):
        """
        An HTML/XML parser, ready to receive unicode input.

        This is very easy to use and allows you to stream input using `.process()` method; By this way
        you are don't worry about memory usages of huge inputs.

        for `options`, If your input is a HTML document, pass a `HtmlOptions`;
        If your input is a XML document, pass `XmlOptions`.
        """
        self.__raw = _rustlib.Parser(options)

        # 0 - processing
        # 1 - finished
        # 2 - converted
        self.__state = 0

    def process(self, content: typing.Union[str, bytes]) -> None:
        """
        Processes an input.

        `content` must be `str` or `bytes`.

        Raises `RuntimeError` if `.finish()` method is called.
        """
        self.__raw.process(content)

    def finish(self) -> None:
        """
        Finishes the parser and marks it as finished.

        Raises `RuntimeError` if is already finished.
        """
        self.__raw.finish()
        self.__state = 1

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
        """Returns the quirks mode (always is QUIRKS_MODE_OFF for XML)."""
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
    options: typing.Union[_rustlib.HtmlOptions, _rustlib.XmlOptions],
) -> TreeDom:
    """
    Parses your HTML (or XML depends on `options` type) content and returns the parsed document tree.
    """
    parser = Parser(options)
    parser.process(content)
    parser.finish()
    return parser.into_dom()


def parse_file(
    path: typing.Union[str, typing.TextIO, typing.BinaryIO],
    options: typing.Union[_rustlib.HtmlOptions, _rustlib.XmlOptions],
    *,
    chunk_size: int = 10240,
) -> TreeDom:
    """
    Parses your HTML (or XML depends on `options` type) file and returns the parsed document tree.
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

        parser.finish()
        return parser.into_dom()
    finally:
        if close:
            path.close()
