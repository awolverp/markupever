---
title: More Examples
description: Parsing HTML/XML documents using markupever in Python
---

# More Examples
There's a collection of examples for markupever library.


!!! warning

    **This documentation is incomplete**. Documenting everything take a while.


### Using markupever alongside HTTP clients
How to use markupever alongside HTTP clients such as `httpx`, `requests` and `aiohttp`.

=== "httpx (traditional)"

    ```python
    with httpx.Client() as client:
        response = client.get("https://www.example.com/")
        dom = markupever.parse(response.content, markupever.HtmlOptions())
    ```

=== "httpx (recommended)"

    ```python
    with httpx.Client() as client:
        with client.stream(
            "GET",
            "https://www.example.com/",
        ) as stream:
            parser = markupever.Parser(markupever.HtmlOptions())

            for content in stream.iter_bytes():
                parser.process(content)
            
            dom = parser.finish().into_dom()
    ```

=== "requests"

    ```python
    response = requests.get("https://www.example.com/")
    dom = markupever.parse(response.content, markupever.HtmlOptions())
    ```

=== "aiohttp"

    ```python
    async with aiohttp.ClientSession() as session:
        async with session.get('https://www.google.com/') as resp:
            dom = markupever.parse(await resp.read(), markupever.HtmlOptions())
    ```
