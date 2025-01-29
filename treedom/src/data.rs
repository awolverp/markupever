use crate::atomic::{make_atomic_tendril, AtomicTendril, OnceLock};
use tendril::StrTendril;

/// The root of HTML document
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Document;

/// the doctype is the required <!doctype html> preamble found at the top of all documents.
/// Its sole purpose is to prevent a browser from switching into so-called "quirks mode"
/// when rendering a document; that is, the <!doctype html> doctype ensures that the browser makes
/// a best-effort attempt at following the relevant specifications, rather than using a different
/// rendering mode that is incompatible with some specifications.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Doctype {
    pub name: AtomicTendril,
    pub public_id: AtomicTendril,
    pub system_id: AtomicTendril,
}

impl Doctype {
    /// Create a new `Doctype`
    #[inline]
    pub fn new(name: AtomicTendril, public_id: AtomicTendril, system_id: AtomicTendril) -> Self {
        Self {
            name,
            public_id,
            system_id,
        }
    }

    /// Create a new `Doctype` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(name: StrTendril, public_id: StrTendril, system_id: StrTendril) -> Self {
        Self::new(
            make_atomic_tendril(name),
            make_atomic_tendril(public_id),
            make_atomic_tendril(system_id),
        )
    }
}

/// The Comment interface represents textual notations within markup; although it is generally not
/// visually shown, such comments are available to be read in the source view.
///
/// Comments are represented in HTML and XML as content between <!-- and -->. In XML,
/// like inside SVG or MathML markup, the character sequence -- cannot be used within a comment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment {
    pub contents: AtomicTendril,
}

impl Comment {
    /// Create a new `Comment`
    #[inline]
    pub fn new(contents: AtomicTendril) -> Self {
        Self { contents }
    }

    /// Create a new `Comment` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(contents: StrTendril) -> Self {
        Self::new(make_atomic_tendril(contents))
    }
}

/// A text
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Text {
    pub contents: AtomicTendril,
}

impl Text {
    /// Create a new `Text`
    #[inline]
    pub fn new(contents: AtomicTendril) -> Self {
        Self { contents }
    }

    /// Create a new `Text` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(contents: StrTendril) -> Self {
        Self::new(make_atomic_tendril(contents))
    }

    /// Push another StrTendril onto the end of this one.
    #[inline]
    pub fn push_non_atomic(&mut self, contents: StrTendril) {
        self.contents.push_tendril(&make_atomic_tendril(contents));
    }
}

/// Element attributes that caches 'id' and 'class' attributes of element
/// and also triggers update will removes caches when attributes updated
#[derive(Clone)]
pub struct ElementAttributeTrigger {
    item: Vec<(markup5ever::QualName, AtomicTendril)>,
    id: OnceLock<Option<AtomicTendril>>,
    classes: OnceLock<Vec<markup5ever::LocalName>>,
}

impl ElementAttributeTrigger {
    /// Creates a new `ElementAttributeTrigger`
    #[inline]
    pub fn new<I>(item: I) -> Self
    where
        I: Iterator<Item = (markup5ever::QualName, AtomicTendril)>,
    {
        Self {
            item: item.collect(),
            id: OnceLock::new(),
            classes: OnceLock::new(),
        }
    }

    /// Finds, caches, and returns the 'id' attribute from attributes.
    #[inline]
    pub fn id(&self) -> Option<&str> {
        self.id
            .get_or_init(|| {
                self.item
                    .iter()
                    .find(|(name, _)| &name.local == "id")
                    .map(|(_, value)| value.clone())
            })
            .as_deref()
    }

    /// Finds, caches, and returns the 'class' attributes from attributes.
    #[inline]
    pub fn classes(&self) -> std::slice::Iter<'_, markup5ever::LocalName> {
        let classes = self.classes.get_or_init(|| {
            let mut classes = self
                .item
                .iter()
                .filter(|(name, _)| name.local.as_ref() == "class")
                .flat_map(|(_, value)| {
                    value
                        .split_ascii_whitespace()
                        .map(markup5ever::LocalName::from)
                })
                .collect::<Vec<_>>();

            classes.sort_unstable();
            classes.dedup();

            classes
        });

        classes.iter()
    }
}

impl std::ops::Deref for ElementAttributeTrigger {
    type Target = Vec<(markup5ever::QualName, AtomicTendril)>;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl std::ops::DerefMut for ElementAttributeTrigger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.id.take();
        self.classes.take();

        &mut self.item
    }
}

/// An element
#[derive(Clone)]
pub struct Element {
    pub name: markup5ever::QualName,
    pub attrs: ElementAttributeTrigger,
    pub template: bool,
    pub mathml_annotation_xml_integration_point: bool,
}

