import typing

QUIRKS_MODE_OFF: int
QUIRKS_MODE_LIMITED: int
QUIRKS_MODE_FULL: int

class HtmlOptions:
    # built-in - use driver.HtmlOptions
    ...

class XmlOptions:
    # built-in - use driver.XmlOptions
    ...

class Xml:
    # built-in - use driver.Xml
    ...

class Html:
    # built-in - use driver.Html
    ...

class Node:
    # built-in - use node.Node
    ...

Namespaces = typing.Literal["", "*", "xhtml", "html", "xml", "xmlns", "xlink", "svg", "mathml"]

class QualName:
    """
    A fully qualified name (with a namespace), used to depict names of tags and attributes.

    Namespaces can be used to differentiate between similar XML fragments. For example:

    ```text
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

    ```text

    // Furniture XML
    <furn:table xmlns:furn="https://furniture.rs">
      <furn:name>African Coffee Table</furn:name>
      <furn:width>80</furn:width>
      <furn:length>120</furn:length>
    </furn:table>
    ```

    and bind the prefix `furn` to a different namespace.

    For this reason we parse names that contain a colon in the following way:

    ```text
    <furn:table>
       |    |
       |    +- local name
       |
     prefix (when resolved gives namespace_url `https://furniture.rs`)
    ```
    """

    def __init__(
        self,
        local: str,
        namespace: typing.Union[Namespaces, str] = "",
        prefix: typing.Optional[str] = None,
        /,
    ) -> None: ...
    @property
    def local(self) -> str: ...
    @property
    def namespace(self) -> str: ...
    @property
    def prefix(self) -> typing.Optional[str]: ...
    def copy(self) -> "QualName":
        """Copies the `self` and returns a new one"""
        ...

    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...

class CommentData:
    """
    A comment node data

    The comment interface represents textual notations within markup; although it is generally not
    visually shown, such comments are available to be read in the source view.

    Comments are represented in HTML and XML as content between <!-- and -->. In XML,
    like inside SVG or MathML markup, the character sequence -- cannot be used within a comment.
    """

    contents: str

    def __init__(self, contents: str, /) -> None: ...
    def copy(self) -> "CommentData":
        """Copies the `self` and returns a new one"""
        ...

    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...

class TextData:
    """
    A text node data
    """

    contents: str

    def __init__(self, contents: str, /) -> None: ...
    def copy(self) -> "TextData":
        """Copies the `self` and returns a new one"""
        ...

    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...

class ProcessingInstructionData:
    """
    A processing instruction node data

    The ProcessingInstruction interface represents a processing instruction; that is,
    a Node which embeds an instruction targeting a specific application but that can
    be ignored by any other applications which don't recognize the instruction.
    """

    data: str
    target: str

    def __init__(self, data: str, target: str, /) -> None: ...
    def copy(self) -> "ProcessingInstructionData":
        """Copies the `self` and returns a new one"""
        ...

    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...

QualNameOrStr = typing.Union[QualName, str]

class ElementDataAttributes:
    """
    An element node data attributes
    """

    def __len__(self) -> int: ...
    def __bool__(self) -> bool: ...
    def clear(self) -> None: ...
    def append(self, value: typing.Tuple[QualNameOrStr, str]) -> None: ...
    def pop(self) -> typing.Tuple[QualNameOrStr, str]: ...
    def __getitem__(self, index: int) -> typing.Tuple[QualNameOrStr, str]: ...
    def __setitem__(self, index: int, value: typing.Tuple[QualNameOrStr, str]) -> None: ...
    def __delitem__(self, index: int) -> None: ...
    def swap_remove(self, index: int) -> None: ...
    def insert(self, index: int, value: typing.Tuple[QualNameOrStr, str]) -> None: ...
    def index(self, value: typing.Tuple[QualNameOrStr, str], start: int = 0) -> None: ...
    def sort(self) -> None: ...
    def dedup(self) -> None: ...
    def __iter__(self) -> "ElementDataAttributes": ...
    def __next__(self) -> typing.Tuple[QualNameOrStr, str]: ...
    def __repr__(self) -> str: ...

class ElementData:
    """
    An element node data
    """

    def __init__(
        self,
        name: QualNameOrStr,
        attrs: typing.List[typing.Tuple[QualNameOrStr, str]],
        template: bool = False,
        mathml_annotation_xml_integration_point: bool = False,
        /,
    ) -> None: ...

    name: QualName
    template: bool
    mathml_annotation_xml_integration_point: bool

    def id(self) -> typing.Optional[str]: ...
    def classes(self) -> typing.List[str]: ...
    @property
    def attrs(self) -> ElementDataAttributes: ...
    @attrs.setter
    def set_attrs(self, value: typing.Sequence[typing.Tuple[QualNameOrStr, str]]) -> None: ...
    def copy(self) -> "ElementData":
        """Copies the `self` and returns a new one"""
        ...

    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...
