<p align="center">
  <img src="https://github.com/user-attachments/assets/4fc58bbf-3fde-47a1-aa42-ae100ba1029a" alt="MarkupEver">
</p>
<p align="center">
    <em>The fast, most optimal, and correct HTML & XML parsing library</em>
</p>


---

**DOCUMENTATION**: <a href="https://awolverp.github.io/markupever" target="_blank">https://awolverp.github.io/markupever</a>

**SOURCE CODE**: <a href="https://github.com/awolverp/markupever" target="_blank">https://github.com/awolverp/markupever</a>

---

> [!WARNING]\
> This project is still in development and might be unstable or incomplete. Please use it with caution. Any feedback or contributions are greatly appreciated.

MarkupEver is a modern, fast (high-performance), XML & HTML languages parsing library written in Rust.

**KEY FEATURES:**
* 🚀 **Fast**: Very high performance and fast (thanks to **[html5ever](https://github.com/servo/html5ever)** and **[selectors](https://github.com/servo/stylo/tree/main/selectors)**). <u>About 20x faster than BeautifulSoup and Parsel.</u>
* 🔥 **Easy**: Designed to be easy to use and learn. <abbr title="also known as auto-complete, autocompletion, IntelliSense">Completion</abbr> everywhere.
* ✨ **Low-Memory**: Written in Rust. Uses low memory. Don't worry about memory leaks. Uses Rust memory allocator.
* 🧶 **Thread-safe**: Completely thread-safe. 
* 🎯 **Quering**: Use your **CSS** knowledge for selecting elements from a HTML or XML document.

> [!NOTE]\
> ❤️ I ask for your support to continue on this path and make this Python library better and better ...
> - Star this repository
> - Tell me in issues your ideas and questions

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
import markupever as mr

dom = mr.parse_file("file.html", mr.HtmlOptions())
# Or parse a HTML content directly:
# dom = markupever.parse("... content ...", mr.HtmlOptions())

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

# TODO List
- [x] Rewrite TreeDom `__repr__` and `__str__`
- [ ] Add benchmarks
- [ ] Add memory usage report
- [ ] Add PyPI version, downloads, test coverage, and python versions badges
- [ ] Complete docs
- [ ] Add prettier feature
- [ ] Provide more control on serializer
- [ ] Add advanced examples to docs (such as socket and http streams)
