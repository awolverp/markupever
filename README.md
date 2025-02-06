# XMarkup

**"Low-Level" Target:**
```python
import xmarkup

dom = xmarkup.dom.TreeDom()

body: xmarkup.dom.Element = dom.create_element("body", {"class": "main"})
text: xmarkup.dom.Text = body.create_text("Body Text")
comment: xmarkup.dom.Comment = body.create_comment("Comment")
# other elements ...

dom.serialize("html")
"""
<body class="main">
    Body Text
    <!-- Comment -->
</body>
"""
```

**"Parsing" Target:**
```python
import xmarkup

parser = xmarkup.parse("... content ...", xmarkup.HTMLOptions(...)) # or xmarkup.XMLOptions(...)
parser.errors # errors in content
parser.lineno # number of lines

dom: xmarkup.dom.DOMTree = parser.dom

dom.select_one("h1.title")
```
