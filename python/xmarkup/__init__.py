from .parser import (
    Parser as Parser,
    parse as parse,
    parse_file as parse_file,
)
from ._rustlib import (
    HtmlOptions as HtmlOptions,
    XmlOptions as XmlOptions,
    QualName as QualName,
    QUIRKS_MODE_FULL as QUIRKS_MODE_FULL,
    QUIRKS_MODE_OFF as QUIRKS_MODE_OFF,
    QUIRKS_MODE_LIMITED as QUIRKS_MODE_LIMITED,
)
from . import dom as dom
