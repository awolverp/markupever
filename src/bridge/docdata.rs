#![allow(clippy::new_without_default)]

use super::utils::{get_node_from_pyobject, make_repr};
use crate::core::arcdom;

/// A document node of DOM.
///
/// Document is the root node of a DOM.
#[pyo3::pyclass(name = "DocumentData", module = "markupselect._rustlib", frozen)]
pub struct PyDocumentData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyDocumentData {
    #[new]
    pub(super) fn new() -> Self {
        Self(arcdom::Node::new(arcdom::DocumentData))
    }

    /// Copies the `self` and returns a new one
    pub(super) fn copy(&self) -> Self {
        Self(arcdom::Node::copy(&self.0))
    }

    pub(super) fn __eq__(
        &self,
        py: pyo3::Python<'_>,
        other: pyo3::PyObject,
    ) -> pyo3::PyResult<bool> {
        let other = get_node_from_pyobject(other.bind(py))?;
        Ok(self.0.eq(&other))
    }

    pub(super) fn __repr__(&self) -> String {
        let data = self.0.as_enum();
        make_repr(&data)
    }
}

/// A doctype node data
///
/// the doctype is the required <!doctype html> preamble found at the top of all documents.
/// Its sole purpose is to prevent a browser from switching into so-called "quirks mode"
/// when rendering a document; that is, the <!doctype html> doctype ensures that the browser makes
/// a best-effort attempt at following the relevant specifications, rather than using a different
/// rendering mode that is incompatible with some specifications.
#[pyo3::pyclass(name = "DoctypeData", module = "markupselect._rustlib", frozen)]
pub struct PyDoctypeData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyDoctypeData {
    #[new]
    #[pyo3(signature=(name, public_id, system_id, /))]
    pub(super) fn new(name: String, public_id: String, system_id: String) -> Self {
        let node = arcdom::Node::new(arcdom::DoctypeData::new(
            name.into(),
            public_id.into(),
            system_id.into(),
        ));

        Self(node)
    }

    /// Copies the `self` and returns a new one
    pub(super) fn copy(&self) -> Self {
        Self(arcdom::Node::copy(&self.0))
    }

    #[getter]
    pub(super) fn name(&self) -> String {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .name
            .clone()
            .into()
    }

    #[setter]
    pub(super) fn set_name(&self, value: String) {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .name = value.into();
    }

    #[getter]
    pub(super) fn public_id(&self) -> String {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .public_id
            .clone()
            .into()
    }

    #[setter]
    pub(super) fn set_public_id(&self, value: String) {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .public_id = value.into();
    }

    #[getter]
    pub(super) fn system_id(&self) -> String {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .system_id
            .clone()
            .into()
    }

    #[setter]
    pub(super) fn set_system_id(&self, value: String) {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .system_id = value.into();
    }

    pub(super) fn __eq__(
        &self,
        py: pyo3::Python<'_>,
        other: pyo3::PyObject,
    ) -> pyo3::PyResult<bool> {
        let other = get_node_from_pyobject(other.bind(py))?;
        Ok(self.0.eq(&other))
    }

    pub(super) fn __repr__(&self) -> String {
        let data = self.0.as_enum();
        make_repr(&data)
    }
}

/// A comment node data
///
/// The comment interface represents textual notations within markup; although it is generally not
/// visually shown, such comments are available to be read in the source view.
///
/// Comments are represented in HTML and XML as content between <!-- and -->. In XML,
/// like inside SVG or MathML markup, the character sequence -- cannot be used within a comment.
#[pyo3::pyclass(name = "CommentData", module = "markupselect._rustlib", frozen)]
pub struct PyCommentData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyCommentData {
    #[new]
    #[pyo3(signature=(contents, /))]
    pub(super) fn new(contents: String) -> Self {
        let node = arcdom::Node::new(arcdom::CommentData::new(contents.into()));

        Self(node)
    }

    /// Copies the `self` and returns a new one
    pub(super) fn copy(&self) -> Self {
        Self(arcdom::Node::copy(&self.0))
    }

    #[getter]
    pub(super) fn contents(&self) -> String {
        self.0
            .as_comment()
            .expect("PyCommentData holds a node other than comment")
            .contents
            .clone()
            .into()
    }

