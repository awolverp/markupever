from . import _rustlib, iterators
from ._rustlib import QualName as QualName
import typing
import itertools


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

    def select(self, expr: str, limit: int = 0, offset: int = 0) -> iterators.Select:
        """Shorthand for `self.root().select(expr, limit, offset)`"""
        return self.root().select(expr, limit, offset)

    def select_one(self, expr: str, offset: int = 0) -> typing.Optional["Element"]:
        """Shorthand for `self.root().select_one(expr, offset)`"""
        return self.root().select_one(expr, offset)

    def serialize_bytes(self, is_html: typing.Optional[bool] = None) -> bytes:
        """Shorthand for `self.root().serialize_bytes(is_html)`"""
        return self.root().serialize_bytes(is_html=is_html)  # pragma: no cover

    def serialize(self, is_html: typing.Optional[bool] = None) -> str:
        """Shorthand for `self.root().serialize(is_html)`"""
        return self.root().serialize(is_html=is_html)  # pragma: no cover

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
        from ._display import _Indentation

        res = ""

        indent = _Indentation(True)

        for edge in self.root().traverse():
            if edge.closed:
                if edge.node.has_children:
                    indent.deindent()

                continue

            if edge.node.has_children:
                indent.indent(edge.node.next_sibling is not None)
                res += str(indent) + str(edge.node) + "\n"

            else:
                indent.indent(edge.node.next_sibling is not None)
                res += str(indent) + str(edge.node) + "\n"
                indent.deindent()

        return res[:-1]  # remove the last '\n'

    def __repr__(self):
        return f"TreeDom(len={len(self)}, namespaces={self.namespaces()})"


class _ConfigNode:
    __slots__ = ("basetype", "invalid_ordering")

    def __init__(self, basetype: typing.Optional[type], invalid_ordering: typing.Tuple[int]):
        self.basetype = basetype
        self.invalid_ordering = invalid_ordering


class Ordering:
    """Enum for `create_*` methods of a node."""

    APPEND = 0
    """Means create and append the node as the `last_child`."""
    PREPEND = 1
    """Means create and append the node as the `first_child`."""
    AFTER = 2
    """Means create and insert the node as the `next_sibling`."""
    BEFORE = 3
    """Means create and insert the node as the `prev_sibling`."""


