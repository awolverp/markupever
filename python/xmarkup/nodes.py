from . import _rustlib
from ._rustlib import (
    DocumentData,
    DoctypeData,
    TextData,
    CommentData,
    ElementData,
    ProcessingInstructionData,
    RawNode,
    QualName as QualName,
)
import typing


NodeData = typing.Union[
    DocumentData,
    DoctypeData,
    TextData,
    CommentData,
    ElementData,
    ProcessingInstructionData,
]


class Children:
    __slots__ = ("_children",)

    def __init__(
        self,
        node: typing.Union["Node", RawNode],
    ) -> None:
        """
        Children vector of a node.
        """
        if isinstance(node, Node):
            node = node._node

        self._children: _rustlib.RawChildren = node.children()

    def __len__(self) -> int:
        """Returns `len(self)` - length of the vector."""
        return self._children.__len__()

    def __bool__(self) -> bool:
        """Returns `bool(self)` - `true` if the vector is not empty"""
        return self._children.__bool__()

    def clear(self) -> None:
        """Clears the attributes vector"""
        self._children.clear()

    def append(self, child: typing.Union["Node", RawNode, NodeData]) -> None:
        """
        Append a new child into node and sets its new parent

        Returns error if the child has parent for itself. Also returns error if child cycle be detected.
        """
        if isinstance(child, Node):
            child = child._node

        self._children.append(child)

    def pop(self) -> "Node":
        """Pop a child from node and removes its parent"""
        return Node(self._children.pop())

    def __getitem__(self, index: int) -> "Node":
        """Returns `self[index]`"""
        return Node(self._children[index])

    def __setitem__(self, index: int, child: typing.Union["Node", RawNode, NodeData]) -> None:
        """Performs `self[index] = Node`"""
        if isinstance(child, Node):
            child = child._node

        self._children[index] = child

    def __delitem__(self, index: int) -> None:
        """Performs `del self[index]`"""
        del self._children[index]

    def index(self, child: typing.Union["Node", RawNode, NodeData]) -> int:
        """
        Return first index of value.

        Raises ValueError if the value is not present.
        """
        if isinstance(child, Node):
            child = child._node

        return self._children.index(child)

    def insert(self, index: int, child: typing.Union["Node", RawNode, NodeData]) -> None:
        """
        Inserts a child at position index.

        Returns error if the child has parent for itself. Also returns error if child cycle be detected.
        """
        if isinstance(child, Node):
            child = child._node

        self._children.insert(index, child)

    def __iter__(self) -> "Children":
        """
        Returns `iter(self)`

        Note that you cannot have multiple iter(self) in a same time. each one must be done before creating next one.
        """
        self._children = iter(self._children)
        return self

    def __next__(self) -> "Node":
        """Returns `next(self)`"""
        return Node(next(self._children))


class TreeIterator:
    __slots__ = ("_tree",)

    def __init__(
        self,
        node: typing.Union["Node", RawNode],
        include_self: bool = True,
    ) -> None:
        """
        Iterates all children and also their children like a tree.
        """
        if isinstance(node, Node):
            node = node._node

        self._tree: _rustlib.RawTree = node.tree(include_self=include_self)

    def __iter__(self) -> "TreeIterator":
        """
        Returns `iter(self)`.
        """
        self._tree = iter(self._tree)
        return self

    def __next__(self) -> "Node":
        """
        Returns `next(self)`.
        """
        return Node(next(self._tree))


class ParentsIterator:
    __slots__ = ("_parents",)

    def __init__(
        self,
        node: typing.Union["Node", RawNode],
        include_self: bool = False,
    ) -> None:
        if isinstance(node, Node):
            node = node._node

        self._parents: _rustlib.RawParents = node.parents(include_self=include_self)

    def __iter__(self) -> "ParentsIterator":
        self._parents = iter(self._parents)
        return self

    def __next__(self) -> "Node":
        return Node(next(self._parents))


