use crate::atomic::{make_atomic_tendril, AtomicTendril, OnceLock};
use tendril::StrTendril;

/// The root of a document
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct DocumentInterface;

/// the doctype is the required <!doctype html> preamble found at the top of all documents.
/// Its sole purpose is to prevent a browser from switching into so-called "quirks mode"
/// when rendering a document; that is, the <!doctype html> doctype ensures that the browser makes
/// a best-effort attempt at following the relevant specifications, rather than using a different
/// rendering mode that is incompatible with some specifications.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DoctypeInterface {
    pub name: AtomicTendril,
    pub public_id: AtomicTendril,
    pub system_id: AtomicTendril,
}

impl DoctypeInterface {
    /// Create a new [`DoctypeInterface`]
    #[inline]
    pub fn new(name: AtomicTendril, public_id: AtomicTendril, system_id: AtomicTendril) -> Self {
        Self {
            name,
            public_id,
            system_id,
        }
    }

    /// Create a new [`DoctypeInterface`] from non-atomic tendril
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
pub struct CommentInterface {
    pub contents: AtomicTendril,
}

impl CommentInterface {
    /// Create a new [`CommentInterface`]
    #[inline]
    pub fn new(contents: AtomicTendril) -> Self {
        Self { contents }
    }

    /// Create a new [`CommentInterface`] from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(contents: StrTendril) -> Self {
        Self::new(make_atomic_tendril(contents))
    }
}

/// A text
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextInterface {
    pub contents: AtomicTendril,
}

impl TextInterface {
    /// Create a new [`TextInterface`]
    #[inline]
    pub fn new(contents: AtomicTendril) -> Self {
        Self { contents }
    }

    /// Create a new [`TextInterface`] from non-atomic tendril
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttributeKey(markup5ever::QualName);

impl AttributeKey {
    pub fn into_qualname(self) -> markup5ever::QualName {
        self.0
    }
}

impl AsRef<markup5ever::QualName> for AttributeKey {
    fn as_ref(&self) -> &markup5ever::QualName {
        &self.0
    }
}

impl std::ops::Deref for AttributeKey {
    type Target = markup5ever::QualName;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AttributeKey {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PartialEq<str> for AttributeKey {
    fn eq(&self, other: &str) -> bool {
        &self.0.local == other
    }
}

impl From<markup5ever::QualName> for AttributeKey {
    fn from(value: markup5ever::QualName) -> Self {
        Self(value)
    }
}

#[derive(Clone)]
pub struct ElementAttributes {
    list: Vec<(AttributeKey, AtomicTendril)>,
    id: OnceLock<Option<AtomicTendril>>,
    class: OnceLock<Vec<markup5ever::LocalName>>,
}

impl ElementAttributes {
    pub fn new(list: Vec<(AttributeKey, AtomicTendril)>) -> Self {
        Self {
            list,
            id: OnceLock::new(),
            class: OnceLock::new(),
        }
    }

    pub fn replace(&mut self, list: Vec<(AttributeKey, AtomicTendril)>) {
        self.list = list;
        self.id.take();
        self.class.take();
    }

    #[inline]
    pub fn id(&self) -> Option<&str> {
        self.id
            .get_or_init(|| {
                self.list
                    .iter()
                    .find(|(name, _)| name == "id")
                    .map(|(_, value)| value.clone())
            })
            .as_deref()
    }

    #[inline]
    pub fn class(&self) -> &[markup5ever::LocalName] {
        let classes = self.class.get_or_init(|| {
            let mut classes = self
                .list
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

        classes.as_slice()
    }
}

impl PartialEq for ElementAttributes {
    fn eq(&self, other: &Self) -> bool {
        self.list == other.list
    }
}

impl std::fmt::Debug for ElementAttributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ElementAttributes({:?})", &self.list)
    }
}

impl std::ops::Deref for ElementAttributes {
    type Target = Vec<(AttributeKey, AtomicTendril)>;

    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl std::ops::DerefMut for ElementAttributes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.id.take();
        self.class.take();
        &mut self.list
    }
}

/// An element
#[derive(Clone)]
pub struct ElementInterface {
    pub name: markup5ever::QualName,
    pub attrs: ElementAttributes,
    pub template: bool,
    pub mathml_annotation_xml_integration_point: bool,
}

impl ElementInterface {
    /// Creates a new [`ElementInterface`]
    #[inline]
    pub fn new<I, Q>(
        name: markup5ever::QualName,
        attrs: I,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> Box<Self>
    where
        I: Iterator<Item = (Q, AtomicTendril)>,
        AttributeKey: From<Q>,
    {
        Box::new(Self {
            name,
            attrs: ElementAttributes::new(attrs.map(|(k, v)| (AttributeKey::from(k), v)).collect()),
            template,
            mathml_annotation_xml_integration_point,
        })
    }

    /// Creates a new [`ElementInterface`] from non-atomic tendril
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

impl PartialEq for ElementInterface {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name
            || self.template != other.template
            || self.mathml_annotation_xml_integration_point
                != other.mathml_annotation_xml_integration_point
        {
            return false;
        }

        self.attrs == other.attrs
    }
}

impl Eq for ElementInterface {}

impl std::fmt::Debug for ElementInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("name", &self.name)
            .field("attrs", &self.attrs)
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
pub struct ProcessingInstructionInterface {
    pub data: AtomicTendril,
    pub target: AtomicTendril,
}

impl ProcessingInstructionInterface {
    /// Creates a new [`ProcessingInstructionInterface`]
    #[inline]
    pub fn new(data: AtomicTendril, target: AtomicTendril) -> Self {
        Self { data, target }
    }

    /// Creates a new [`ProcessingInstructionInterface`] from non-atomic tendril
    #[inline]
    pub fn from_non_atomic(data: StrTendril, target: StrTendril) -> Self {
        Self::new(make_atomic_tendril(data), make_atomic_tendril(target))
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum Interface {
    Document(DocumentInterface),
    Doctype(DoctypeInterface),
    Comment(CommentInterface),
    Text(TextInterface),
    Element(Box<ElementInterface>),
    ProcessingInstruction(ProcessingInstructionInterface),
}

macro_rules! impl_nodedata_from_trait {
    (
        $($name:ident $from:ty)+
    ) => {
        $(
            impl From<$from> for Interface {
                fn from(value: $from) -> Interface {
                    Interface::$name(value)
                }
            }
        )+
    };
}

impl_nodedata_from_trait!(
    Document DocumentInterface
    Doctype DoctypeInterface
    Comment CommentInterface
    Text TextInterface
    Element Box<ElementInterface>
    ProcessingInstruction ProcessingInstructionInterface
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

impl Interface {
    /// Creates a new [`Interface`].
    pub fn new<T: Into<Interface>>(val: T) -> Self {
        val.into()
    }

    declare_nodedata_methods!(
        is_document document document_mut (Interface::Document(_x) => _x) -> DocumentInterface
        is_doctype doctype doctype_mut (Interface::Doctype(_x) => _x) -> DoctypeInterface
        is_comment comment comment_mut (Interface::Comment(_x) => _x) -> CommentInterface
        is_text text text_mut (Interface::Text(_x) => _x) -> TextInterface
        is_element element element_mut (Interface::Element(_x) => _x) -> Box<ElementInterface>
        is_processing_instruction processing_instruction processing_instruction_mut (Interface::ProcessingInstruction(_x) => _x) -> ProcessingInstructionInterface
    );
}

impl std::hash::Hash for Interface {
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

impl std::fmt::Debug for Interface {
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

impl std::fmt::Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
