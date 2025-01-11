import xmarkup

html_doc = """<tag xmlns:ns1="http://namespace1/" xmlns:ns2="http://namespace2/">
 <ns1:child>I'm in namespace 1</ns1:child>
 <ns2:child>I'm in namespace 2</ns2:child>
</tag> """

p = xmarkup.Xml(html_doc)
print(p.select("ns1|child"))
