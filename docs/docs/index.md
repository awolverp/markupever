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


![text](https://img.shields.io/badge/coverage-100-08000)
![image](https://img.shields.io/pypi/v/markupever.svg)
![image](https://img.shields.io/pypi/l/markupever.svg)
![image](https://img.shields.io/pypi/pyversions/markupever.svg)
![python-test](https://github.com/awolverp/markupever/actions/workflows/test.yml/badge.svg)

MarkupEver is a modern, fast (high-performance), XML & HTML languages parsing library written in Rust.


<div class="grid cards" markdown>

-   :material-clock-fast:{ .lg } - __Fast__

    ---

    Very high performance and fast (thanks to [html5ever](https://github.com/servo/html5ever) and [selectors](https://github.com/servo/stylo/tree/main/selectors)).

    [Benchmarks :material-arrow-top-right:](#benchmarks)

-   :simple-greasyfork:{ .lg } - __Easy To Use__

    ---

    Designed to be easy to use and learn. <abbr title="also known as auto-complete, autocompletion, IntelliSense">Completion</abbr> everywhere.

    [Examples :material-arrow-top-right:](#examples)

-   :material-memory:{ .lg } - __Low Memory Usage__

    ---

    It boasts efficient memory usage, thanks to Rust's memory allocator, ensuring no memory leaks. 

    [Memory Usage :material-arrow-top-right:](#memory-usage)

-   :simple-css3:{ .lg .middle } - __Your CSS Knowledge__

    ---

    Leverage your **CSS** knowledge to select elements from HTML or XML documents effortlessly.

    [Querying :material-arrow-top-right:](querying.md)

</div>


## Installation
You can install MarkupEver using **pip**:

```console
$ pip3 install markupever
```

!!! tip "Use Virtual Environments"

    It's recommended to use virtual environments for installing and managing libraries in Python.

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


## Examples

!!! tip "More Examples"

    There are some good and basic examples to how to use `markupever` library.
    Even if you need more examples, see this page: [More Examples :material-arrow-top-right:](more-examples.md)

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

We want to extract the `href` attributes from it, and we have three ways to achieve this:

=== "Parse Content"

    You can parse HTML/XML content with `parse()` function.

    ```python title="main.py"
    import markupever
    with open("index.html", "rb") as fd: # (2)!
        dom = markupever.parse(fd.read(), markupever.HtmlOptions()) # (1)!

    for element in dom.select("a[href]"):
        print(element.attrs["href"])
    ```

    1.  Use `HtmlOptions()` for HTML documents and `XmlOptions()` for XML documents. If used incorrectly, don't worry—it won't disrupt the process. These options specify namespaces and other differences between XML and HTML, while also providing distinct features for each type.

    2.  It's recommended to open files with `"rb"` mode, but not required; you can use `"r"` mode also.

=== "Read From File"

    You can parse HTML/XML content from files with `.parse_file()` function.

    ```python title="main.py"
    import markupever
    dom = markupever.parse_file("index.html", markupever.HtmlOptions()) # (1)!
    
    for element in dom.select("a[href]"):
        print(element.attrs["href"])
    ```

    1.  Use `HtmlOptions()` for HTML documents and `XmlOptions()` for XML documents. If used incorrectly, don't worry—it won't disrupt the process. These options specify namespaces and other differences between XML and HTML, while also providing distinct features for each type.

=== "Use Parser Directly"

    The .parse() and .parse_file() functions are shorthand for using the .Parser class. However, you can also use the class directly. It's designed to allow you to stream input using the .process() method, so you don't have to worry about the memory usage of large inputs.

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

    1.  Use `HtmlOptions()` for HTML documents and `XmlOptions()` for XML documents. If used incorrectly, don't worry—it won't disrupt the process. These options specify namespaces and other differences between XML and HTML, while also providing distinct features for each type.

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

## Performance
This library is designed with a strong focus on performance and speed. It's written in **Rust** and avoids the use of unsafe code blocks.

I have compared **MarkupEver** with **BeautifulSoup** and **Parsel** (which directly uses `lxml`):

### Benchmarks

!!! info "System"

    The system on which the benchmarks are done: **Manjaro Linux x86_64, 8G, Intel i3-1115G4**

| Parsing                   |  Min   |  Max   |  Avg   |
|---------------------------|--------|--------|--------|
| markupever                | 4907µs | 4966µs | 4927µs |
| markupever (exact_errors) | 8920µs | 9172µs | 8971µs |
| beautifulsoup4 (html.parser)| 35283µs| 36460µs| 35828µs|
| beautifulsoup4 (lxml)     | 22576µs| 23092µs| 22809µs|
| parsel                    | 3937µs | 4147µs | 4072µs |
| html5lib (etree)          | 63214µs | 63844µs | 63489µs |
| html5lib (dom)            | 88111µs | 90721µs | 89580µs |

| Selecting (CSS)           |  Min   |  Max   |  Avg   |
|---------------------------|--------|--------|--------|
| markupever                | 308µs | 314µs | 310µs |
| beautifulsoup4            | 2936µs| 3074µs| 2995µs|
| parsel                    | 159µs | 165µs | 161µs |
| html5lib                  | N/A | N/A | N/A |

| Serializing               |  Min   |  Max   |  Avg   |
|---------------------------|--------|--------|--------|
| markupever                | 1932µs | 1973µs | 1952µs |
| beautifulsoup4            | 14705µs| 15021µs| 14900µs|
| parsel                    | 1264µs | 1290µs | 1276µs |
| html5lib                  | 17557µs | 18097µs | 17831µs |

!!! abstract "Summary"

    The **Parsel** is the fastest library (Actually `lxml` is) and is specially designed for scraping,
    but it offers less control over the document.
    The **html5lib** is the slowest library and does not support selecting elements by **CSS Selectors**.
    The **BeautifulSoup** library's speed is also slow which provides full control over the document.

    The **MarkupEver** sites between **BeautifulSoup** and **Parsel** library. It is extremely fast - close to Parsel, and offers full control over the document.

### Memory Usage
As you know, this library is written in Rust and uses the Rust allocator. Like other libraries written in **C** and other low-level languages, it uses very low memory, so you don't have to worry about memory usage. Manage huge documents with ease...

## License
This project is licensed under the terms of the MPL-2.0 license.
