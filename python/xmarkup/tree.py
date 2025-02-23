from . import _rustlib
import typing


class TreeDom:
    __slots__ = "__raw"

    def __init__(self, *, raw: typing.Optional[_rustlib.TreeDom] = None):
        if raw is None:
            self.__raw = _rustlib.TreeDom()
        else:
            assert isinstance(raw, _rustlib.TreeDom)
            self.__raw = raw
