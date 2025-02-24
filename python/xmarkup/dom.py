from . import _rustlib
import typing


class TreeDom:
    __slots__ = ("__raw",)

    def __init__(self, *, raw: typing.Optional[_rustlib.TreeDom] = None):
        """
        A tree structure which specialy designed for HTML and XML documents. Uses Rust's `Vec` type in backend.

        The memory consumed by the `TreeDom` is dynamic and depends on the number of tokens stored in the tree.
        The allocated memory is never reduced and is only released when it is dropped.
        """
        if raw is None:
            self.__raw = _rustlib.TreeDom()
        else:
            assert isinstance(raw, _rustlib.TreeDom)
            self.__raw = raw

    def namespaces(self) -> typing.Dict[str, str]:
        """Returns the DOM namespaces."""
        return self.__raw.namespaces()

    def root(self) -> "Document":
        """Returns the root node."""
        return Document(self.__raw.root())

    def __len__(self) -> int:
        return len(self.__raw)

    def __str__(self):
        return str(self.__raw)

    def __repr__(self):
        return repr(self.__raw)


class BaseNode:
    __slots__ = ("_raw",)

    _BASETYPE: typing.Optional[type] = None
    _SUBCLASS_WRAP = {}

    def __init__(self, node: typing.Any):
        if self._BASETYPE is not None and not isinstance(node, self._BASETYPE):
            raise TypeError(
                "expected {} for node, got {} - It's recommended to use nodes `create_*` methods for creating nodes and don't call directly xmarkup.nodes classes.".format(
                    self._BASETYPE.__name__, type(node).__name__
                )
            )

        if not _rustlib._is_node_impl(node):
            raise TypeError(
                "expected one of _rustlib nodes implementations (such as _rustlib.Element, _rustlib.Comment, ...), got {}".format(
                    type(node).__name__
                )
            )

        self._raw = node

    @classmethod
    def _wrap(cls, node: typing.Any) -> "BaseNode":
        try:
            _type = cls._SUBCLASS_WRAP[type(node)]
        except KeyError:
            raise TypeError(
                "the type of node is not acceptable ({}).".format(type(node).__name__)
            ) from None

        return _type(node)

    def __init_subclass__(cls):
        assert cls._BASETYPE is not None
        BaseNode._SUBCLASS_WRAP[cls._BASETYPE] = cls

    def tree(self) -> "TreeDom":
        return TreeDom(raw=self._raw.tree())

    @property
    def parent(self) -> typing.Optional["BaseNode"]:
        parent = self._raw.parent()
        return BaseNode._wrap(parent) if parent is not None else None

    @property
    def prev_sibling(self) -> typing.Optional["BaseNode"]:
        prev_sibling = self._raw.prev_sibling()
        return BaseNode._wrap(prev_sibling) if prev_sibling is not None else None

    @property
    def next_sibling(self) -> typing.Optional["BaseNode"]:
        next_sibling = self._raw.next_sibling()
        return BaseNode._wrap(next_sibling) if next_sibling is not None else None

    @property
    def first_child(self) -> typing.Optional["BaseNode"]:
        first_child = self._raw.first_child()
        return BaseNode._wrap(first_child) if first_child is not None else None

    @property
    def last_child(self) -> typing.Optional["BaseNode"]:
        last_child = self._raw.last_child()
        return BaseNode._wrap(last_child) if last_child is not None else None

    @property
    def has_siblings(self) -> bool:
        return self._raw.has_siblings

    @property
    def has_children(self) -> bool:
        return self._raw.has_children

    def __repr__(self) -> str:
        return repr(self._raw)


class Document(BaseNode):
    _BASETYPE = _rustlib.Document


class Doctype(BaseNode):
    _BASETYPE = _rustlib.Doctype


class Comment(BaseNode):
    _BASETYPE = _rustlib.Comment


class Text(BaseNode):
    _BASETYPE = _rustlib.Text


class Element(BaseNode):
    _BASETYPE = _rustlib.Element


class ProcessingInstruction(BaseNode):
    _BASETYPE = _rustlib.ProcessingInstruction
