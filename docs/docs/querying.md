---
title: CSS Selectors
description: Selecting elements using CSS selectors using markupever in Python
---

# CSS Selectors
The **MarkupEver** supports all the syntax that [soupsieve](https://facelessuser.github.io/soupsieve/selectors/basic/) does, except for some pseudo-elements (I prefer not to document all these syntaxes again).

## Some Examples
To start, imagine this document:

```python
import markupever

tree = markupever.parse(
    """
    <html>
    <head>
        <title>Example Document</title>
    </head>
    <body>
        <p class="par-one">CSS Selector Example</p>
        <p class="par-two">I wish you a good day</p>
        <p class="end-par">I wish you a good day</p>
    </body>
    </html>
    """,
    markupever.HtmlOptions()
)
```

Let's see some examples:

```python
print(tree.select_one("head > title").text())
# Example Document

for element in tree.select("[class^=par-]"):
    print(element)
# Element(name=QualName(local="p", ns="http://www.w3.org/1999/xhtml", prefix=None), attrs=[(QualName(local="class"), "par-one")], template=false, integration_point=false)
# Element(name=QualName(local="p", ns="http://www.w3.org/1999/xhtml", prefix=None), attrs=[(QualName(local="class"), "par-two")], template=false, integration_point=false)

print(tree.select_one("p", offset=3))
# Element(name=QualName(local="p", ns="http://www.w3.org/1999/xhtml", prefix=None), attrs=[(QualName(local="class"), "end-par")], template=false, integration_point=false)

for element in tree.select("[class*=par]", offset=3, limit=1):
    print(element)
# Element(name=QualName(local="p", ns="http://www.w3.org/1999/xhtml", prefix=None), attrs=[(QualName(local="class"), "end-par")], template=false, integration_point=false)
```


