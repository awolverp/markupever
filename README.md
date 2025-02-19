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

dom: xmarkup.dom.DOMTree = parser.into_dom()

dom.select_one("h1.title")
```

**"Streaming" Target:**
```python
import xmarkup

parser = xmarkup.Parser(xmarkup.HTMLOptions(...))
parser.process("... content part 1 ...")
parser.process("... content part 2 ...")
parser.finish()

parser.errors # errors in content
parser.lineno # number of lines

dom: xmarkup.dom.DOMTree = parser.into_dom()

dom.select("h1.title")
```

**"\_rustlib" Target:**
```python
import xmarkup._rustlib as rl

dom = rl.TreeDom()

dom.root() # is rl.Document

element = rl.Element(dom, rl.QualName("body"), {"id": "hello"}) # orphan element on dom
dom.append(dom.root(), element) # append element as child of root

text = rl.Text(dom, "Hello World") # orphan text on dom
dom.append(element, text) # append text as child of element
```
