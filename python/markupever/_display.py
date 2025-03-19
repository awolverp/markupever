import typing


class _DisplayCharacterToken:  # pragma: no cover
    """
    Internal token representing display characters for tree-like indentation visualization.

    Manages rendering of tree-like indentation characters based on sibling and children states,
    used for generating structured text representations with visual hierarchy.
    """

    __slots__ = ("siblings", "children")

    def __init__(self, siblings: bool):
        self.siblings = siblings
        self.children = False

    def __str__(self):
        # match (siblings, children)
        # (true, true) => "│   ",
        # (true, false) => "├── ",
        # (false, true) => "    ",
        # (false, false) => "└── ",

        if self.children:
            return "│   " if self.siblings else "    "

        if self.siblings:
            return "├── "

        return "└── "


class _Indentation:  # pragma: no cover
    """
    Internal class for managing tree-like indentation tokens during display rendering.

    Tracks display tokens with optional root ignoring, allowing dynamic indentation
    and deindentation while maintaining a visual hierarchy of tokens.
    """

    __slots__ = ("tokens", "ignore_root")

    def __init__(self, ignore_root: bool):
        self.tokens: typing.List[_DisplayCharacterToken] = []
        self.ignore_root = ignore_root

    def indent(self, siblings: bool):
        length = len(self.tokens)
        if length > 0:
            self.tokens[length - 1].children = True

        self.tokens.append(_DisplayCharacterToken(siblings))
        return self

    def deindent(self):
        self.tokens.pop()
        return self

    def __str__(self) -> str:
        s = ""
        for token in self.tokens[int(self.ignore_root) :]:
            s += str(token)

        return s
