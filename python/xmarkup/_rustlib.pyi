from __future__ import annotations
import typing

__version__: str
__author__: str

QUIRKS_MODE_OFF: int
QUIRKS_MODE_LIMITED: int
QUIRKS_MODE_FULL: int

class RawHtmlOptions:
    # built-in - use driver.HtmlOptions
    ...

class RawXmlOptions:
    # built-in - use driver.XmlOptions
    ...

class RawXml:
    # built-in - use driver.Xml
    ...

class RawHtml:
    # built-in - use driver.Html
    ...

class RawNode:
    # built-in - use nodes.Node
    ...

class RawChildren:
    # built-in - use nodes.Children
    ...

class RawTree:
    # built-in - use nodes.TreeIterator
    ...

class RawParents:
    # built-in - use nodes.ParentsIterator
    ...

class RawMatching:
    # built-in - use nodes.Matching
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

class DocumentData:
    """
    A document node of DOM.

    Document is the root node of a DOM.
    """

    def __init__(self) -> None: ...
    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...
    def copy(self) -> "DocumentData":
        """Copies the `self` and returns a new one"""
        ...

class DoctypeData:
    """
    A doctype node data

    the doctype is the required <!doctype html> preamble found at the top of all documents.
    Its sole purpose is to prevent a browser from switching into so-called "quirks mode"
    when rendering a document; that is, the <!doctype html> doctype ensures that the browser makes
    a best-effort attempt at following the relevant specifications, rather than using a different
    rendering mode that is incompatible with some specifications.
    """

    name: str
    public_id: str
    system_id: str

    def __init__(self, name: str, public_id: str, system_id: str, /) -> None: ...
    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...
    def copy(self) -> "DoctypeData":
        """Copies the `self` and returns a new one"""
        ...

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
    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...
    def copy(self) -> "CommentData":
        """Copies the `self` and returns a new one"""
        ...

class TextData:
    """
    A text node data
    """

    contents: str

    def __init__(self, contents: str, /) -> None: ...
    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...
    def copy(self) -> "TextData":
        """Copies the `self` and returns a new one"""
        ...

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
    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...
    def copy(self) -> "ProcessingInstructionData":
        """Copies the `self` and returns a new one"""
        ...

QualNameOrStr = typing.Union[QualName, str]

class ElementDataAttributes:
    """
    An element node data attributes
    """

    def __len__(self) -> int:
        """Returns `len(self)` - length of the attributes vector."""
        ...
    def __bool__(self) -> bool:
        """Returns `bool(self)` - `true` if the vector is not empty"""
        ...

    def clear(self) -> None:
        """Clears the attributes vector"""
        ...

    def append(self, value: typing.Tuple[QualNameOrStr, str]) -> None:
        """Append a new `(QualName, str)` sequence to the vector"""
        ...

    def pop(self) -> typing.Tuple[QualNameOrStr, str]:
        """
        Removes an item from the end of the vector and returns it.

        Raises IndexError if the vector is empty
        """
        ...

    def __getitem__(self, index: int) -> typing.Tuple[QualNameOrStr, str]:
        """Returns `self[index]`"""
        ...
    def __setitem__(self, index: int, value: typing.Tuple[QualNameOrStr, str]) -> None:
        """Performs `self[index] = (QualName, str)`"""
        ...
    def __delitem__(self, index: int) -> None:
        """Performs `del self[index]`"""
        ...
    def swap_remove(self, index: int) -> None:
        """
        Performs del self[index] but is O(1), because does not reorder the vector,
        and replace self[index] with last element.

        If the order is not important for you, use this method instead of del self[index]
        """
        ...
    def insert(self, index: int, value: typing.Tuple[QualNameOrStr, str]) -> None:
        """
        Insert a (QualName, str) at position index, shifting all elements after it to the right.

        Raises IndexError if index > len
        """
        ...
    def index(self, value: typing.Tuple[QualNameOrStr, str], start: int = 0) -> None:
        """
        Return first index of value.

        Raises ValueError if the value is not present.
        """
        ...
    def sort(self) -> None:
        """
        Sorts the slice with a comparison function, without preserving the initial order of equal elements.

        This sort is unstable (i.e., may reorder equal elements), in-place (i.e., does not allocate),
        and O(n * log(n)) worst-case.
        """
        ...
    def dedup(self) -> None:
        """
        Removes consecutive duplicate elements.
        """
        ...
    def __iter__(self) -> "ElementDataAttributes":
        """
        Returns iter(self)

        Note that you cannot have multiple iter(self) in a same time. each one must be done
        before creating next one.
        """
        ...
    def __next__(self) -> typing.Tuple[QualNameOrStr, str]:
        """Returns `next(self)`"""
        ...
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

    def id(self) -> typing.Optional[str]:
        """Finds, caches, and returns the 'id' attribute from attributes."""
        ...
    def classes(self) -> typing.List[str]:
        """Finds, caches, and returns the 'class' attributes as list from attributes."""
        ...
    @property
    def attrs(self) -> ElementDataAttributes: ...
    @attrs.setter
    def set_attrs(self, value: typing.Sequence[typing.Tuple[QualNameOrStr, str]]) -> None: ...
    def copy(self) -> "ElementData":
        """Copies the `self` and returns a new one"""
        ...

    def __eq__(self, other) -> bool: ...
    def __repr__(self) -> str: ...
