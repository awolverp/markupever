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


class _ConfigNode:
    __slots__ = ("basetype", "invalid_ordering")

    def __init__(self, basetype: typing.Optional[type], invalid_ordering: typing.Tuple[int]):
        self.basetype = basetype
        self.invalid_ordering = invalid_ordering


class Order:
    APPEND = 0
    PREPEND = 1
    INSERT_AFTER = 2
    INSERT_BEFORE = 3


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

    def _connect_node(self, ordering: int, dom, child) -> "BaseNode":
        if ordering in self._CONFIG.invalid_ordering:
            raise ValueError("This ordering value is not acceptable for this type.")

        if ordering == Order.APPEND:
            dom.append(self._raw, child)

        if ordering == Order.PREPEND:
            dom.prepend(self._raw, child)

        if ordering == Order.INSERT_AFTER:
            dom.insert_after(self._raw, child)

        if ordering == Order.INSERT_BEFORE:
            dom.insert_before(self._raw, child)

        else:
            raise ValueError(
                "ordering must be one of Order variables like Order.APPEND, Order.PREPEND, ..."
            )

        return BaseNode._wrap(child)

    def __init_subclass__(cls):
        assert cls._CONFIG.basetype is not None
        BaseNode._SUBCLASS_WRAP[cls._CONFIG.basetype] = cls

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

    def detach(self) -> None:
        if isinstance(self._raw, _rustlib.Document):
            raise ValueError("you cannot detach Document instance.")

        self._raw.tree().detach(self._raw)

    def __eq__(self, value):
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw == value

    def __ne__(self, value):
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw != value

    def __le__(self, value):
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw <= value

    def __lt__(self, value):
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw < value

    def __ge__(self, value):
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw >= value

    def __gt__(self, value):
        if isinstance(value, BaseNode):
            value = value._raw

        return self._raw > value

    def __repr__(self) -> str:
        return repr(self._raw)


class Document(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.Document, (Order.INSERT_AFTER, Order.INSERT_BEFORE))

    def create_doctype(
        self, name: str, public_id: str = "", system_id: str = "", *, ordering: int = Order.APPEND
    ) -> "Doctype":
        dom = self._raw.tree()

        return typing.cast(
            Doctype,
            self._connect_node(ordering, dom, _rustlib.Doctype(dom, name, public_id, system_id)),
        )

    def create_comment(self, content: str, *, ordering: int = Order.APPEND) -> "Comment":
        dom = self._raw.tree()

        return typing.cast(
            Comment, self._connect_node(ordering, dom, _rustlib.Comment(dom, content))
        )

    def create_text(self, content: str, *, ordering: int = Order.APPEND) -> "Text":
        dom = self._raw.tree()

        return typing.cast(Text, self._connect_node(ordering, dom, _rustlib.Text(dom, content)))

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
        ordering: int = Order.APPEND,
    ) -> "Element":
        dom = self._raw.tree()

        if isinstance(attrs, dict):
            attrs = list(attrs.items())

        return typing.cast(
            Element,
            self._connect_node(
                ordering,
                dom,
                _rustlib.Element(
                    dom, name, attrs, template, mathml_annotation_xml_integration_point
                ),
            ),
        )

    def create_processing_instruction(
        self, data: str, target: str, *, ordering: int = Order.APPEND
    ) -> "ProcessingInstruction":
        dom = self._raw.tree()

        return typing.cast(
            ProcessingInstruction,
            self._connect_node(ordering, dom, _rustlib.ProcessingInstruction(dom, data, target)),
        )


class Doctype(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.Doctype, (Order.APPEND, Order.PREPEND))

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
    _CONFIG = _ConfigNode(_rustlib.Comment, (Order.APPEND, Order.PREPEND))

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
    _CONFIG = _ConfigNode(_rustlib.Text, (Order.APPEND, Order.PREPEND))

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


class Element(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.Element, ())


class ProcessingInstruction(BaseNode):
    _CONFIG = _ConfigNode(_rustlib.ProcessingInstruction, (Order.APPEND, Order.PREPEND))
