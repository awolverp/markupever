from .parser import (
    Parser as Parser,
    parse as parse,
    parse_file as parse_file,
)
from ._rustlib import (
    HtmlOptions as HtmlOptions,
    XmlOptions as XmlOptions,
    QUIRKS_MODE_FULL as QUIRKS_MODE_FULL,
    QUIRKS_MODE_OFF as QUIRKS_MODE_OFF,
    QUIRKS_MODE_LIMITED as QUIRKS_MODE_LIMITED,
    __version__ as __version__,
    __author__ as __author__,
)
from . import dom as dom
from . import iterators as iterators
