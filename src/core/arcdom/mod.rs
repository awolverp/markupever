mod node;
mod treesink;

pub mod iter;
pub mod errors {
    #[derive(Debug, PartialEq, Eq)]
    pub enum ErrorKind {
        ChildCycleDetected,
        NodeHasParent,
    }

    impl std::fmt::Display for ErrorKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::ChildCycleDetected => write!(f, "child cycle detected"),
                Self::NodeHasParent => {
                    write!(f, "node has parent, you have to remove its parent before")
                }
            }
        }
    }
}

pub use node::CommentData;
pub use node::DoctypeData;
pub use node::DocumentData;
pub use node::ElementAttributeTrigger;
pub use node::ElementData;
pub use node::Node;
pub use node::NodeData;
pub use node::ProcessingInstructionData;
pub use node::SizedSmallVec;
pub use node::TextData;
pub use node::WeakNode;
pub use node::NamespacesHashMap;

pub use treesink::ArcDom;
