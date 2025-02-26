from . import _rustlib, iterators
from ._rustlib import QualName as QualName
import typing


class TreeDom:
    __slots__ = ("_raw",)

    def __init__(self, *, raw: typing.Optional[_rustlib.TreeDom] = None):
        """
        A tree structure which specialy designed for HTML and XML documents. Uses Rust's `Vec` type in backend.

        The memory consumed by the `TreeDom` is dynamic and depends on the number of tokens stored in the tree.
        The allocated memory is never reduced and is only released when it is dropped.
        """
        if raw is None:
            self._raw = _rustlib.TreeDom()
        else:
            assert isinstance(raw, _rustlib.TreeDom)
            self._raw = raw

    def namespaces(self) -> typing.Dict[str, str]:
        """Returns the DOM namespaces."""
        return self._raw.namespaces()

    def root(self) -> "Document":
        """Returns the root node."""
        return Document(self._raw.root())

    def __iter__(self) -> typing.Generator["BaseNode", typing.Any, None]:
        """Iterates the nodes in insert order - don't matter which are orphan which not."""
        for rn in _rustlib.iter.Iterator(self._raw):
            yield BaseNode._wrap(rn)

    def __eq__(self, val: "TreeDom") -> bool:
        if not isinstance(val, TreeDom):
            return False

        return self._raw == val._raw

    def __len__(self) -> int:
        """Returns the number of nodes in tree."""
        return len(self._raw)

    def __str__(self) -> str:
        return str(self._raw)

    def __repr__(self) -> str:
        return repr(self._raw)


class _ConfigNode:
    __slots__ = ("basetype", "invalid_ordering")

    def __init__(self, basetype: typing.Optional[type], invalid_ordering: typing.Tuple[int]):
        self.basetype = basetype
        self.invalid_ordering = invalid_ordering


class Ordering:
    APPEND = 0
    PREPEND = 1
    AFTER = 2
    BEFORE = 3


