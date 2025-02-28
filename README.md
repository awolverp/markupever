# markupever
![Untitled](https://github.com/user-attachments/assets/4fc58bbf-3fde-47a1-aa42-ae100ba1029a)

**"Low-Level" Target:**
```python
import markupever

dom = markupever.dom.TreeDom()

body: markupever.dom.Element = dom.create_element("body", {"class": "main"})
text: markupever.dom.Text = body.create_text("Body Text")
comment: markupever.dom.Comment = body.create_comment("Comment")
# other elements ...

dom.serialize(is_xml=False)
"""
<body class="main">
    Body Text
    <!-- Comment -->
</body>
"""
```

**"Parsing" Target:**
```python
import markupever

parser = markupever.parse("... content ...", markupever.HTMLOptions(...)) # or markupever.XMLOptions(...)
parser.errors # errors in content
parser.lineno # number of lines

dom: markupever.dom.DOMTree = parser.into_dom()

dom.select_one("h1.title")
```

**"Streaming" Target:**
```python
import markupever

parser = markupever.Parser(markupever.HTMLOptions(...))
parser.process("... content part 1 ...")
parser.process("... content part 2 ...")
parser.finish()

parser.errors # errors in content
parser.lineno # number of lines

dom: markupever.dom.DOMTree = parser.into_dom()

dom.select("h1.title")
```

**"\_rustlib" Target:**
```python
import markupever._rustlib as rl

dom = rl.TreeDom()

dom.root() # is rl.Document

element = rl.Element(dom, rl.QualName("body"), {"id": "hello"}) # orphan element on dom
dom.append(dom.root(), element) # append element as child of root

text = rl.Text(dom, "Hello World") # orphan text on dom
dom.append(element, text) # append text as child of element
```
