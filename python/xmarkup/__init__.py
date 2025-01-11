from ._rustlib import (
    __version__ as __version__,
    __author__ as __author__,
    QUIRKS_MODE_FULL as QUIRKS_MODE_FULL,
    QUIRKS_MODE_LIMITED as QUIRKS_MODE_LIMITED,
    QUIRKS_MODE_OFF as QUIRKS_MODE_OFF,
)
from .driver import (
    HtmlOptions as HtmlOptions,
    XmlOptions as XmlOptions,
    Html as Html,
    Xml as Xml,
)
from . import nodes as nodes