    #[setter]
    pub(super) fn set_contents(&self, value: String) {
        self.0
            .as_comment()
            .expect("PyCommentData holds a node other than comment")
            .contents = value.into();
    }

    pub(super) fn __eq__(
        &self,
        py: pyo3::Python<'_>,
        other: pyo3::PyObject,
    ) -> pyo3::PyResult<bool> {
        let other = get_node_from_pyobject(other.bind(py))?;
        Ok(self.0.eq(&other))
    }

    pub(super) fn __repr__(&self) -> String {
        let data = self.0.as_enum();
        make_repr(&data)
    }
}

/// A text node data
#[pyo3::pyclass(name = "TextData", module = "markupselect._rustlib", frozen)]
pub struct PyTextData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyTextData {
    #[new]
    #[pyo3(signature=(contents, /))]
    pub(super) fn new(contents: String) -> Self {
        let node = arcdom::Node::new(arcdom::TextData::new(contents.into()));

        Self(node)
    }

    /// Copies the `self` and returns a new one
    pub(super) fn copy(&self) -> Self {
        Self(arcdom::Node::copy(&self.0))
    }

    #[getter]
    pub(super) fn contents(&self) -> String {
        self.0
            .as_text()
            .expect("PyTextData holds a node other than text")
            .contents
            .clone()
            .into()
    }

    #[setter]
    pub(super) fn set_contents(&self, value: String) {
        self.0
            .as_text()
            .expect("PyTextData holds a node other than text")
            .contents = value.into();
    }

    pub(super) fn __eq__(
        &self,
        py: pyo3::Python<'_>,
        other: pyo3::PyObject,
    ) -> pyo3::PyResult<bool> {
        let other = get_node_from_pyobject(other.bind(py))?;
        Ok(self.0.eq(&other))
    }

    pub(super) fn __repr__(&self) -> String {
        let data = self.0.as_enum();
        make_repr(&data)
    }
}

/// A processing instruction node data
///
/// The ProcessingInstruction interface represents a processing instruction; that is,
/// a Node which embeds an instruction targeting a specific application but that can
/// be ignored by any other applications which don't recognize the instruction.
#[pyo3::pyclass(
    name = "ProcessingInstructionData",
    module = "markupselect._rustlib",
    frozen
)]
pub struct PyProcessingInstructionData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyProcessingInstructionData {
    #[new]
    #[pyo3(signature=(data, target, /))]
    pub(super) fn new(data: String, target: String) -> Self {
        let node = arcdom::Node::new(arcdom::ProcessingInstructionData::new(
            data.into(),
            target.into(),
        ));

        Self(node)
    }

    /// Copies the `self` and returns a new one
    pub(super) fn copy(&self) -> Self {
        Self(arcdom::Node::copy(&self.0))
    }

    #[getter]
    pub(super) fn data(&self) -> String {
        self.0
            .as_processing_instruction()
            .expect("PyProcessingInstructionData holds a node other than processing instruction")
            .data
            .clone()
            .into()
    }

    #[setter]
    pub(super) fn set_data(&self, value: String) {
        self.0
            .as_processing_instruction()
            .expect("PyProcessingInstructionData holds a node other than processing instruction")
            .data = value.into();
    }

    #[getter]
    pub(super) fn target(&self) -> String {
        self.0
            .as_processing_instruction()
            .expect("PyProcessingInstructionData holds a node other than processing instruction")
            .target
            .clone()
            .into()
    }

    #[setter]
    pub(super) fn set_target(&self, value: String) {
        self.0
            .as_processing_instruction()
            .expect("PyProcessingInstructionData holds a node other than processing instruction")
            .target = value.into();
    }

    pub(super) fn __eq__(
        &self,
        py: pyo3::Python<'_>,
        other: pyo3::PyObject,
    ) -> pyo3::PyResult<bool> {
        let other = get_node_from_pyobject(other.bind(py))?;
        Ok(self.0.eq(&other))
    }

    pub(super) fn __repr__(&self) -> String {
        let data = self.0.as_enum();
        make_repr(&data)
    }
}
