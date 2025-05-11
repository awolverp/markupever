---
title: More Examples
description: Parsing HTML/XML documents using markupever in Python
---

# More Examples
There's a collection of examples for markupever library.


!!! warning

    **This documentation is incomplete**. Documenting everything take a while.


### Using markupever alongside HTTPX
How to use markupever alongside `httpx` library.

=== "httpx (traditional)"

    ```python
    import markupever
    import httpx

    # Create a Client instance
    with httpx.Client() as client:
        # Send a GET request to google
        response = client.get("https://www.example.com/")

        # Parse the result using markupever
        dom = markupever.parse(response.content, markupever.HtmlOptions())
    ```

=== "httpx (recommended)"

    ```python
    import markupever
    import httpx

    # Create a Client instance
    with httpx.Client() as client:
        # Stream a GET request to google
        with client.stream(
            "GET",
            "https://www.example.com/",
        ) as stream:
            # Parse the result using markupever
            parser = markupever.Parser(markupever.HtmlOptions())

            for content in stream.iter_bytes():
                parser.process(content)
            
            dom = parser.finish().into_dom()
    ```

### Using markupever alongside Requests
How to use markupever alongside `requests` library.

=== "requests"

    ```python
    import markupever
    import requests

    # Send a GET request to google
    response = requests.get("https://www.example.com/")

    # Parse the result using markupever
    dom = markupever.parse(response.content, markupever.HtmlOptions())
    ```

### Using markupever alongside AIOHttp
How to use markupever alongside `aiohttp` library.

=== "aiohttp"

    ```python
    # Create a ClientSession instance
    async with aiohttp.ClientSession() as session:
        # Send a GET request to google
        async with session.get('https://www.google.com/') as resp:
            # Parse the result using markupever
            dom = markupever.parse(await resp.read(), markupever.HtmlOptions())
    ```

### Using markupever alongside PycURL
How to use markupever alongside `PycURL` library.

=== "pycurl (recommended & easy)"

    ```python
    import pycurl
    import certifi
    from io import BytesIO

    # Create a PycURL instance
    c = pycurl.Curl()

    # Define Options ...
    c.setopt(c.URL, 'https://www.google.com/')
    c.setopt(c.CAINFO, certifi.where())

    # Setup markupever to recieve response
    parser = markupever.Parser()
    c.setopt(c.WRITEDATA, parser)

    # Send Request
    c.perform()

    # Close Connection
    c.close()

    # Use the parsed DOM
    dom = parser.finish().into_dom()
    ```
