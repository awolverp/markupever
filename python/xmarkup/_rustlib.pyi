import typing

QUIRKS_MODE_FULL: typing.Literal[0]
QUIRKS_MODE_LIMITED: typing.Literal[1]
QUIRKS_MODE_OFF: typing.Literal[2]

class HtmlOptions:
    """
    These are options for HTML parsing.

    Note: this type is immutable.
    """

    def __new__(
        cls: typing.Type,
        full_document=...,
        exact_errors=...,
        discard_bom=...,
        profile=...,
        iframe_srcdoc=...,
        drop_doctype=...,
        quirks_mode=...,
    ) -> "HtmlOptions":
        """
        Creates a new `HtmlOptions`

        - `full_document`: Is this a complete document? (means includes html, head, and body tag). Default: true.
        - `exact_errors`: Report all parse errors described in the spec, at some performance penalty? Default: false.
        - `discard_bom`: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? Default: true.
        - `profile`: Keep a record of how long we spent in each state? Printed when `finish()` is called. Default: false.
        - `iframe_srcdoc`: Is this an `iframe srcdoc` document? Default: false.
        - `drop_doctype`: Should we drop the DOCTYPE (if any) from the tree? Default: false.
        - `quirks_mode`: Initial TreeBuilder quirks mode. Default: QUIRKS_MODE_OFF.
        """
        ...

    @property
    def full_document(self) -> bool: ...
    @property
    def exact_errors(self) -> bool: ...
    @property
    def discard_bom(self) -> bool: ...
    @property
    def profile(self) -> bool: ...
    @property
    def iframe_srcdoc(self) -> bool: ...
    @property
    def drop_doctype(self) -> bool: ...
    @property
    def quirks_mode(self) -> int: ...
    def __repr__(self) -> str: ...

class XmlOptions:
    """
    These are options for XML parsing.

    Note: this type is immutable.
    """

    def __new__(
        cls: typing.Type,
        exact_errors=...,
        discard_bom=...,
        profile=...,
    ) -> "XmlOptions":
        """
        Creates a new `XmlOptions`

        - `exact_errors`: Report all parse errors described in the spec, at some performance penalty? Default: false.
        - `discard_bom`: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? Default: true.
        - `profile`: Keep a record of how long we spent in each state? Printed when `finish()` is called. Default: false.
        """
        ...

    @property
    def exact_errors(self) -> bool: ...
    @property
    def discard_bom(self) -> bool: ...
    @property
    def profile(self) -> bool: ...
    def __repr__(self) -> str: ...
