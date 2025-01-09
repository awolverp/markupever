mod builder;
mod docdata;
mod elementdata;
mod node;
mod qualname;

pub use builder::PyHtml;
pub use builder::PyHtmlOptions;
pub use builder::PyXml;
pub use builder::PyXmlOptions;
pub use builder::QUIRKS_MODE_FULL;
pub use builder::QUIRKS_MODE_LIMITED;
pub use builder::QUIRKS_MODE_OFF;

pub use qualname::PyQualName;

pub use docdata::PyCommentData;
pub use docdata::PyDoctypeData;
pub use docdata::PyDocumentData;
pub use docdata::PyProcessingInstructionData;
pub use docdata::PyTextData;

pub use elementdata::PyElementData;
pub use elementdata::PyElementDataAttributes;

pub use node::PyNode;
pub use node::PyNodeChildren;
pub use node::PyParentsIterator;
pub use node::PySelectExpr;
pub use node::PyTreeIterator;

mod utils {
    use super::docdata;
    use super::elementdata;
    use super::node::PyNode;
    use crate::core::arcdom;

    use pyo3::types::PyAnyMethods;
    use pyo3::PyTypeInfo;

    #[inline]
    pub(super) fn get_node_from_pyobject(
        val: &pyo3::Bound<'_, pyo3::PyAny>,
    ) -> pyo3::PyResult<arcdom::Node> {
        if PyNode::is_type_of(val) {
            let data = val.extract::<pyo3::PyRef<'_, PyNode>>().unwrap();

            Ok(data.0.clone())
        } else if docdata::PyDocumentData::is_type_of(val) {
            let data = val
                .extract::<pyo3::PyRef<'_, docdata::PyDocumentData>>()
                .unwrap();

            Ok(data.0.clone())
        } else if docdata::PyDoctypeData::is_type_of(val) {
            let data = val
                .extract::<pyo3::PyRef<'_, docdata::PyDoctypeData>>()
                .unwrap();

            Ok(data.0.clone())
        } else if docdata::PyCommentData::is_type_of(val) {
            let data = val
                .extract::<pyo3::PyRef<'_, docdata::PyCommentData>>()
                .unwrap();

            Ok(data.0.clone())
        } else if docdata::PyTextData::is_type_of(val) {
            let data = val
                .extract::<pyo3::PyRef<'_, docdata::PyTextData>>()
                .unwrap();

            Ok(data.0.clone())
        } else if elementdata::PyElementData::is_type_of(val) {
            let data = val
                .extract::<pyo3::PyRef<'_, elementdata::PyElementData>>()
                .unwrap();

            Ok(data.0.clone())
        } else if docdata::PyProcessingInstructionData::is_type_of(val) {
            let data = val
                .extract::<pyo3::PyRef<'_, docdata::PyProcessingInstructionData>>()
                .unwrap();

            Ok(data.0.clone())
        } else {
            Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "argument is not acceptable. must be an instance of: Node, PyDocumentData, PyFragmentData, PyDoctypeData, PyCommentData, PytextData, PyElementData, or PyProcessingInstructionData",
        ))
        }
    }

    #[inline]
    pub(super) fn make_repr(data: &arcdom::NodeData) -> String {
        match data {
            arcdom::NodeData::Document(..) => String::from("DocumentData"),
            arcdom::NodeData::Doctype(doc) => {
                format!(
                    "DoctypeData(name={:?}, public_id={:?}, system_id={:?})",
                    &*doc.name, &*doc.public_id, &*doc.system_id
                )
            }
            arcdom::NodeData::Text(text) => format!("TextData(contents={:?})", &*text.contents),
            arcdom::NodeData::Comment(comment) => {
                format!("CommentData(contents={:?})", &*comment.contents)
            }
            arcdom::NodeData::Element(element) => {
                let mut writer = format!(
                    "ElementData(name=QualName(local={:?}, namespace={:?}, prefix={:?}), attrs=[",
                    &*element.name.local,
                    &*element.name.ns,
                    element.name.prefix.as_deref()
                );

                let mut iter_ = element.attrs.iter();

                if let Some((key, val)) = iter_.next() {
                    writer += &format!("({:?}, {:?})", &*key.local, val.as_ref());
                }

                for (key, val) in iter_ {
                    writer += &format!(", ({:?}, {:?})", &*key.local, val.as_ref());
                }

                writer
                    + &format!(
                        "], template={}, mathml_annotation_xml_integration_point={})",
                        element.template, element.mathml_annotation_xml_integration_point
                    )
            }
            arcdom::NodeData::ProcessingInstruction(pi) => {
                format!(
                    "ProcessingInstructionData(data={:?}, target={:?})",
                    &*pi.data, &*pi.target
                )
            }
        }
    }
}
