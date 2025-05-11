<p align="center">
  <img src="https://github.com/user-attachments/assets/4fc58bbf-3fde-47a1-aa42-ae100ba1029a" alt="MarkupEver">
</p>
<p align="center">
    <em>The fast, most optimal, and correct HTML & XML parsing library</em>
</p>
<p align="center">
    <a href="https://awolverp.github.io/markupever" target="_blank"><b>Documentation</b></a> | <a href="https://github.com/awolverp/cachebox/releases"><b>Releases</b></a> | <a href="https://awolverp.github.io/markupever/#performance" target="_blank"><b>Benchmarks</b></a>
</p>

![text](https://img.shields.io/badge/coverage-100-08000)
![image](https://img.shields.io/pypi/v/markupever.svg)
![image](https://img.shields.io/pypi/l/markupever.svg)
![image](https://img.shields.io/pypi/pyversions/markupever.svg)
![python-test](https://github.com/awolverp/markupever/actions/workflows/test.yml/badge.svg)
![download](https://img.shields.io/pypi/dm/markupever?style=flat-square&color=%23314bb5)

------

MarkupEver is a modern, fast (high-performance), XML & HTML languages parsing library written in Rust.

**KEY FEATURES:**
* 🚀 **Fast**: Very high performance and fast (thanks to **[html5ever](https://github.com/servo/html5ever)** and **[selectors](https://github.com/servo/stylo/tree/main/selectors)**).
* 🔥 **Easy**: Designed to be easy to use and learn. <abbr title="also known as auto-complete, autocompletion, IntelliSense">Completion</abbr> everywhere.
* ✨ **Low-Memory**: Written in Rust. Uses low memory. Don't worry about memory leaks. Uses Rust memory allocator.
* 🧶 **Thread-safe**: Completely thread-safe. 
* 🎯 **Quering**: Use your **CSS** knowledge for selecting elements from a HTML or XML document.

## Installation
You can install MarkupEver by using **pip**:

<small>It's recommended to use virtual environments.</small>

```console
$ pip3 install markupever
```

## Example

### Parse
Parsing a HTML content and selecting elements:

```python
import markupever

dom = markupever.parse_file("file.html", markupever.HtmlOptions())
# Or parse a HTML content directly:
# dom = markupever.parse("... content ...", markupever.HtmlOptions())

for element in dom.select("div.section > p:child-nth(1)"):
    print(element.text())
```

### Create DOM
Creating a DOM from zero:

```python
from markupever import dom

dom = dom.TreeDom()
root: dom.Document = dom.root()

root.create_doctype("html")

html = root.create_element("html", {"lang": "en"})
body = html.create_element("body")
body.create_text("Hello Everyone ...")

print(root.serialize())
# <!DOCTYPE html>
# <html lang="en">
#   <body>Hello Everyone ...</body>
# </html>
```