impl Element {
    /// Creates a new `Element`
    #[inline]
    pub fn new<I>(
        name: markup5ever::QualName,
        attrs: I,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> Box<Self>
    where
        I: Iterator<Item = (markup5ever::QualName, AtomicTendril)>,
    {
        Box::new(Self {
            name,
            attrs: ElementAttributeTrigger::new(attrs),
            template,
            mathml_annotation_xml_integration_point,
        })
    }

    /// Creates a new `Element` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic<I>(
        name: markup5ever::QualName,
        attrs: I,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> Box<Self>
    where
        I: Iterator<Item = (markup5ever::QualName, StrTendril)>,
    {
        Self::new(
            name,
            attrs.map(|(key, val)| (key, make_atomic_tendril(val))),
            template,
            mathml_annotation_xml_integration_point,
        )
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name
            || self.template != other.template
            || self.mathml_annotation_xml_integration_point
                != other.mathml_annotation_xml_integration_point
        {
            return false;
        }

        self.attrs
            .iter()
            .all(|x| other.attrs.binary_search(x).is_ok())
    }
}

impl Eq for Element {}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("name", &self.name)
            .field("attrs", &*self.attrs)
            .field("template", &self.template)
            .field(
                "mathml_annotation_xml_integration_point",
                &self.mathml_annotation_xml_integration_point,
            )
            .finish()
    }
}

/// The ProcessingInstruction interface represents a processing instruction; that is,
/// a Node which embeds an instruction targeting a specific application but that can
/// be ignored by any other applications which don't recognize the instruction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProcessingInstruction {
    pub data: AtomicTendril,
    pub target: AtomicTendril,
}

impl ProcessingInstruction {
    /// Creates a new `ProcessingInstruction`
    #[inline]
    pub fn new(data: AtomicTendril, target: AtomicTendril) -> Self {
        Self { data, target }
    }

    /// Creates a new `ProcessingInstruction` from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(data: StrTendril, target: StrTendril) -> Self {
        Self::new(make_atomic_tendril(data), make_atomic_tendril(target))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NodeData {
    Document(Document),
    Doctype(Doctype),
    Comment(Comment),
    Text(Text),
    Element(Box<Element>),
    ProcessingInstruction(ProcessingInstruction),
}

macro_rules! impl_nodedata_from_trait {
    (
        $($name:ident $from:ty)+
    ) => {
        $(
            impl From<$from> for NodeData {
                fn from(value: $from) -> NodeData {
                    NodeData::$name(value)
                }
            }
        )+
    };
}

impl_nodedata_from_trait!(
    Document Document
    Doctype Doctype
    Comment Comment
    Text Text
    Element Box<Element>
    ProcessingInstruction ProcessingInstruction
);

macro_rules! declare_nodedata_methods {
    (
        $(
            $isname:ident $name:ident $mutname:ident ($pattern:pat_param => $param:expr) -> $ret:ty
        )+
    ) => {
        $(
            pub fn $isname(&self) -> bool {
                matches!(self, $pattern)
            }

            pub fn $name(&self) -> Option<&$ret> {
                match self {
                    $pattern => Some($param),
                    _ => None,
                }
            }

            pub fn $mutname(&mut self) -> Option<&mut $ret> {
                match self {
                    $pattern => Some($param),
                    _ => None,
                }
            }
        )+
    };
}

impl NodeData {
    /// Creates a new [`NodeData`].
    pub fn new<T: Into<NodeData>>(val: T) -> Self {
        val.into()
    }

    declare_nodedata_methods!(
        is_document document document_mut (NodeData::Document(_x) => _x) -> Document
        is_doctype doctype doctype_mut (NodeData::Doctype(_x) => _x) -> Doctype
        is_comment comment comment_mut (NodeData::Comment(_x) => _x) -> Comment
        is_text text text_mut (NodeData::Text(_x) => _x) -> Text
        is_element element element_mut (NodeData::Element(_x) => _x) -> Element
        is_processing_instruction processing_instruction processing_instruction_mut (NodeData::ProcessingInstruction(_x) => _x) -> ProcessingInstruction
    );
}

impl std::hash::Hash for NodeData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Comment(x) => std::hash::Hash::hash(x, state),
            Self::Text(x) => std::hash::Hash::hash(x, state),
            Self::Element(_) => panic!("element does not implement hash"),
            Self::ProcessingInstruction(x) => std::hash::Hash::hash(x, state),
            Self::Doctype(x) => std::hash::Hash::hash(x, state),
            Self::Document(x) => std::hash::Hash::hash(x, state),
        }
    }
}

impl std::fmt::Display for NodeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Comment(x) => write!(f, "{x:?}"),
            Self::Text(x) => write!(f, "{x:?}"),
            Self::Element(x) => write!(f, "{x:?}"),
            Self::ProcessingInstruction(x) => write!(f, "{x:?}"),
            Self::Doctype(x) => write!(f, "{x:?}"),
            Self::Document(x) => write!(f, "{x:?}"),
        }
    }
}