class Matching:
    __slots__ = ("_raw_matching",)

    def __init__(
        self, node: typing.Union["Node", RawNode, NodeData], expr: str, *, parser: typing.Any = None
    ) -> None:
        from .driver import Html, Xml

        if isinstance(node, Node):
            node = node._node

        if isinstance(parser, (Html, Xml)):
            parser = parser._parser

        self._raw_matching = _rustlib.RawMatching(node, expr, parser)

    def __iter__(self) -> "Matching":
        self._raw_matching = iter(self._raw_matching)
        return self

    def __next__(self) -> "Node":
        return Node(next(self._raw_matching))


class Node:
    __slots__ = ("_node",)

    def __init__(
        self,
        data: typing.Union["Node", RawNode, NodeData],
    ) -> None:
        """
        A node of DOM.

        - data: a node or node data.
        """
        if isinstance(data, Node):
            self._node = data._node

        elif isinstance(data, RawNode):
            self._node = data

        else:
            self._node = RawNode(data)

    def data(self) -> NodeData:
        """Returns the node data as `*Data` classes"""
        return self._node.data()

    def is_document(self) -> bool:
        """Returns `True` if the node is a document"""
        return self._node.is_document()

    def is_doctype(self) -> bool:
        """Returns `True` if the node is a doctype"""
        return self._node.is_doctype()

    def is_comment(self) -> bool:
        """Returns `True` if the node is a comment"""
        return self._node.is_comment()

    def is_text(self) -> bool:
        """Returns `True` if the node is a text"""
        return self._node.is_text()

    def is_element(self) -> bool:
        """Returns `True` if the node is an element"""
        return self._node.is_element()

    def is_processing_instruction(self) -> bool:
        """Returns `True` if the node is a processing instruction"""
        return self._node.is_processing_instruction()

    def parent(self) -> typing.Optional["Node"]:
        """Returns the parent if exist"""
        parent = self._node.parent()
        return Node(parent) if parent is not None else None

    def copy(self) -> "Node":
        """Copies the `self` and returns a new one"""
        return Node(self._node.copy())

    def serialize_html(self, include_self: bool = True) -> bytes:
        """Serialize nodes into HTML document."""
        return self._node.serialize_html(include_self=include_self)

    def serialize_xml(self, include_self: bool = True) -> bytes:
        """Serialize nodes into XML document."""
        return self._node.serialize_xml(include_self=include_self)

    def children(self) -> Children:
        """Returns node children."""
        return Children(self._node)

    def tree(self, include_self: bool = False) -> TreeIterator:
        """Iterates all children and also their children like a tree."""
        return TreeIterator(self._node, include_self=include_self)

    def parents(self, include_self: bool = False) -> ParentsIterator:
        """Iterates all parents"""
        return ParentsIterator(self._node, include_self=include_self)

    def select(self, expr: str, *, parser: typing.Any = None) -> Matching:
        """Execute a css expr and iterates all matched nodes"""
        return Matching(self, expr, parser=parser)

    def text(self, strip: bool = False, strip_chars: typing.Optional[str] = None) -> str:
        _text = ""

        for node in self.tree(include_self=True):
            data = node.data()

            if isinstance(data, TextData):
                _text += data.contents if not strip else data.contents.strip(strip_chars)

        return _text

    def previous_sibling(self) -> typing.Optional["Node"]:
        parent = self.parent()
        if parent is None:
            return None

        children = parent.children()
        index = children.index(self)

        if index == 0:
            return None

        return children[index - 1]

    def next_sibling(self) -> typing.Optional["Node"]:
        parent = self.parent()
        if parent is None:
            return None

        children = parent.children()
        index = children.index(self)

        if index + 1 == len(children):
            return None

        return children[index + 1]

    def __eq__(self, value: typing.Union["Node", RawNode, NodeData]) -> bool:
        if isinstance(value, Node):
            value = value._node

        return self._node == value

    def __repr__(self) -> str:
        return self._node.__repr__()
