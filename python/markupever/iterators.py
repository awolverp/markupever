from . import _rustlib
import typing


if typing.TYPE_CHECKING:  # pragma: no cover
    from . import dom


class _IteratorMetaClass:
    """Bridge _rustlib iterators to Python"""

    _BASECLASS: typing.Callable[["dom.BaseNode"], typing.Iterable]

    __slots__ = ("_raw",)

    def __init__(self, value: "dom.BaseNode"):
        self._raw = iter(self._BASECLASS(value._raw))

    def __iter__(self):
        """Returns `iter(self)`"""
        return self

    def __next__(self) -> "dom.BaseNode":
        """Returns `next(self)`"""
        from .dom import BaseNode

        return BaseNode._wrap(next(self._raw))


class Ancestors(_IteratorMetaClass):
    """Iterates over ancestors (parents)."""

    _BASECLASS = _rustlib.iter.Ancestors


class PrevSiblings(_IteratorMetaClass):
    """Iterates over previous siblings."""

    _BASECLASS = _rustlib.iter.PrevSiblings


class NextSiblings(_IteratorMetaClass):
    """Iterates over next siblings."""

    _BASECLASS = _rustlib.iter.NextSiblings


class FirstChildren(_IteratorMetaClass):
    """Iterates over first children."""

    _BASECLASS = _rustlib.iter.FirstChildren


class LastChildren(_IteratorMetaClass):
    """Iterates over last children."""

    _BASECLASS = _rustlib.iter.LastChildren


class Children(_IteratorMetaClass):
    """Iterates children of a node."""

    _BASECLASS = _rustlib.iter.Children


class EdgeTraverse:
    """Open or close edge of a node (The returning type of `Traverse`)."""

    __slots__ = ("node", "closed")

    def __init__(self, node: "dom.BaseNode", closed: bool) -> None:
        self.node = node
        self.closed = closed

    def __repr__(self):
        if self.closed:
            return f"EdgeTraverse[closed]({self.node})"

        return f"EdgeTraverse[opened]({self.node})"


class Traverse(_IteratorMetaClass):
    """Iterator which traverses a tree."""

    _BASECLASS = _rustlib.iter.Traverse

    def __next__(self) -> EdgeTraverse:
        from .dom import BaseNode

        rn, closed = next(self._raw)
        return EdgeTraverse(BaseNode._wrap(rn), closed)


class Descendants(_IteratorMetaClass):
    """Iterates over a node and its descendants."""

    _BASECLASS = _rustlib.iter.Descendants


class Select:
    """An iterator that uses CSS selectors to match and find nodes."""

    __slots__ = ("__raw", "__limit", "__offset")

    def __init__(
        self, value: "dom.BaseNode", expr: str, *, limit: int = 0, offset: int = 0
    ) -> None:
        self.__raw = iter(_rustlib.Select(value._raw, expr))

        self.__limit = limit or -1
        self.__offset = offset - 1

    def __iter__(self):
        return self

    def __next__(self) -> "dom.Element":
        from .dom import Element

        if self.__limit == 0:
            raise StopIteration

        while self.__offset > 0:
            next(self.__raw)
            self.__offset -= 1

        node = Element(next(self.__raw))
        self.__limit -= 1

        return node
