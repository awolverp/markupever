---
title: Home
description: The fast, most optimal, and correct HTML & XML parsing library
---

#

<p align="center">
  <img src="logo.png" alt="MarkupEver">
</p>
<p align="center">
    <em>The fast, most optimal, and correct HTML & XML parsing library</em>
</p>


---

**DOCUMENTATION**: <a href="https://awolverp.github.io/markupever" target="_blank">https://awolverp.github.io/markupever</a>

**SOURCE CODE**: <a href="https://github.com/awolverp/markupever" target="_blank">https://github.com/awolverp/markupever</a>

---

MarkupEver is a modern, fast (high-performance), XML & HTML languages parsing library written in Rust.


<div class="grid cards" markdown>

-   :material-clock-fast:{ .lg } - __Fast__

    ---

    Very high performance and fast (thanks to [html5ever](https://github.com/servo/html5ever) and [selectors](https://github.com/servo/stylo/tree/main/selectors)). **About 20x faster than BeautifulSoup and Parsel.**

    [Benchmarks :material-arrow-top-right:](#benchmarks)

-   :simple-greasyfork:{ .lg } - __Easy To Use__

    ---

    Designed to be easy to use and learn. <abbr title="also known as auto-complete, autocompletion, IntelliSense">Completion</abbr> everywhere.

    [Examples :material-arrow-top-right:](#examples)

-   :material-memory:{ .lg } - __Low Memory Usage__

    ---

    Written in Rust. Uses low memory. Don't worry about memory leaks. Uses Rust memory allocator.

    [Memory Usage :material-arrow-top-right:](#memory-usage)

-   :simple-css3:{ .lg .middle } - __Your CSS Knowledge__

    ---

    Use your **CSS** knowledge for selecting elements from a HTML or XML document.

    [Querying :material-arrow-top-right:](querying.md)

</div>


!!! note annotate "Support"

    I ask for your support to continue on this path and make this Python library better and better (1)

1.  Star github repository and tell me in issues your ideas and questions
    

## Examples

### Parsing & Scraping
Parsing a HTML content and selecting elements:

Imagine this **`index.html`** file:

```html title="index.html"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Example Document</title>
</head>
<body>
    <h1 id="title">Welcome to My Page</h1>
    <p>This page has a link and an image.</p>
    <a href="https://www.example.com">Visit Example.com</a>
    <br>
    <img src="https://www.example.com/image.jpg" alt="My Image">
    <a href="https://www.google.com">Visit Google</a>
    <a>No Link</a>
</body>
</html>
```

We want to extract `href` attributes from this file - We have 3 ways:

=== "Parse Content"

    You can parse HTML/XML content with `parse()` function.

    ```python title="main.py"
    import markupever
    with open("index.html", "rb") as fd: # (2)!
        dom = markupever.parse(fd.read(), markupever.HtmlOptions()) # (1)!

    for element in dom.select("a[href]"):
        print(element.attrs["href"])
    ```

    1.  Use `HtmlOptions()` for HTML documents and `XmlOptions()` for XML documents. If you used it incorrectly, don't worry; it won't disrupt the process. These options specify namespaces and such differences between XML and HTML. Additionally, each provides you with different features.

    2.  It's recommended to open files with `"rb"` mode, but not required; you can use `"r"` mode also.

=== "Read From File"

    You can parse HTML/XML content from files with `.parse_file()` function.

    ```python title="main.py"
    import markupever
    dom = markupever.parse_file("index.html", markupever.HtmlOptions()) # (1)!
    
    for element in dom.select("a[href]"):
        print(element.attrs["href"])
    ```

    1.  Use `HtmlOptions()` for HTML documents and `XmlOptions()` for XML documents. If you used it incorrectly, don't worry; it won't disrupt the process. These options specify namespaces and such differences between XML and HTML. Additionally, each provides you with different features.

=== "Use Parser Directly"

    These `.parse()` and `.parse_file()` function is a shorthand for using `.Parser` class.
    But you can use it directly. It's designed to allow you to stream input using `.process()` method; By this way
    you are don't worry about memory usages of huge inputs.

    ```python title="main.py"
    import markupever
    parser = markupever.Parser(markupever.HtmlOptions()) # (1)!

    with open("index.html", "rb") as fd: # (2)!
        for line in fd: # Read line by line (3)
            parser.process(line)
    
    parser.finish()
    dom = parser.into_dom()

    for element in dom.select("a[href]"):
        print(element.attrs["href"])
    ```

    1.  Use `HtmlOptions()` for HTML documents and `XmlOptions()` for XML documents. If you used it incorrectly, don't worry; it won't disrupt the process. These options specify namespaces and such differences between XML and HTML. Additionally, each provides you with different features.

    2.  It's recommended to open files with `"rb"` mode, but not required; you can use `"r"` mode also.

    3.  You can read the file all at once and pass it to the `process` function. We have broken the file into lines here to show you the `Parser`'s abilities.

Then run **`main.py`** to see result:

```console
$ python3 main.py
https://www.example.com
https://www.google.com
```

### Creating Documents
Also there's a structure called `TreeDom` (1). You can directly work with it and generate documents (such as HTML and XML) very easy.
{ .annotate }

1. A tree structure which specialy designed for HTML and XML documents. Uses Rust's `Vec` type in backend.
    The memory consumed by the `TreeDom` is dynamic and depends on the number of tokens stored in the tree.
    The allocated memory is never reduced and is only released when it is dropped.

```python
from markupever import dom

dom = dom.TreeDom()
root: dom.Document = dom.root()

root.create_doctype("html")

html = root.create_element("html", {"lang": "en"})
body = html.create_element("body")
body.create_text("Hello Everyone ...")

print(root.serialize())
# <!DOCTYPE html><html lang="en"><body>Hello Everyone ...</body></html>
```


## Installation
You can install MarkupEver by using **pip**:

```console
$ pip3 install markupever
```

!!! tip "Use Virtual Environments"

    It is recommended to use virtual environments for installing and using libraries in Python.

    === "Linux (venv)"

        ```console
        $ python3 -m venv venv
        $ source venv/bin/activate
        ```
    
    === "Linux (virtualenv)"

        ```console
        $ virtualenv venv
        $ source venv/bin/activate
        ```

    === "Windows (venv)"

        ```cmd
        $ python3 -m venv venv
        $ venv\Scripts\activate
        ```
    
    === "Windows (virtualenv)"

        ```cmd
        $ virtualenv venv
        $ venv\Scripts\activate
        ```


## Performance
This library is designed by focusing on performance and speed. It's written in **Rust** and avoids unsafe code blocks.

#### Benchmarks
Comming Soon ...

#### Memory Usage
Comming Soon ...

## License
This project is licensed under the terms of the MPL-2.0 license.
