import typing

from . import iter as iter

__all__ = [
    "QUIRKS_MODE_FULL",
    "QUIRKS_MODE_LIMITED",
    "QUIRKS_MODE_OFF",
    "AttrsList",
    "AttrsListItems",
    "Comment",
    "Doctype",
    "Document",
    "Element",
    "HtmlOptions",
    "Parser",
    "ProcessingInstruction",
    "QualName",
    "Select",
    "Text",
    "TreeDom",
    "XmlOptions",
    "__author__",
    "__version__",
    "_is_node_impl",
    "iter",
    "serialize",
]

__version__: str
__author__: str

QUIRKS_MODE_FULL: typing.Literal[0]
QUIRKS_MODE_LIMITED: typing.Literal[1]
QUIRKS_MODE_OFF: typing.Literal[2]

@typing.final
class HtmlOptions:
    """
    These are options for HTML parsing.

    Note: this type is immutable.
    """

    def __new__(
        cls,
        full_document: bool | None = ...,
        exact_errors: bool = ...,
        discard_bom: bool = ...,
        profile: bool = ...,
        iframe_srcdoc: bool = ...,
        drop_doctype: bool = ...,
        quirks_mode: int = ...,
        fragment_context: QualName | str | None = ...,
    ) -> HtmlOptions:
        """
        Creates a new `HtmlOptions`

        - `full_document`: Is this a complete document? (means includes html, head, and body tag). Default: true,
          or false if `fragment_context` is given.
        - `exact_errors`: Report all parse errors described in the spec, at some performance penalty? Default: false.
        - `discard_bom`: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? Default: true.
        - `profile`: Keep a record of how long we spent in each state? Printed when `finish()` is called. Default: false.
        - `iframe_srcdoc`: Is this an `iframe srcdoc` document? Default: false.
        - `drop_doctype`: Should we drop the DOCTYPE (if any) from the tree? Default: false.
        - `quirks_mode`: Initial TreeBuilder quirks mode. Default: QUIRKS_MODE_OFF.
        - `fragment_context`: Parse a fragment as if it were the contents of this context element, e.g. `"td"` or
          `QualName("path", "svg")`. A name without a namespace is an HTML element. Default: None (fragments are
          parsed in a `body` context).
        """

    @property
    def full_document(self) -> bool: ...
    @property
    def fragment_context(self) -> QualName | None: ...
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

@typing.final
class XmlOptions:
    """
    These are options for XML parsing.

    Note: this type is immutable.
    """

    def __new__(
        cls,
        exact_errors: bool = ...,
        discard_bom: bool = ...,
        profile: bool = ...,
    ) -> XmlOptions:
        """
        Creates a new `XmlOptions`

        - `exact_errors`: Report all parse errors described in the spec, at some performance penalty? Default: false.
        - `discard_bom`: Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? Default: true.
        - `profile`: Keep a record of how long we spent in each state? Printed when `finish()` is called. Default: false.
        """

    @property
    def exact_errors(self) -> bool: ...
    @property
    def discard_bom(self) -> bool: ...
    @property
    def profile(self) -> bool: ...

@typing.final
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
        ns: str
        | typing.Literal[
            "html", "xml", "xhtml", "xmlns", "xlink", "svg", "mathml", "*"
        ] = ...,
        prefix: str | None = ...,
    ) -> QualName: ...
    @property
    def local(self) -> str:
        """The local name (e.g. `table` in `<furn:table>` above)."""
    @property
    def ns(self) -> str:
        """The namespace after resolution (e.g. https://furniture.rs in example above)."""
    @property
    def prefix(self) -> str | None:
        """
        The prefix of qualified (e.g. furn in <furn:table> above).
        Optional (since some namespaces can be empty or inferred),
        and only useful for namespace resolution (since different prefix can still resolve to same namespace)
        """

    def copy(self) -> QualName:
        """
        Create a copy of the current QualName instance.

        Returns a new QualName instance with the same local name, namespace, and prefix.
        """

    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    def __gt__(self, value: QualName, /) -> bool: ...
    def __ge__(self, value: QualName, /) -> bool: ...
    def __lt__(self, value: QualName, /) -> bool: ...
    def __le__(self, value: QualName, /) -> bool: ...
    def __hash__(self) -> int: ...

_QualNameOrStr: typing.TypeAlias = QualName | str
_Node: typing.TypeAlias = (
    Document | Doctype | Comment | Text | Element | ProcessingInstruction
)

