pub mod atomic;
pub mod interface;
mod parser;
mod dom;

pub use parser::ParserSink;
pub use dom::NamespaceMap;
pub use dom::Serializer;
pub use dom::IDTreeDOM;

pub use markup5ever;
pub use tendril;

pub use ego_tree::iter;
pub use ego_tree::NodeId;
pub type NodeRef<'a> = ego_tree::NodeRef<'a, interface::Interface>;
pub type NodeMut<'a> = ego_tree::NodeMut<'a, interface::Interface>;

#[cfg(feature = "html5ever")]
pub use html5ever;

#[cfg(feature = "xml5ever")]
pub use xml5ever;