class BaseNode:
    __slots__ = ("_raw",)

    _CONFIG: _ConfigNode = _ConfigNode(None, ())
    _SUBCLASS_WRAP = {}

    def __init__(self, node: typing.Any):
        if self._CONFIG.basetype is not None and not isinstance(node, self._CONFIG.basetype):
            raise TypeError(
                "expected {} for node, got {} - It's recommended to use nodes `create_*` methods for creating nodes and don't call directly xmarkup.nodes classes.".format(
                    self._CONFIG.basetype.__name__, type(node).__name__
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

    def _connect_node(self, ordering: int, dom, child):
        if ordering in self._CONFIG.invalid_ordering:
            raise ValueError("This ordering value is not acceptable for this type.")

        if ordering == Ordering.APPEND:
            dom.append(self._raw, child)

        elif ordering == Ordering.PREPEND:
            dom.prepend(self._raw, child)

        elif ordering == Ordering.AFTER:
            dom.insert_after(self._raw, child)

        elif ordering == Ordering.BEFORE:
            dom.insert_before(self._raw, child)

        else:
            raise ValueError(
                "ordering must be one of Ordering variables like Ordering.APPEND, Ordering.PREPEND, ..."
            )

    def __init_subclass__(cls):
        assert cls._CONFIG.basetype is not None
        BaseNode._SUBCLASS_WRAP[cls._CONFIG.basetype] = cls

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

    def tree(self) -> "TreeDom":
        return TreeDom(raw=self._raw.tree())

    def children(self) -> iterators.Children:
        return iterators.Children(self)

    def ancestors(self) -> iterators.Ancestors:
        return iterators.Ancestors(self)

    def prev_siblings(self) -> iterators.PrevSiblings:
        return iterators.PrevSiblings(self)

    def next_siblings(self) -> iterators.NextSiblings:
        return iterators.NextSiblings(self)

    def first_children(self) -> iterators.FirstChildren:
        return iterators.FirstChildren(self)

    def last_children(self) -> iterators.LastChildren:
        return iterators.LastChildren(self)

    def traverse(self) -> iterators.Traverse:
        return iterators.Traverse(self)

    def descendants(self) -> iterators.Descendants:
        return iterators.Descendants(self)

    def detach(self) -> None:
        if isinstance(self._raw, _rustlib.Document):
            raise ValueError("you cannot detach Document instance.")

        self._raw.tree().detach(self._raw)

    def select(self, expr: str, limit: int = 0, offset: int = 0) -> iterators.Select:
        return iterators.Select(self, expr, limit=limit, offset=offset)

    def select_one(self, expr: str, offset: int = 0) -> typing.Optional["Element"]:
        selector = iterators.Select(self, expr, limit=1, offset=offset)

        try:
            node = next(selector)
        except StopIteration:
            return None
        else:
            return node

    def strings(self, strip: bool = False):
        for descendant in self.descendants():
            if not isinstance(descendant, Text):
                continue

            if strip:
                yield descendant.content.strip()
            else:
                yield descendant.content

    def text(self, seperator: str = "", strip: bool = False) -> str:
        return seperator.join(self.strings(strip=strip))

    def serialize_bytes(self, is_xml: bool = False) -> bytes:
        return _rustlib.serialize(self._raw, is_xml)

    def serialize(self, is_xml: bool = False) -> str:
        return self.serialize_bytes(is_xml).decode("utf-8")

    def __eq__(self, value):
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw == value

    def __ne__(self, value):  # pragma: no cover
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw != value

    def __le__(self, value):  # pragma: no cover
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw <= value

    def __lt__(self, value):  # pragma: no cover
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw < value

    def __ge__(self, value):  # pragma: no cover
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw >= value

    def __gt__(self, value):  # pragma: no cover
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw > value

    def __repr__(self) -> str:  # pragma: no cover
        return repr(self._raw)


class Document(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.Document, (Ordering.AFTER, Ordering.BEFORE))

    def create_doctype(
        self,
        name: str,
        public_id: str = "",
        system_id: str = "",
        *,
        ordering: int = Ordering.APPEND,
    ) -> "Doctype":
        dom = self._raw.tree()
        node = _rustlib.Doctype(dom, name, public_id, system_id)
        self._connect_node(ordering, dom, node)
        return Doctype(node)

    def create_comment(self, content: str, *, ordering: int = Ordering.APPEND) -> "Comment":
        dom = self._raw.tree()
        node = _rustlib.Comment(dom, content)
        self._connect_node(ordering, dom, node)
        return Comment(node)

    def create_text(self, content: str, *, ordering: int = Ordering.APPEND) -> "Text":
        dom = self._raw.tree()
        node = _rustlib.Text(dom, content)
        self._connect_node(ordering, dom, node)
        return Text(node)

    def create_element(
        self,
        name: str,
        attrs: typing.Union[
            typing.Sequence[typing.Tuple[typing.Union[_rustlib.QualName, str], str]],
            typing.Dict[typing.Union[_rustlib.QualName, str], str],
        ] = [],
        template: bool = False,
        mathml_annotation_xml_integration_point: bool = False,
        *,
        ordering: int = Ordering.APPEND,
    ) -> "Element":
        dom = self._raw.tree()

        if isinstance(attrs, dict):
            attrs = list(attrs.items())

        node = _rustlib.Element(dom, name, attrs, template, mathml_annotation_xml_integration_point)
        self._connect_node(ordering, dom, node)
        return Element(node)

    def create_processing_instruction(
        self, data: str, target: str, *, ordering: int = Ordering.APPEND
    ) -> "ProcessingInstruction":
        dom = self._raw.tree()
        node = _rustlib.ProcessingInstruction(dom, data, target)
        self._connect_node(ordering, dom, node)
        return ProcessingInstruction(node)


class Doctype(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.Doctype, (Ordering.APPEND, Ordering.PREPEND))

    @property
    def name(self) -> str:
        return self._raw.name

    @name.setter
    def name(self, value: str) -> None:
        self._raw.name = value

    @property
    def system_id(self) -> str:
        return self._raw.system_id

    @system_id.setter
    def system_id(self, value: str) -> None:
        self._raw.system_id = value

    @property
    def public_id(self) -> str:
        return self._raw.public_id

    @public_id.setter
    def public_id(self, value: str) -> None:
        self._raw.public_id = value


class Comment(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.Comment, (Ordering.APPEND, Ordering.PREPEND))

    @property
    def content(self) -> str:
        return self._raw.content

    @content.setter
    def content(self, value: str) -> None:
        self._raw.content = value

    def __eq__(self, value):
        if isinstance(value, str):
            return self._raw.content == value

        return super().__eq__(value)

    def __ne__(self, value):
        if isinstance(value, str):
            return self._raw.content != value

        return super().__ne__(value)

    def __le__(self, value):
        if isinstance(value, str):
            return self._raw.content <= value

        return super().__le__(value)

    def __lt__(self, value):
        if isinstance(value, str):
            return self._raw.content < value

        return super().__lt__(value)

    def __ge__(self, value):
        if isinstance(value, str):
            return self._raw.content >= value

        return super().__ge__(value)

    def __gt__(self, value):
        if isinstance(value, str):
            return self._raw.content > value

        return super().__gt__(value)


class Text(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.Text, (Ordering.APPEND, Ordering.PREPEND))

    @property
    def content(self) -> str:
        return self._raw.content

    @content.setter
    def content(self, value: str) -> None:
        self._raw.content = value

    def __eq__(self, value):
        if isinstance(value, str):
            return self._raw.content == value

        return super().__eq__(value)

    def __ne__(self, value):
        if isinstance(value, str):
            return self._raw.content != value

        return super().__ne__(value)

    def __le__(self, value):
        if isinstance(value, str):
            return self._raw.content <= value

        return super().__le__(value)

    def __lt__(self, value):
        if isinstance(value, str):
            return self._raw.content < value

        return super().__lt__(value)

    def __ge__(self, value):
        if isinstance(value, str):
            return self._raw.content >= value

        return super().__ge__(value)

    def __gt__(self, value):
        if isinstance(value, str):
            return self._raw.content > value

        return super().__gt__(value)


_D = typing.TypeVar("_D")


class AttrsList:
    __slots__ = ("__raw",)

    def __init__(self, attrs: _rustlib.AttrsList):
        self.__raw = attrs

    def append(self, key: typing.Union[_rustlib.QualName, str], value: str):
        self.__raw.push(key, value)

    def insert(self, index: int, key: typing.Union[_rustlib.QualName, str], value: str):
        self.__raw.insert(index, key, value)

    def _find_by_key(
        self,
        key: typing.Union[_rustlib.QualName, str],
        default: _D = None,
    ) -> typing.Tuple[typing.Union[str, _D], int]:
        for index, item in enumerate(self.__raw.items()):
            k, v = item

            if k == key:
                return v, index

        return default, -1

    def _find_by_item(
        self,
        key: typing.Union[_rustlib.QualName, str],
        value: str,
    ) -> int:
        for index, item in enumerate(self.__raw.items()):
            k, v = item

            if k == key and value == v:
                return index

        return -1

    def index(self, key: typing.Union[typing.Union[_rustlib.QualName, str], tuple]) -> int:
        if isinstance(key, tuple):
            index = self._find_by_item(*key)
        else:
            _, index = self._find_by_key(key)

        if index == -1:
            raise ValueError(key)

        return index

    def find(
        self, key: typing.Union[_rustlib.QualName, str], default: _D = None
    ) -> typing.Tuple[typing.Union[str, _D], int]:
        val, index = self._find_by_key(key)
        if index == -1:
            return default

        return val

    def dedup(self) -> None:
        self.__raw.dedup()

    def pop(self, index: int = -1) -> typing.Tuple[_rustlib.QualName, str]:
        return self.__raw.remove(index)

    def remove(self, key: typing.Union[typing.Union[_rustlib.QualName, str], tuple]) -> None:
        index = self.index(key)
        self.__raw.remove(index)

    def reverse(self) -> None:
        self.__raw.reverse()

    def extend(self, m: typing.Union[dict, typing.Iterable[tuple]]):
        if isinstance(m, dict):
            m = m.items()

        for key, val in m:
            self.__raw.push(key, val)

    def clear(self) -> None:
        self.__raw.clear()

    def __iter__(self) -> _rustlib.AttrsListItems:
        return iter(self.__raw)

    def __contains__(self, key: typing.Union[typing.Union[_rustlib.QualName, str], tuple]) -> bool:
        if isinstance(key, tuple):
            index = self._find_by_item(*key)
        else:
            _, index = self._find_by_key(key)

        return index > -1

    def __delitem__(self, index: int) -> None:
        self.__raw.remove(index)

    def __setitem__(
        self, index: int, val: typing.Tuple[typing.Union[_rustlib.QualName, str], str]
    ) -> None:
        self.__raw.update_item(index, val[0], val[1])

    def __getitem__(self, index: int) -> typing.Tuple[_rustlib.QualName, str]:
        return self.__raw.get_by_index(index)

    def __repr__(self) -> str:
        return repr(self.__raw)


class Element(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.Element, ())

    @property
    def name(self) -> _rustlib.QualName:
        return self._raw.name

    @name.setter
    def name(self, value: typing.Union[str, _rustlib.QualName]) -> None:
        self._raw.name = value

    @property
    def attrs(self) -> AttrsList:
        return AttrsList(self._raw.attrs)

    @attrs.setter
    def attrs(
        self,
        value: typing.Union[
            typing.Sequence[typing.Tuple[typing.Union[_rustlib.QualName, str], str]],
            typing.Dict[typing.Union[_rustlib.QualName, str], str],
        ],
    ) -> None:
        if isinstance(value, dict):
            value = list(value.items())

        self._raw.attrs = value

    @property
    def template(self) -> bool:
        return self._raw.template

    @template.setter
    def template(self, value: bool) -> None:
        self._raw.template = value

    @property
    def mathml_annotation_xml_integration_point(self) -> bool:
        return self._raw.mathml_annotation_xml_integration_point

    @mathml_annotation_xml_integration_point.setter
    def mathml_annotation_xml_integration_point(self, value: bool) -> None:
        self._raw.mathml_annotation_xml_integration_point = value

    @property
    def id(self) -> typing.Optional[str]:
        return self._raw.id()

    @property
    def class_list(self) -> typing.List[str]:
        return self._raw.class_list()

    def create_doctype(
        self,
        name: str,
        public_id: str = "",
        system_id: str = "",
        *,
        ordering: int = Ordering.APPEND,
    ) -> "Doctype":  # pragma: no cover # it is a copy of Document.create_doctype
        dom = self._raw.tree()
        node = _rustlib.Doctype(dom, name, public_id, system_id)
        self._connect_node(ordering, dom, node)
        return Doctype(node)

    def create_comment(
        self, content: str, *, ordering: int = Ordering.APPEND
    ) -> "Comment":  # pragma: no cover # it is a copy of Document.create_comment
        dom = self._raw.tree()
        node = _rustlib.Comment(dom, content)
        self._connect_node(ordering, dom, node)
        return Comment(node)

    def create_text(
        self, content: str, *, ordering: int = Ordering.APPEND
    ) -> "Text":  # pragma: no cover # it is a copy of Document.create_text
        dom = self._raw.tree()
        node = _rustlib.Text(dom, content)
        self._connect_node(ordering, dom, node)
        return Text(node)

    def create_element(
        self,
        name: str,
        attrs: typing.Union[
            typing.Sequence[typing.Tuple[typing.Union[_rustlib.QualName, str], str]],
            typing.Dict[typing.Union[_rustlib.QualName, str], str],
        ] = [],
        template: bool = False,
        mathml_annotation_xml_integration_point: bool = False,
        *,
        ordering: int = Ordering.APPEND,
    ) -> "Element":  # pragma: no cover # it is a copy of Document.create_element
        dom = self._raw.tree()

        if isinstance(attrs, dict):
            attrs = list(attrs.items())

        node = _rustlib.Element(dom, name, attrs, template, mathml_annotation_xml_integration_point)
        self._connect_node(ordering, dom, node)
        return Element(node)

    def create_processing_instruction(
        self, data: str, target: str, *, ordering: int = Ordering.APPEND
    ) -> (
        "ProcessingInstruction"
    ):  # pragma: no cover # it is a copy of Document.create_processing_instruction
        dom = self._raw.tree()
        node = _rustlib.ProcessingInstruction(dom, data, target)
        self._connect_node(ordering, dom, node)
        return ProcessingInstruction(node)


class ProcessingInstruction(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.ProcessingInstruction, (Ordering.APPEND, Ordering.PREPEND))

    @property
    def target(self) -> str:
        return self._raw.target

    @target.setter
    def target(self, value: str) -> None:
        self._raw.target = value

    @property
    def data(self) -> str:
        return self._raw.data

    @data.setter
    def data(self, value: str) -> None:
        self._raw.data = value
