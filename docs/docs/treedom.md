---
title: DOM Usage
description: Navigating the tree in markupever library in Python
---

# DOM Usage


<div class="grid cards" markdown>

-   :simple-buildkite:{ .lg } - __Build A Document__

    ---

    In **MarkupEver**, we use a class named `TreeDom` (1) as a tree structure. This class allows you to ...

    [More :material-arrow-top-right:](#build-a-document)

-   :material-navigation-variant-outline:{ .lg } - __Navigating the tree__

    ---

    The most important thing in trees is navigating between elements and how to interact with them ...

    [More :material-arrow-top-right:](#navigating-the-tree)

</div>


## Navigating the tree
The most important thing in trees is navigating between elements and how to interact with them — specially after parsing a document.

Each node may contain text and other nodes. **MarkupEver** provides numerous properties, methods, and iterators to help you work with and navigate between them.

!!! note

    We won't discuss .select and .select_one here. They are introduced in [Parsing](./parser.md) and [Querying](./querying.md).

Imagine this to start:

```python
import markupever

dom: markupever.dom.TreeDom = markupever.parse(
    """
    <note><to>Tove</to>
        <from>Jani</from>
        <heading>Reminder</heading>
        <body>Don't forget me this weekend!</body></note>
    """,
    markupever.XmlOptions()
)
root = dom.root()

# Document
# └── Element(name=QualName(local="note"), attrs=[], template=false, integration_point=false)
#     ├── Element(name=QualName(local="to"), attrs=[], template=false, integration_point=false)
#     │   └── Text(content="Tove")
#     ├── Text(content="\n        ")
#     ├── Element(name=QualName(local="from"), attrs=[], template=false, integration_point=false)
#     │   └── Text(content="Jani")
#     ├── Text(content="\n        ")
#     ├── Element(name=QualName(local="heading"), attrs=[], template=false, integration_point=false)
#     │   └── Text(content="Reminder")
#     ├── Text(content="\n        ")
#     └── Element(name=QualName(local="body"), attrs=[], template=false, integration_point=false)
#         └── Text(content="Don't forget me this weekend!")
```

Let's discuss about `first_child`, `last_child`, `parent`, `next_sibling` and `prev_sibling` properties:

* **first_child**: This property retrieves the first child node of the given element. If the element has no children, it returns `None`.

* **last_child**: This property retrieves the last child node of the given element. If the element has no children, it returns `None`.

* **parent**: This property retrieves the parent node of the given element. If the element has no parent (e.g., it's the root), it returns `None`.

* **next_sibling**: This property retrieves the next sibling node of the given element. If there is no next sibling, it returns `None`.

* **prev_sibling**: This property retrieves the previous sibling node of the given element. If there is no previous sibling, it returns `None`.

=== "`first_child`"

    ```python
    root.first_child
    # Element(name=QualName(local="note"), attrs=[], template=false, integration_point=false)

    root.first_child.first_child
    # Element(name=QualName(local="to"), attrs=[], template=false, integration_point=false)
    ```

=== "`last_child`"

    ```python
    root.last_child
    # Element(name=QualName(local="note"), attrs=[], template=false, integration_point=false)

    root.last_child.last_child
    # Element(name=QualName(local="body"), attrs=[], template=false, integration_point=false)
    ```

=== "`parent`"

    ```python
    note_element = root.first_child
    # Element(name=QualName(local="note"), attrs=[], template=false, integration_point=false)

    note_element.parent
    # Document

    note_element.parent == root
    # True
    ```

=== "`next_sibling`"

    ```python
    root.next_sibling
    # None

    root.first_child.next_sibling
    # None

    root.first_child.first_child
    # Element(name=QualName(local="body"), attrs=[], template=false, integration_point=false)

    root.first_child.first_child.next_sibling
    # Text(content="\n        ")
    ```

=== "`prev_sibling`"

    ```python
    root.next_sibling
    # None

    root.first_child.prev_sibling
    # None

    root.first_child.last_children
    # Element(name=QualName(local="to"), attrs=[], template=false, integration_point=false)

    root.first_child.first_child.prev_sibling
    # Text(content="\n        ")
    ```

While these properties are useful, they might not always meet our needs. In such cases, methods like `.children()`, `.ancestors()`, `.prev_siblings()`, `.next_siblings()`, `.first_children()`, `.last_children()`, `.traverse()`, and `.descendants()` can provide additional functionality.

* **children()** - Returns an iterator which iterates over children of node.
* **ancestors()** - Returns an iterator which iterates over ancestors (parents) of node.
* **prev_siblings()** - Returns an iterator which iterates over previous siblings of node.
* **next_siblings()** - Returns an iterator which iterates over next siblings of node.
* **first_children()** - Returns an iterator which iterates over first children.
* **last_children()** - Returns an iterator which iterates over last children.
* **traverse()** - Returns a traverse iterator.
* **descendants()** - Returns an iterator which iterates over a node and its descendants.

## Build a document
In **MarkupEver**, we use a class named `TreeDom` (1) as a tree structure. This class allows you to work with the document — move, create, remove, select, serialize, and more. In this tutorial, <u>we'll create a document without using the `Parser` class</u>. We'll focus on `TreeDom` properties and methods.
{ .annotate }

1.  A tree structure which specialy designed for HTML and XML documents. Uses Rust's `Vec` type in backend.
    The memory consumed by the `TreeDom` is dynamic and depends on the number of tokens stored in the tree.
    The allocated memory is never reduced and is only released when it is dropped.

### Start
To start creating a document, we first need to create a `TreeDom`.

```python
>>> from markupever import dom
>>> tree = dom.TreeDom()
```

Each `TreeDom` always has a root node of the `dom.Document` type. We can access it using the `.root()` method.

```python hl_lines="4"
>>> from markupever import dom
>>> tree = dom.TreeDom()
>>> 
>>> root = tree.root() # type is dom.Document
>>> root
Document
```

!!! tip "Be Careful"

    Avoid using `is` for node types in `markupever.dom` (such as `Document`, `Element`, `Text`, etc.) because they are not alive and serve only as a bridge for you to communicate with the core written in **Rust**.

    ```python
    >>> root = tree.root()
    >>> root is tree.root()
    False
    ```

### Adding nodes
`dom.Document` and `dom.Element` types have methods start with `create_`. These are help you to create and add new nodes to
document. Let's add a DOCTYPE to our document:

```python hl_lines="4"
>>> from markupever import dom
>>> tree = dom.TreeDom()
>>>
>>> tree.root().create_doctype("html")
Doctype(name="html", public_id="", system_id="")
```

Let's check what we did by printing or serializing the tree:
```python
>>> print(tree)
Document
└── Doctype(name="html", public_id="", system_id="")

>>> tree.serialize(is_html=True)
'<!DOCTYPE html>'
```

OK. Let's add some elements and check again:
```python
>>> html = tree.root().create_element("html", {"lang": "en"}) # type is dom.Element
>>> html.create_element("body")
Element(name=QualName(local="body", ns="", prefix=None), attrs=[], template=false, integration_point=false)

>>> print(tree)
Document
└── Element(name=QualName(local="html", ns="", prefix=None), attrs=[(QualName(local="lang", ns="", prefix=None), "en")], template=false, integration_point=false)
    └── Element(name=QualName(local="body", ns="", prefix=None), attrs=[], template=false, integration_point=false)

>>> tree.serialize(is_html=True)
'<!DOCTYPE html><html lang="en"><body></body></html>'
```

This is very easy as you can see ...

### Ordering
The `create_*` methods allow you to perform append, prepend, insert after, and insert before operations within the document.

* **append** means adding a child as the last child of a node (default).
* **prepend** means adding a child as the first child of a node.
* **insert after** means adding a node as the next sibling of another node.
* **insert before** means adding a node as the previous sibling of another node.

You can specify the operation with `dom.Ordering` class and the `ordering` parameter in the `create_*` methods.

=== "Append"

    ```python hl_lines="9"
    from markupever import dom
    tree = dom.TreeDom()
    root = tree.root()

    root.create_element("child1")
    # document
    # └── child1

    root.create_element("child2", ordering=dom.Ordering.APPEND)
    # document
    # ├── child1
    # └── child2
    ```

=== "Prepend"

    ```python hl_lines="9"
    from markupever import dom
    tree = dom.TreeDom()
    root = tree.root()

    root.create_element("child1")
    # document
    # └── child1

    root.create_element("child2", ordering=dom.Ordering.PREPEND)
    # document
    # ├── child2
    # └── child1
    ```

=== "Insert After"

    ```python hl_lines="9"
    from markupever import dom
    tree = dom.TreeDom()
    root = tree.root()

    child = root.create_element("child")
    # document
    # └── child

    child.create_element("sibling", ordering=dom.Ordering.AFTER)
    # document
    # ├── child
    # └── sibling
    ```

=== "Insert Before"

    ```python hl_lines="9"
    from markupever import dom
    tree = dom.TreeDom()
    root = tree.root()

    child = root.create_element("child")
    # document
    # └── child

    child.create_element("sibling", ordering=dom.Ordering.BEFORE)
    # document
    # ├── sibling
    # └── child
    ```