@typing.final
class TreeDom:
    """A tree of nodes, which owns all of its nodes."""

    def __new__(cls, *, namespaces: dict[str, str] | None = ...) -> TreeDom: ...
    @classmethod
    def with_capacity(
        cls, capacity: int, *, namespaces: dict[str, str] | None = ...
    ) -> TreeDom: ...
    def namespaces(self) -> dict[str, str]: ...
    def root(self) -> Document: ...
    def append(self, parent: _Node, child: _Node) -> None: ...
    def prepend(self, parent: _Node, child: _Node) -> None: ...
    def insert_before(self, sibling: _Node, new_sibling: _Node) -> None: ...
    def insert_after(self, sibling: _Node, new_sibling: _Node) -> None: ...
    def detach(self, node: _Node) -> None: ...
    def reparent_append(self, parent: _Node, child: _Node) -> None: ...
    def reparent_prepend(self, parent: _Node, child: _Node) -> None: ...
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    __hash__: typing.ClassVar[None]  # type: ignore[assignment]
    def __len__(self) -> int: ...

class _NodeMethods:
    """Methods that all node types have (not a real class)."""

    def tree(self) -> TreeDom: ...
    def parent(self) -> _Node | None: ...
    def prev_sibling(self) -> _Node | None: ...
    def next_sibling(self) -> _Node | None: ...
    def first_child(self) -> _Node | None: ...
    def last_child(self) -> _Node | None: ...
    def has_children(self) -> bool: ...
    def has_siblings(self) -> bool: ...
    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...
    __hash__: typing.ClassVar[None]  # type: ignore[assignment]

@typing.final
class Document(_NodeMethods):
    """A document node, the root of a tree. It has no constructor."""

    def __new__(cls, treedom: object) -> typing.NoReturn: ...

@typing.final
class Doctype(_NodeMethods):
    def __new__(
        cls, treedom: TreeDom, name: str, public_id: str, system_id: str
    ) -> Doctype: ...
    name: str
    public_id: str
    system_id: str

@typing.final
class Comment(_NodeMethods):
    def __new__(cls, treedom: TreeDom, content: str) -> Comment: ...
    content: str

@typing.final
class Text(_NodeMethods):
    def __new__(cls, treedom: TreeDom, content: str) -> Text: ...
    content: str

@typing.final
class Element(_NodeMethods):
    def __new__(
        cls,
        treedom: TreeDom,
        name: _QualNameOrStr,
        attrs: typing.Iterable[tuple[_QualNameOrStr, str]],
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> Element: ...
    @property
    def name(self) -> QualName: ...
    @name.setter
    def name(self, value: _QualNameOrStr) -> None: ...
    @property
    def attrs(self) -> AttrsList: ...
    @attrs.setter
    def attrs(self, value: typing.Iterable[tuple[_QualNameOrStr, str]]) -> None: ...
    template: bool
    mathml_annotation_xml_integration_point: bool
    def id(self) -> str | None: ...
    def class_list(self) -> list[str]: ...

@typing.final
class ProcessingInstruction(_NodeMethods):
    def __new__(
        cls, treedom: TreeDom, data: str, target: str
    ) -> ProcessingInstruction: ...
    data: str
    target: str

@typing.final
class AttrsList:
    """The attributes of an element. It has no constructor."""

    def __new__(cls, *args: object, **kwargs: object) -> typing.NoReturn: ...
    def clear(self) -> None: ...
    def insert(self, index: int, key: _QualNameOrStr, value: str) -> None: ...
    def push(self, key: _QualNameOrStr, value: str) -> None: ...
    def items(self) -> AttrsListItems: ...
    def update_item(self, index: int, key: _QualNameOrStr, value: str) -> None: ...
    def update_value(self, index: int, value: str) -> None: ...
    def get_by_index(self, index: int) -> tuple[QualName, str]: ...
    def remove(self, index: int) -> tuple[QualName, str]: ...
    def swap_remove(self, index: int) -> tuple[QualName, str]: ...
    def dedup(self) -> None: ...
    def reverse(self) -> None: ...
    def __len__(self) -> int: ...

@typing.final
class AttrsListItems:
    def __new__(cls, *args: object, **kwargs: object) -> typing.NoReturn: ...
    def __iter__(self) -> AttrsListItems: ...
    def __next__(self) -> tuple[QualName, str]: ...
    def __len__(self) -> int: ...

@typing.final
class Parser:
    def __new__(cls, options: HtmlOptions | XmlOptions) -> Parser: ...
    def process(self, content: str | bytes) -> None: ...
    def finish(self) -> None: ...
    def into_dom(self) -> TreeDom: ...
    def errors(self) -> list[str]: ...
    def quirks_mode(self) -> int: ...
    def lineno(self) -> int: ...

@typing.final
class Select:
    """An iterator over the elements matching a CSS selector, in document order."""

    def __new__(cls, node: _Node, expression: str) -> Select: ...
    def __iter__(self) -> Select: ...
    def __next__(self) -> Element: ...

def serialize(
    node: _Node, indent: int = ..., include_self: bool = ..., is_html: bool | None = ...
) -> bytes: ...
def _is_node_impl(object: object) -> bool: ...
