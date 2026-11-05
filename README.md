<div align="center">

<img src="https://github.com/user-attachments/assets/75ca729c-1eb0-405e-be03-f8a7cea5cba2" alt="MarkupEver">

_MarkupEver is a modern, fast (high-performance), XML & HTML languages parsing library written in Rust._

[![badge](https://shieldcn.dev/badge/Documentation-ff6e42.svg?logo=materialformkdocs)](https://awolverp.github.io/markupever/) [![badge](https://shieldcn.dev/badge/Benchmarks-ff6e42.svg?logo=ri%3ATbBrandSpeedtest)](https://awolverp.github.io/markupever/#performance)

[![PyPi](https://shieldcn.dev/pypi/markupever.svg?variant=branded&font=geist-mono&size=xs)](https://pypi.org/project/markupever)
[![Monthly Downloads](https://shieldcn.dev/pypi/dm/markupever.svg?variant=branded&font=geist-mono&size=xs)](https://pypi.org/project/markupever)
[![Python Version](https://shieldcn.dev/pypi/python/markupever.svg?variant=branded&font=geist-mono&size=xs)](https://pypi.org/project/markupever)

<a href="https://github.com/awolverp/markupever/actions?query=branch%3Amain"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/ci/awolverp/markupever.svg?variant=outline&font=geist-mono&size=xs&animate=pulse&mode=dark"><img alt="CI" src="https://shieldcn.dev/github/ci/awolverp/markupever.svg?variant=outline&font=geist-mono&size=xs&animate=pulse&mode=light"></picture></a>
<a href="https://github.com/awolverp/markupever/commits/main"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/last-commit/awolverp/markupever.svg?variant=outline&font=geist-mono&size=xs&mode=dark"><img alt="Last Commit" src="https://shieldcn.dev/github/last-commit/awolverp/markupever.svg?variant=outline&font=geist-mono&size=xs&mode=light"></picture></a>
<a href="https://github.com/awolverp/markupever/blob/main/LICENSE"><picture><source media="(prefers-color-scheme: dark)" srcset="https://shieldcn.dev/github/awolverp/markupever/license.svg?variant=outline&font=geist-mono&size=xs&mode=dark"><img alt="License" src="https://shieldcn.dev/github/awolverp/markupever/license.svg?variant=outline&font=geist-mono&size=xs&mode=light"></picture></a>

</div>

---

**KEY FEATURES:**

- 🚀 **Fast**: Very high performance and fast (thanks to **[html5ever](https://github.com/servo/html5ever)** and **[selectors](https://github.com/servo/stylo/tree/main/selectors)**).
- 🔥 **Easy**: Designed to be easy to use and learn. <abbr title="also known as auto-complete, autocompletion, IntelliSense">Completion</abbr> everywhere.
- ✨ **Low-Memory**: Written in Rust. Uses low memory. Don't worry about memory leaks. Uses Rust memory allocator.
- 🧶 **Thread-safe**: Completely thread-safe.
- 🎯 **Quering**: Use your **CSS** knowledge for selecting elements from a HTML or XML document.
- ⚡ **Streaming**: Incremental/streaming parsing support.

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

dom = markupever.parse_file("file.html", "html")
# Or parse a HTML content directly:
# dom = markupever.parse("... content ...", "html")

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
# <!DOCTYPE html><html lang="en"><body>Hello Everyone ...</body></html>
```
