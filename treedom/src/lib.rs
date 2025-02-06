pub mod atomic;
pub mod data;
mod parser;
mod treedom;

pub use parser::MarkupParser;
pub use treedom::TreeDom;
pub use treedom::Serializer;
pub use treedom::NamespaceMap;

pub use ego_tree;
pub use markup5ever;
pub use tendril;

#[cfg(feature = "html5ever")]
pub use html5ever;

#[cfg(feature = "xml5ever")]
pub use xml5ever;