class BaseNode:
    __slots__ = ("_raw",)

    _CONFIG: _ConfigNode = _ConfigNode(None, ())
    _SUBCLASS_WRAP = {}

    def __init__(self, node: typing.Any):
        if self._CONFIG.basetype is not None and not isinstance(node, self._CONFIG.basetype):
            raise TypeError(
                "expected {} for node, got {} - It's recommended to use nodes `create_*` methods for creating nodes and don't call directly markupever.nodes classes.".format(
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
        """Returns the parent of this node."""
        parent = self._raw.parent()
        return BaseNode._wrap(parent) if parent is not None else None

    @property
    def prev_sibling(self) -> typing.Optional["BaseNode"]:
        """Returns the previous sibling of this node."""
        prev_sibling = self._raw.prev_sibling()
        return BaseNode._wrap(prev_sibling) if prev_sibling is not None else None

    @property
    def next_sibling(self) -> typing.Optional["BaseNode"]:
        """Returns the next sibling of this node."""
        next_sibling = self._raw.next_sibling()
        return BaseNode._wrap(next_sibling) if next_sibling is not None else None

    @property
    def first_child(self) -> typing.Optional["BaseNode"]:
        """Returns the first child of this node."""
        first_child = self._raw.first_child()
        return BaseNode._wrap(first_child) if first_child is not None else None

    @property
    def last_child(self) -> typing.Optional["BaseNode"]:
        """Returns the last child of this node."""
        last_child = self._raw.last_child()
        return BaseNode._wrap(last_child) if last_child is not None else None

    @property
    def has_siblings(self) -> bool:
        """Returns `True` if the node has sibling."""
        return self._raw.has_siblings

    @property
    def has_children(self) -> bool:
        """Returns `True` if the node has children."""
        return self._raw.has_children

    def tree(self) -> "TreeDom":
        """Returns the tree which this node connected to."""
        return TreeDom(raw=self._raw.tree())

    def children(self) -> iterators.Children:
        """Returns an iterator which iterates over children of this node."""
        return iterators.Children(self)

    def ancestors(self) -> iterators.Ancestors:
        """Returns an iterator which iterates over ancestors (parents) of this node."""
        return iterators.Ancestors(self)

    def prev_siblings(self) -> iterators.PrevSiblings:
        """Returns an iterator which iterates over previous siblings of this node."""
        return iterators.PrevSiblings(self)

    def next_siblings(self) -> iterators.NextSiblings:
        """Returns an iterator which iterates over next siblings of this node."""
        return iterators.NextSiblings(self)

    def first_children(self) -> iterators.FirstChildren:
        """Returns an iterator which iterates over first children."""
        return iterators.FirstChildren(self)  # pragma: no cover

    def last_children(self) -> iterators.LastChildren:
        """Returns an iterator which iterates over last children."""
        return iterators.LastChildren(self)  # pragma: no cover

    def traverse(self) -> iterators.Traverse:
        """Returns a traverse iterator."""
        return iterators.Traverse(self)

    def descendants(self) -> iterators.Descendants:
        """Returns an iterator which iterates over this node and its descendants."""
        return iterators.Descendants(self)

    def attach(self, node: "BaseNode", *, ordering: int = Ordering.APPEND) -> None:
        """
        Attaches and connect `node` to this node depends on `ordering` value.

        This is not important the `node` has detached or not.
        """
        if isinstance(node._raw, _rustlib.Document):
            raise ValueError("you cannot attach a Document node to another node.")

        self._connect_node(ordering, self._raw.tree(), node._raw)

    def detach(self) -> None:
        """
        Detaches this node from other nodes (means makes it an orphan node).

        Note: you cannot detach a node move it to another tree.
        """
        if isinstance(self._raw, _rustlib.Document):
            raise ValueError("you cannot detach Document node.")

        self._raw.tree().detach(self._raw)

    def select(self, expr: str, limit: int = 0, offset: int = 0) -> iterators.Select:
        """
        Returns an iterator that uses CSS selectors to match and find nodes.

        You can use a group of CSS selectors (seperated by comma).
        """
        return iterators.Select(self, expr, limit=limit, offset=offset)

    def select_one(self, expr: str, offset: int = 0) -> typing.Optional["Element"]:
        """
        Works like `self.select(expr, offset=offset)` but only returns the first match.
        """
        selector = iterators.Select(self, expr, limit=1, offset=offset)

        try:
            node = next(selector)
        except StopIteration:
            return None
        else:
            return node

    def strings(self, strip: bool = False):
        """Go through this node descendants and yields texts."""
        for descendant in self.descendants():
            if not isinstance(descendant, Text):
                continue

            if strip:
                yield descendant.content.strip()
            else:
                yield descendant.content

    def text(self, seperator: str = "", strip: bool = False) -> str:
        """Returns the text of this node."""
        return seperator.join(self.strings(strip=strip))

    def serialize_bytes(self, is_html: typing.Optional[bool] = None) -> bytes:
        """Serialize the tree (starts from this node) to bytes."""
        return _rustlib.serialize(self._raw, is_html)

    def serialize(self, is_html: typing.Optional[bool] = None) -> str:
        """Serialize the tree (starts from this node) to string."""
        return self.serialize_bytes(is_html).decode("utf-8")

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
    """The root of a document."""

    _CONFIG = _ConfigNode(_rustlib.Document, (Ordering.AFTER, Ordering.BEFORE))

    def create_doctype(
        self,
        name: str,
        public_id: str = "",
        system_id: str = "",
        *,
        ordering: int = Ordering.APPEND,
    ) -> "Doctype":
        """
        Create and connect a `Doctype` to this node depends on `ordering` value.
        """
        dom = self._raw.tree()
        node = _rustlib.Doctype(dom, name, public_id, system_id)
        self._connect_node(ordering, dom, node)
        return Doctype(node)

    def create_comment(self, content: str, *, ordering: int = Ordering.APPEND) -> "Comment":
        """
        Create and connect a `Comment` to this node depends on `ordering` value.
        """
        dom = self._raw.tree()
        node = _rustlib.Comment(dom, content)
        self._connect_node(ordering, dom, node)
        return Comment(node)

    def create_text(self, content: str, *, ordering: int = Ordering.APPEND) -> "Text":
        """
        Create and connect a `Text` to this node depends on `ordering` value.
        """
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
        """
        Create and connect a `Element` to this node depends on `ordering` value.
        """
        dom = self._raw.tree()

        if isinstance(attrs, dict):
            attrs = list(attrs.items())

        node = _rustlib.Element(dom, name, attrs, template, mathml_annotation_xml_integration_point)
        self._connect_node(ordering, dom, node)
        return Element(node)

    def create_processing_instruction(
        self, data: str, target: str, *, ordering: int = Ordering.APPEND
    ) -> "ProcessingInstruction":
        """
        Create and connect a `ProcessingInstruction` to this node depends on `ordering` value.
        """
        dom = self._raw.tree()
        node = _rustlib.ProcessingInstruction(dom, data, target)
        self._connect_node(ordering, dom, node)
        return ProcessingInstruction(node)


class Doctype(BaseNode):
    """
    the doctype is the required <!doctype html> preamble found at the top of all documents.
    Its sole purpose is to prevent a browser from switching into so-called "quirks mode"
    when rendering a document; that is, the <!doctype html> doctype ensures that the browser makes
    a best-effort attempt at following the relevant specifications, rather than using a different
    rendering mode that is incompatible with some specifications.
    """

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
    """
    The Comment interface represents textual notations within markup; although it is generally not
    visually shown, such comments are available to be read in the source view.

    Comments are represented in HTML and XML as content between <!-- and -->. In XML,
    like inside SVG or MathML markup, the character sequence -- cannot be used within a comment.
    """

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

    def __ne__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content != value

        return super().__ne__(value)

    def __le__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content <= value

        return super().__le__(value)

    def __lt__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content < value

        return super().__lt__(value)

    def __ge__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content >= value

        return super().__ge__(value)

    def __gt__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content > value

        return super().__gt__(value)


class Text(BaseNode):
    """A text node."""

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

    def __ne__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content != value

        return super().__ne__(value)

    def __le__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content <= value

        return super().__le__(value)

    def __lt__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content < value

        return super().__lt__(value)

    def __ge__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content >= value

        return super().__ge__(value)

    def __gt__(self, value):  # pragma: no cover
        if isinstance(value, str):
            return self._raw.content > value

        return super().__gt__(value)


_D = typing.TypeVar("_D")


class AttrsList:
    """
    This type is only designed for communicating with element attributes.

    Really it's a list, but has a behaviour between dictionary and list to provide you easy-to-use management.
    """

    __slots__ = ("__raw",)

    def __init__(self, attrs: _rustlib.AttrsList):
        self.__raw = attrs

    def append(self, key: typing.Union[_rustlib.QualName, str], value: str):
        """
        Appends a key-value pair into attributes list.
        """
        self.__raw.push(key, value)

    def insert(self, index: int, key: typing.Union[_rustlib.QualName, str], value: str):
        """
        Inserts a key-value pair at position `index` within the list, shifting all elements after it to the right.
        """
        self.__raw.insert(index, key, value)

    def _find_by_key(
        self,
        key: typing.Union[_rustlib.QualName, str],
        default: _D = None,
        start: int = 0,
    ) -> typing.Tuple[typing.Union[str, _D], int]:
        for index, item in itertools.islice(enumerate(self.__raw.items()), start, None):
            k, v = item

            if k == key:
                return v, index

        return default, -1

    def _find_by_item(
        self,
        key: typing.Union[_rustlib.QualName, str],
        value: str,
        start: int = 0,
    ) -> int:
        for index, item in itertools.islice(enumerate(self.__raw.items()), start, None):
            k, v = item

            if k == key and value == v:
                return index

        return -1

    def index(
        self, key: typing.Union[typing.Union[_rustlib.QualName, str], tuple], start: int = 0
    ) -> int:
        """
        Returns the first match index.
        """
        if isinstance(key, tuple):
            index = self._find_by_item(*key, start=start)
        else:
            _, index = self._find_by_key(key, start=start)

        if index == -1:
            raise ValueError(key)

        return index

    def get(
        self, key: typing.Union[_rustlib.QualName, str], default: _D = None, start: int = 0
    ) -> typing.Union[str, _D]:
        """
        Return the value for key if key is in the dictionary, else default.
        """
        val, index = self._find_by_key(key, start=start)
        if index == -1:
            return default

        return val

    def dedup(self) -> None:
        """
        Removes consecutive repeated elements in the attributes list.
        """
        self.__raw.dedup()  # pragma: no cover

    def pop(self, index: int = -1) -> typing.Tuple[_rustlib.QualName, str]:
        """
        Remove and return item at index (default last).

        Raises `IndexError` if list is empty or index is out of range.
        """
        if index < 0:
            index = len(self.__raw) + index

        return self.__raw.remove(index)

    def remove(
        self, key: typing.Union[typing.Union[_rustlib.QualName, str], tuple], start: int = 0
    ) -> None:
        """
        Remove first occurrence of value.

        Raises ValueError if the value is not present.
        """
        index = self.index(key, start=start)
        self.__raw.remove(index)

    def reverse(self) -> None:
        """Reverses the order of elements in the list."""
        self.__raw.reverse()

    def extend(self, m: typing.Union[dict, typing.Iterable[tuple]]):
        """
        Extend the attributes list by appending key-value pairs from the iterable or dictionary.
        """
        if isinstance(m, dict):
            m = m.items()

        for key, val in m:
            self.__raw.push(key, val)

    def clear(self) -> None:
        """Clears the attributes list, removing all values."""
        self.__raw.clear()

    def keys(self) -> typing.Generator[QualName, None, None]:
        """Returns a generator of attribute keys."""
        return (i for i, _ in self.__raw.items())

    def values(self) -> typing.Generator[str, None, None]:
        """Returns a generator of attribute values."""
        return (i for _, i in self.__raw.items())

    def __len__(self) -> int:
        """Returns `len(self)`."""
        return len(self.__raw)

    def __iter__(self):
        """Returns a generator of attribute keys."""
        return self.keys()

    def __contains__(self, key: typing.Union[typing.Union[_rustlib.QualName, str], tuple]) -> bool:
        """
        Returns `True` if the list has the specified key, else `False`.
        """
        if isinstance(key, tuple):
            index = self._find_by_item(*key)
        else:
            _, index = self._find_by_key(key)

        return index > -1

    def __delitem__(self, index: typing.Union[int, str, _rustlib.QualName]) -> None:
        """
        - If `index` is `str` or `QualName`: Finds and removes the first match from list.
        - If `index` is `int`: Removes the `index` from list.
        """
        if not isinstance(index, int):
            _, index = self._find_by_key(index)
            if index == -1:
                raise KeyError(index)

        self.__raw.remove(index)

    def __setitem__(
        self,
        index: typing.Union[int, str, _rustlib.QualName],
        val: typing.Union[str, typing.Tuple[typing.Union[_rustlib.QualName, str], str]],
    ) -> None:
        if not isinstance(index, int):
            if isinstance(val, str):
                val = (index, val)

            _, index = self._find_by_key(index)
            if index == -1:
                self.__raw.push(*val)
                return

        self.__raw.update_item(index, val[0], val[1])

    @typing.overload
    def __getitem__(self, index: typing.Union[str, _rustlib.QualName]) -> str: ...

    @typing.overload
    def __getitem__(self, index: int) -> typing.Tuple[_rustlib.QualName, str]: ...

    def __getitem__(self, index):
        if not isinstance(index, int):
            _, index = self._find_by_key(index)
            if index == -1:
                raise KeyError(index)

            _, val = self.__raw.get_by_index(index)
            return val

        return self.__raw.get_by_index(index)

    def __repr__(self) -> str:
        return repr(self.__raw)


class Element(BaseNode):
    """An element node."""

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
        """
        Create and connect a `Doctype` to this node depends on `ordering` value.
        """
        dom = self._raw.tree()
        node = _rustlib.Doctype(dom, name, public_id, system_id)
        self._connect_node(ordering, dom, node)
        return Doctype(node)

    def create_comment(
        self, content: str, *, ordering: int = Ordering.APPEND
    ) -> "Comment":  # pragma: no cover # it is a copy of Document.create_comment
        """
        Create and connect a `Comment` to this node depends on `ordering` value.
        """
        dom = self._raw.tree()
        node = _rustlib.Comment(dom, content)
        self._connect_node(ordering, dom, node)
        return Comment(node)

    def create_text(
        self, content: str, *, ordering: int = Ordering.APPEND
    ) -> "Text":  # pragma: no cover # it is a copy of Document.create_text
        """
        Create and connect a `Text` to this node depends on `ordering` value.
        """
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
        """
        Create and connect a `Element` to this node depends on `ordering` value.
        """
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
        """
        Create and connect a `ProcessingInstruction` to this node depends on `ordering` value.
        """
        dom = self._raw.tree()
        node = _rustlib.ProcessingInstruction(dom, data, target)
        self._connect_node(ordering, dom, node)
        return ProcessingInstruction(node)


class ProcessingInstruction(BaseNode):
    """
    The ProcessingInstruction interface represents a processing instruction; that is,
    a Node which embeds an instruction targeting a specific application but that can
    be ignored by any other applications which don't recognize the instruction.
    """

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
