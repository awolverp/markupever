//! **TreeDom**:
//! - new
//! - with_capacity
//! - root
//! - append
//! - prepend
//! - insert_before
//! - insert_after
//! - detach
//! - reparent_append
//! - reparent_prepend
//!
//! **Node**:
//! - new
//! - tree
//! - parent
//! - prev_sibling
//! - next_sibling
//! - first_child
//! - last_child
//! - has_sibling
//! - has_children
//! - serialize
//! - copy
//! - hash
//!
//! **Document** extends **Node**
//!
//! **Doctype** extends **Node**:
//! - name
//! - public_id
//! - system_id
//!
//! **Comment** extends **Node**:
//! - contents
//!
//! **Text** extends **Node**:
//! - contents
//!
//! **ElementAttrs** extends **list**:
//! - ...
//!
//!
//! **Element** extends **Node**:
//! - name
//! - attrs
//! - template
//! - mathml_annotation_xml_integration_point
//!
//! **ProcessingInstruction** extends **Node**:
//! - data
//! - target
//!
// use std::sync::Arc;

// #[pyo3::pyclass(name = "TreeDom", module = "xmarkup._rustlib", frozen)]
// pub struct PyTreeDom {
//     dom: Arc<parking_lot::Mutex<treedom::TreeDom>>,
// }
