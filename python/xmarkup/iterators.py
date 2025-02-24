from . import _rustlib
import typing


if typing.TYPE_CHECKING:
    from . import dom


class _IteratorMetaClass:
    _BASECLASS: typing.Callable[["dom.BaseNode"], typing.Iterable]

    __slots__ = ("_raw",)

    def __init__(self, value: "dom.BaseNode"):
        self._raw = iter(self._BASECLASS(value._raw))

    def __iter__(self):
        return self

    def __next__(self) -> "dom.BaseNode":
        from .dom import BaseNode

        return BaseNode._wrap(next(self._raw))


class Ancestors(_IteratorMetaClass):
    _BASECLASS = _rustlib.iter.Ancestors


class PrevSiblings(_IteratorMetaClass):
    _BASECLASS = _rustlib.iter.PrevSiblings


class NextSiblings(_IteratorMetaClass):
    _BASECLASS = _rustlib.iter.NextSiblings


class FirstChildren(_IteratorMetaClass):
    _BASECLASS = _rustlib.iter.FirstChildren


class LastChildren(_IteratorMetaClass):
    _BASECLASS = _rustlib.iter.LastChildren


class Children(_IteratorMetaClass):
    _BASECLASS = _rustlib.iter.Children


class EdgeTraverse:
    __slots__ = ("node", "closed")

    def __init__(self, node: "dom.BaseNode", closed: bool) -> None:
        self.node = node
        self.closed = closed

    def __repr__(self):
        if self.closed:
            return f"EdgeTraverse[closed]({self.node})"

        return f"EdgeTraverse[opened]({self.node})"


class Traverse(_IteratorMetaClass):
    _BASECLASS = _rustlib.iter.Traverse

    def __next__(self) -> EdgeTraverse:
        from .dom import BaseNode

        rn, closed = next(self._raw)
        return EdgeTraverse(BaseNode._wrap(rn), closed)


class Descendants(_IteratorMetaClass):
    _BASECLASS = _rustlib.iter.Descendants


class Select:
    __slots__ = ("__raw", "__limit", "__offset")

    def __init__(
        self, value: "dom.BaseNode", expr: str, *, limit: int = 0, offset: int = 0
    ) -> None:
        self.__raw = iter(_rustlib.Select(value._raw, expr))

        self.__limit = limit or -1
        self.__offset = offset or -1

    def __iter__(self):
        return self

    def __next__(self) -> "dom.Element":
        from .dom import Element

        if self.__limit <= 0:
            raise StopIteration

        while self.__offset > 0:
            next(self.__raw)
            self.__offset -= 1

        node = Element(next(self.__raw))
        self.__limit -= 1

        return typing.cast(Element, node)
