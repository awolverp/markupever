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

class QualName:
    """
    A fully qualified name (with a namespace), used to depict names of tags and attributes.

    Namespaces can be used to differentiate between similar XML fragments. For example:

    ```
    // HTML
    <table>
      <tr>
        <td>Apples</td>
        <td>Bananas</td>
      </tr>
    </table>

    // Furniture XML
    <table>
      <name>African Coffee Table</name>
      <width>80</width>
      <length>120</length>
    </table>
    ```

    Without XML namespaces, we can't use those two fragments in the same document
    at the same time. However if we declare a namespace we could instead say:

    ```
    // Furniture XML
    <furn:table xmlns:furn="https://furniture.rs">
      <furn:name>African Coffee Table</furn:name>
      <furn:width>80</furn:width>
      <furn:length>120</furn:length>
    </furn:table>
    ```

    and bind the prefix `furn` to a different namespace.

    For this reason we parse names that contain a colon in the following way:

    ```
    <furn:table>
       |    |
       |    +- local name
       |
     prefix (when resolved gives namespace_url `https://furniture.rs`)
    ```

    Note: This type is immutable.
    """

    def __new__(
        cls,
        local: str,
        ns: typing.Union[
            str, typing.Literal["html", "xml", "xhtml", "xmlns", "xlink", "svg", "mathml", "*"]
        ] = ...,
        prefix: typing.Optional[str] = ...,
    ): ...
    @property
    def local(self) -> str:
        """The local name (e.g. `table` in `<furn:table>` above)."""
        ...
    @property
    def ns(self) -> str:
        """The namespace after resolution (e.g. https://furniture.rs in example above)."""
        ...
    @property
    def prefix(self) -> typing.Optional[str]:
        """
        The prefix of qualified (e.g. furn in <furn:table> above).
        Optional (since some namespaces can be empty or inferred),
        and only useful for namespace resolution (since different prefix can still resolve to same namespace)
        """
        ...

    def copy(self) -> "QualName": ...
    def __eq__(self, value) -> bool: ...
    def __ne__(self, value) -> bool: ...
    def __gt__(self, value) -> bool: ...
    def __ge__(self, value) -> bool: ...
    def __lt__(self, value) -> bool: ...
    def __le__(self, value) -> bool: ...
    def __hash__(self) -> int: ...
    def __repr__(self) -> str: ...

class AttrsListItems:
    def __iter__(self) -> "AttrsListItems": ...
    def __next__(self) -> typing.Tuple[QualName, str]: ...
