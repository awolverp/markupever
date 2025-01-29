pub mod atomic;
pub mod data;
mod parser;
mod treedom;

pub use parser::Parser;
pub use treedom::TreeDom;
pub use treedom::Serializer;
pub use treedom::NamespaceMap;

pub use ego_tree;
pub use markup5ever;

#[cfg(feature = "html5ever")]
pub use html5ever;

#[cfg(feature = "xml5ever")]
pub use xml5ever;
