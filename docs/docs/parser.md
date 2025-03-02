---
title: Parsing Usage
---

# Getting started
The first thing expected from such this libraries is the ability to read HTML, XML, and similar documents.

The **MarkupEver** is designed specially for *reading*, *parsing*, and *repairing* HTML and XML documents (also can parse similar documents).

In **MarkupEver** we have some functions (1) and a class (2) for doing that.
{ .annotate }

1.  `.parse()` and `.parse_file()` functions
2.  `Parser` class

Additionaly, they have special features that distinguish this library from others:

* You don't worry about **huge memory** usage.
* You can read and parse documents **part by part** (such as files, streams, ...).
* You can specify some **options** by parsing which are help you (with `HtmlOptions()` ans `XmlOptions()` classes).
* You can **repair** invalid documents automatically.

## Parsing Html
Imagine this **`index.html`** file:

```html title="index.html"
<!DOCTYPE html>
<html>
<head>
    <title>Incomplete Html</title>
</head>
<body>
    <h1>Welcome to My Webpage</h1>
    
    <p>This is a paragraph of text. Below is a list of my favorite websites:</p>
    
    <ul>
        <li><a href="https://www.example.com">Example Website</a></li>
        <li><a href="https://www.wikipedia.org">Wikipedia</a></li>
        <li><a href="https://www.bbc.com">BBC</a></li>
        <li><a href="https://www.microsoft.com">Microsoft</a></li>
    </ul>
    
    <p>If you have any questions, feel free to <a href="mailto:example@example.com">contact me</a>.</p>
```

We can use `.parse()` and `.parse_file()` functions to parse documents.

!!! tip "The Difference"
    
    the `.parse_file()` function gets a `BinaryIO`, a `TextIO` or a file path and parses it chunk by chunk; but `.parse()` function gets all document content at once. By this way, using `.parse_file()` is very better than `.parse()`.

Let's use them:

=== ".parse() function"

    ```python
    import markupever

    with open("index.html", "rb") as fd:
        dom = markupever.parse(fd.read(), markupever.HtmlOptions())
    ```

=== ".parse_file() function"

    ```python
    import markupever

    dom = markupever.parse_file("index.html", markupever.HtmlOptions())
    ```

!!! info "HtmlOptions"

    Let's see what options we have ...

    * `full_document` - Specifies that is this a complete document? default: True.
    * `exact_errors` - Report all parse errors described in the spec, at some performance penalty? default: False.
    * `discard_bom` - Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? default: False.
    * `profile` - Keep a record of how long we spent in each state? Printed when `finish()` is called. default: False.
    * `iframe_srcdoc` - Is this an `iframe srcdoc` document? default: False.
    * `drop_doctype` - Should we drop the DOCTYPE (if any) from the tree? default: False.
    * `quirks_mode` - Initial TreeBuilder quirks mode. default: `markupever.QUIRKS_MODE_OFF`.


That's it, we parsed **`index.html`** file and now have a `TreeDom` class. We can navigate that:

```python
root = dom.root() # Get root node
root
# Document

title = root.select_one("title") # Accepts CSS selectors
title.name
# QualName(local="title", ns="http://www.w3.org/1999/xhtml", prefix=None)

title.serialize()
# '<title>Incomplete Html</title>'

title.text()
# 'Incomplete Html'

title.parent.name
# QualName(local="head", ns="http://www.w3.org/1999/xhtml", prefix=None)

ul = root.select_one("ul")
ul.serialize()
# <ul>
#     <li><a href="https://www.example.com">Example Website</a></li>
#     <li><a href="https://www.wikipedia.org">Wikipedia</a></li>
#     <li><a href="https://www.bbc.com">BBC</a></li>
#     <li><a href="https://www.microsoft.com">Microsoft</a></li>
# </ul>
```

!!! tip "Common task"

    One common tasks is extracting all links from a page:
    ```python
    for tag in root.select("a[href^='https://']"):
        print(tag.attrs["href"])
    
    # https://www.example.com
    # https://www.wikipedia.org
    # https://www.bbc.com
    # https://www.microsoft.com
    ```

## Parsing XML

!!! info "XmlOptions"

    Let's see what options we have ...

    * `exact_errors` - Report all parse errors described in the spec, at some performance penalty? default: False.
    * `discard_bom` - Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? default: False.
    * `profile` - Keep a record of how long we spent in each state? Printed when `finish()` is called. default: False.

## Using Parser

## More about options
