---
title: Parsing Usage
description: Parsing HTML/XML documents using markupever in Python
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
* You can specify some **options** for parsing which can help you (with `HtmlOptions()` and `XmlOptions()` classes).
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
    <ul>
        <li><a href="https://www.example.com">Example Website</a></li>
        <li><a href="https://www.wikipedia.org">Wikipedia</a></li>
        <li><a href="https://www.bbc.com">BBC</a></li>
        <li><a href="https://www.microsoft.com">Microsoft</a></li>
    </ul>
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

    See [HtmlOptions parameters](#htmloptions-parameters).


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

**Additionaly**, if you serialize the parsed DOM you'll see that the incomplete HTML is repaired:
```python hl_lines="12"
root.serialize()
# <!DOCTYPE html><html><head>
#     <title>Incomplete Html</title>
# </head>
# <body>
#     <ul>
#         <li><a href="https://www.example.com">Example Website</a></li>
#         <li><a href="https://www.wikipedia.org">Wikipedia</a></li>
#         <li><a href="https://www.bbc.com">BBC</a></li>
#         <li><a href="https://www.microsoft.com">Microsoft</a></li>
#     </ul>
# </body></html>
```

## Parsing XML
Imagine this **`file.xml`** file:

```xml title="file.xml"
<?xml version="1.0" encoding="UTF-8"?>
<bookstore xmlns:bk="http://www.example.com/books" xmlns:mag="http://www.example.com/magazines">
  <bk:book>
    <bk:title>Programming for Beginners</bk:title>
    <bk:author>Jane Doe</bk:author>
    <bk:year>2021</bk:year>
  </bk:book>
  <mag:magazine>
    <mag:title>Technology Monthly</mag:title>
    <mag:publisher>Tech Publishers</mag:publisher>
    <mag:month>March</mag:month>
  </mag:magazine>
</bookstore>
```

Let's use `.parse()` / `.parse_file()` function to parse it (we explained them [earlier](#parsing-html)):

=== ".parse() function"

    ```python
    import markupever

    with open("file.xml", "rb") as fd:
        dom = markupever.parse(fd.read(), markupever.XmlOptions())
    ```

=== ".parse_file() function"

    ```python
    import markupever

    dom = markupever.parse_file("file.xml", markupever.XmlOptions())
    ```

!!! info "XmlOptions"

    See [XmlOptions parameters](#xmloptions-parameters).

That's it, we parsed **`file.xml`** file and now have a `TreeDom` class. We can navigate that like what we did in this [section](#parsing-html):

```python
root = dom.root() # Get root node
root
# Document

root.select_one("bookstore")
# Element(name=QualName(local="bookstore"), attrs=[], template=false, mathml_annotation_xml_integration_point=false)

for i in root.select("mag|*"): # get all elements which has namespace 'mag'
    print(i)
# Element(name=QualName(local="magazine", ns="http://www.example.com/magazines", prefix=Some("mag")), attrs=[], template=false, mathml_annotation_xml_integration_point=false)
# Element(name=QualName(local="title", ns="http://www.example.com/magazines", prefix=Some("mag")), attrs=[], template=false, mathml_annotation_xml_integration_point=false)
# Element(name=QualName(local="publisher", ns="http://www.example.com/magazines", prefix=Some("mag")), attrs=[], template=false, mathml_annotation_xml_integration_point=false)
# Element(name=QualName(local="month", ns="http://www.example.com/magazines", prefix=Some("mag")), attrs=[], template=false, mathml_annotation_xml_integration_point=false)

book = root.select_one("book")
book.serialize()
# <bk:book xmlns:bk="http://www.example.com/books">
#   <bk:title>Programming for Beginners</bk:title>
#   <bk:author>Jane Doe</bk:author>
#   <bk:year>2021</bk:year>
# </bk:book>
```

## Using Parser
The functions `.parse()` and `.parse_file()`, which you became familiar with earlier, internally use `Parser` class
which actually does the parsing. In this part we want to learn the `Parser` class.

The `Parser` class is an HTML/XML parser, ready to receive Unicode input. It is very easy to use and allows you to stream input using the `.process()` method. This way, you don't have to worry about the memory usage of large inputs.

As we said about *options* parameter in `.parse()` and `.parse_file()`,
if your input is an HTML document, pass a `HtmlOptions()`; if your input is an XML document, pass `XmlOptions()`

To start, create an instance of the `Parser` class. Then, use the `Parser.process()` method to send content for parsing. You can call this method as many times as you want (it's thread-safe). When your inputs are finished, call the `Parser.finish()` method to mark the parser as finished.

```python
import markupever

# Create Parser
parser = markupever.Parser(options=markupever.HtmlOptions())

# Process contents
parser.process("... content 1 ...")
parser.process("... content 2 ...")
parser.process("... content 3 ...")

# Mark as finished
parser.finish()
```

That's it! Your HTML document parsing is now finished and complete. The Parser class has several methods and attributes to inform you about the parsed content, such as the `lineno` property, `quirks_mode` property, and `errors()` method. You can see examples:

=== "`lineno` property"

    Returns the line count of the parsed content (always is `1` for XML).

    ```python
    print(parser.lineno)
    # 56
    ```

=== "`quirks_mode` property"

    Returns the quirks mode (always is QUIRKS_MODE_OFF for XML).
        
    See quirks mode on [wikipedia](https://en.wikipedia.org/wiki/Quirks_mode) for more information.

    ```python
    print(parser.quirks_mode)
    # 2
    ```

=== "`errors()` method"

    Returns the errors which are detected while parsing.

    ```python
    print(parser.errors())
    # ['Unexpected token']
    ```

You can use these properties and methods before calling the `Parser.into_dom()` method. The `Parser.into_dom()` method converts the parser into a `TreeDom` and releases its allocated memory.

```python hl_lines="10"
import markupever
parser = markupever.Parser(options=markupever.HtmlOptions())
parser.process("... content 1 ...")
parser.process("... content 2 ...")
parser.process("... content 3 ...")
parser.finish()

# Use `.errors()`, `.lineno`, or `.quirks_mode` if you want

dom = parser.into_dom()
```

## More about options
We have two structures for parsing options: `HtmlOptions()` and `XmlOptions()`.

Use `HtmlOptions()` for HTML documents and `XmlOptions()` for XML documents. If used incorrectly, don't worry — it won't disrupt the process. These options specify namespaces and other differences between XML and HTML, while also providing distinct features for each type.

### HtmlOptions parameters

Let's see what parameters we have:

* **`full_document`** - Specifies that is this a complete document? default: True.

=== "True"

    ```python hl_lines="5"
    import markupever
    
    dom = markupever.parse("<p>A Text</p>", markupever.HtmlOptions(full_document=True))
    dom.serialize()
    # <html><head></head><body><p>A Text</p></body></html>
    ```

=== "False"

    ```python hl_lines="5"
    import markupever
    
    dom = markupever.parse("<p>A Text</p>", markupever.HtmlOptions(full_document=False))
    dom.serialize()
    # <html><p>A Text</p></html>
    ```

* **`exact_errors`** - Report all parse errors described in the spec, at some performance penalty? default: False.

=== "True"

    ```python hl_lines="6"
    import markupever
    p = markupever.Parser(markupever.HtmlOptions(exact_errors=True))
    p.process("<p>A Text</p>")
    p.finish()
    p.errors()
    # ["Unexpected token TagToken(Tag { kind: StartTag, name: Atom(\\'p\\' type=inline), self_closing: false, attrs: [] }) in insertion mode Initial"]
    ```

=== "False"

    ```python hl_lines="6"
    import markupever
    p = markupever.Parser(markupever.HtmlOptions(exact_errors=False))
    p.process("<p>A Text</p>")
    p.finish()
    p.errors()
    # ["Unexpected token"]
    ```

* **`discard_bom`** - Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? default: False.

* **`profile`** - Keep a record of how long we spent in each state? Printed when `finish()` is called. default: False.

=== "True"

    ```python
    import markupever
    
    markupever.parse("<p>A Text</p>", markupever.HtmlOptions(profile=True))
    #
    # Tokenizer profile, in nanoseconds
    #
    #    93331         total in token sink
    #
    #    46121         total in tokenizer
    #    17651  38.3%  Data
    #    13640  29.6%  TagName
    #    11768  25.5%  TagOpen
    #     3062   6.6%  EndTagOpen
    ```

=== "False"

    ```python
    import markupever
    
    markupever.parse("<p>A Text</p>", markupever.HtmlOptions(profile=False))
    ```

* **`iframe_srcdoc`** - Is this an `iframe srcdoc` document? default: False.

* **`drop_doctype`** - Should we drop the DOCTYPE (if any) from the tree? default: False.

=== "True"

    ```python hl_lines="5"
    import markupever
    
    dom = markupever.parse("<!DOCTYPE html><p>A Text</p>", markupever.HtmlOptions(drop_doctype=True))
    dom.serialize()
    # <html><head></head><body><p>A Text</p></body></html>
    ```

=== "False"

    ```python hl_lines="5"
    import markupever
    
    dom = markupever.parse("<!DOCTYPE html><p>A Text</p>", markupever.HtmlOptions(drop_doctype=False))
    dom.serialize()
    # <!DOCTYPE html><html><head></head><body><p>A Text</p></body></html>
    ```

* **`quirks_mode`** - Initial TreeBuilder quirks mode. default: `markupever.QUIRKS_MODE_OFF`.


### XmlOptions parameters

Let's see what parameters we have:

* **`exact_errors`** - Report all parse errors described in the spec, at some performance penalty? default: False.

=== "True"

    ```python hl_lines="6"
    import markupever
    p = markupever.Parser(markupever.XmlOptions(exact_errors=True))
    p.process("<p>A Text</p>")
    p.finish()
    p.errors()
    # ["Unexpected token TagToken(Tag { kind: StartTag, name: Atom(\\'p\\' type=inline), self_closing: false, attrs: [] }) in insertion mode Initial"]
    ```

=== "False"

    ```python hl_lines="6"
    import markupever
    p = markupever.Parser(markupever.XmlOptions(exact_errors=False))
    p.process("<p>A Text</p>")
    p.finish()
    p.errors()
    # ["Unexpected token"]
    ```

* **`discard_bom`** - Discard a `U+FEFF BYTE ORDER MARK` if we see one at the beginning of the stream? default: False.

* **`profile`** - Keep a record of how long we spent in each state? Printed when `finish()` is called. default: False.

=== "True"

    ```python
    import markupever
    
    markupever.parse("<p>A Text</p>", markupever.XmlOptions(profile=True))
    #
    # Tokenizer profile, in nanoseconds
    #
    #    93331         total in token sink
    #
    #    46121         total in tokenizer
    #    17651  38.3%  Data
    #    13640  29.6%  TagName
    #    11768  25.5%  TagOpen
    #     3062   6.6%  EndTagOpen
    ```

=== "False"

    ```python
    import markupever
    
    markupever.parse("<p>A Text</p>", markupever.XmlOptions(profile=False))
    ```
