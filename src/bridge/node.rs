#![allow(clippy::new_without_default)]

use crate::core::arcdom;
use markup5ever::{namespace_url, ns};
use pyo3::types::PyAnyMethods;
use pyo3::PyTypeInfo;

unsafe fn qualname_from_pyobject(
    py: pyo3::Python<'_>,
    object: &pyo3::PyObject,
) -> pyo3::PyResult<markup5ever::QualName> {
    if pyo3::ffi::PyUnicode_Check(object.as_ptr()) == 1 {
        Ok(markup5ever::QualName::new(
            None,
            ns!(),
            object
                .bind(py)
                .extract::<String>()
                .unwrap_unchecked()
                .into(),
        ))
    } else {
        let pyqual = object.bind(py).extract::<pyo3::PyRef<'_, PyQualName>>()?;
        let lock = pyqual.0.lock();
        Ok(lock.clone())
    }
}

#[inline]
fn get_node_from_pyobject(val: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<arcdom::Node> {
    if PyNode::is_type_of(val) {
        let data = val.extract::<pyo3::PyRef<'_, PyNode>>().unwrap();

        Ok(data.0.clone())
    } else if PyDocumentData::is_type_of(val) {
        let data = val.extract::<pyo3::PyRef<'_, PyDocumentData>>().unwrap();

        Ok(data.0.clone())
    } else if PyFragmentData::is_type_of(val) {
        let data = val.extract::<pyo3::PyRef<'_, PyFragmentData>>().unwrap();

        Ok(data.0.clone())
    } else if PyDoctypeData::is_type_of(val) {
        let data = val.extract::<pyo3::PyRef<'_, PyDoctypeData>>().unwrap();

        Ok(data.0.clone())
    } else if PyCommentData::is_type_of(val) {
        let data = val.extract::<pyo3::PyRef<'_, PyCommentData>>().unwrap();

        Ok(data.0.clone())
    } else if PyTextData::is_type_of(val) {
        let data = val.extract::<pyo3::PyRef<'_, PyTextData>>().unwrap();

        Ok(data.0.clone())
    } else if PyElementData::is_type_of(val) {
        let data = val.extract::<pyo3::PyRef<'_, PyElementData>>().unwrap();

        Ok(data.0.clone())
    } else if PyProcessingInstructionData::is_type_of(val) {
        let data = val
            .extract::<pyo3::PyRef<'_, PyProcessingInstructionData>>()
            .unwrap();

        Ok(data.0.clone())
    } else {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "argument is not acceptable. must be an instance of: Node, PyDocumentData, PyFragmentData, PyDoctypeData, PyCommentData, PytextData, PyElementData, or PyProcessingInstructionData",
        ))
    }
}

/// A fully qualified name (with a namespace), used to depict names of tags and attributes.
///
/// Namespaces can be used to differentiate between similar XML fragments. For example:
///
/// ```text
/// // HTML
/// <table>
///   <tr>
///     <td>Apples</td>
///     <td>Bananas</td>
///   </tr>
/// </table>
///
/// // Furniture XML
/// <table>
///   <name>African Coffee Table</name>
///   <width>80</width>
///   <length>120</length>
/// </table>
/// ```
///
/// Without XML namespaces, we can't use those two fragments in the same document
/// at the same time. However if we declare a namespace we could instead say:
///
/// ```text
///
/// // Furniture XML
/// <furn:table xmlns:furn="https://furniture.rs">
///   <furn:name>African Coffee Table</furn:name>
///   <furn:width>80</furn:width>
///   <furn:length>120</furn:length>
/// </furn:table>
/// ```
///
/// and bind the prefix `furn` to a different namespace.
///
/// For this reason we parse names that contain a colon in the following way:
///
/// ```text
/// <furn:table>
///    |    |
///    |    +- local name
///    |
///  prefix (when resolved gives namespace_url `https://furniture.rs`)
/// ```
///
#[pyo3::pyclass(name = "QualName", module = "markupselect._rustlib", frozen)]
pub struct PyQualName(parking_lot::Mutex<markup5ever::QualName>);

#[pyo3::pymethods]
impl PyQualName {
    #[new]
    #[pyo3(signature=(local, namespace=String::new(), prefix=None, /))]
    pub fn new(local: String, namespace: String, prefix: Option<String>) -> pyo3::PyResult<Self> {
        let namespace = match &*namespace {
            "html" => ns!(html),
            "xhtml" => ns!(html),
            "xml" => ns!(xml),
            "xmlns" => ns!(xmlns),
            "xlink" => ns!(xlink),
            "svg" => ns!(svg),
            "mathml" => ns!(mathml),
            "*" => ns!(*),
            "" => ns!(),
            _ => markup5ever::Namespace::from(namespace),
        };

        let q = markup5ever::QualName::new(
            prefix.map(markup5ever::Prefix::from),
            namespace,
            markup5ever::LocalName::from(local),
        );

        Ok(Self(parking_lot::Mutex::new(q)))
    }

    #[getter]
    pub fn local(&self) -> String {
        let lock = self.0.lock();
        lock.local.to_string()
    }

    #[getter]
    pub fn namespace(&self) -> String {
        let lock = self.0.lock();
        lock.ns.to_string()
    }

    #[getter]
    pub fn prefix(&self) -> Option<String> {
        let lock = self.0.lock();
        lock.prefix.clone().map(|x| x.to_string())
    }

    pub fn __eq__(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<bool> {
        let value = value.bind(py);

        if PyQualName::is_type_of(value) {
            let data = value.extract::<pyo3::PyRef<'_, PyQualName>>()?;
            let l1 = self.0.lock();
            let l2 = data.0.lock();

            Ok(l1.eq(&*l2))
        } else {
            Ok(false)
        }
    }
}

/// A document node data
///
/// The root node
#[pyo3::pyclass(name = "DocumentData", module = "markupselect._rustlib", frozen)]
pub struct PyDocumentData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyDocumentData {
    #[new]
    pub fn new() -> Self {
        Self(arcdom::Node::new(arcdom::DocumentData))
    }

    /// Converts self into `Node`
    pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = PyNode(self.0.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
    }
}

// /// A fragment node data
// ///
// /// This is like document, but isn't root; we specialy used it for specifying templates
#[pyo3::pyclass(name = "FragmentData", module = "markupselect._rustlib", frozen)]
pub struct PyFragmentData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyFragmentData {
    #[new]
    pub fn new() -> Self {
        Self(arcdom::Node::new(arcdom::FragmentData))
    }

    /// Converts self into `Node`
    pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = PyNode(self.0.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
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
    pub fn new(name: String, public_id: String, system_id: String) -> Self {
        let node = arcdom::Node::new(arcdom::DoctypeData::new(
            name.into(),
            public_id.into(),
            system_id.into(),
        ));

        Self(node)
    }

    /// Converts self into `Node`
    pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = PyNode(self.0.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
    }

    #[getter]
    pub fn name(&self) -> String {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .name
            .clone()
            .into()
    }

    #[setter]
    pub fn set_name(&self, value: String) {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .name = value.into();
    }

    #[getter]
    pub fn public_id(&self) -> String {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .public_id
            .clone()
            .into()
    }

    #[setter]
    pub fn set_public_id(&self, value: String) {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .public_id = value.into();
    }

    #[getter]
    pub fn system_id(&self) -> String {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .system_id
            .clone()
            .into()
    }

    #[setter]
    pub fn set_system_id(&self, value: String) {
        self.0
            .as_doctype()
            .expect("PyDoctypeData holds a node other than doctype")
            .system_id = value.into();
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
    pub fn new(contents: String) -> Self {
        let node = arcdom::Node::new(arcdom::CommentData::new(contents.into()));

        Self(node)
    }

    /// Converts self into `Node`
    pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = PyNode(self.0.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
    }

    #[getter]
    pub fn contents(&self) -> String {
        self.0
            .as_comment()
            .expect("PyCommentData holds a node other than comment")
            .contents
            .clone()
            .into()
    }

    #[setter]
    pub fn set_contents(&self, value: String) {
        self.0
            .as_comment()
            .expect("PyCommentData holds a node other than comment")
            .contents = value.into();
    }
}

/// A text node data
#[pyo3::pyclass(name = "TextData", module = "markupselect._rustlib", frozen)]
pub struct PyTextData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyTextData {
    #[new]
    #[pyo3(signature=(contents, /))]
    pub fn new(contents: String) -> Self {
        let node = arcdom::Node::new(arcdom::TextData::new(contents.into()));

        Self(node)
    }

    /// Converts self into `Node`
    pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = PyNode(self.0.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
    }

    #[getter]
    pub fn contents(&self) -> String {
        self.0
            .as_text()
            .expect("PyTextData holds a node other than text")
            .contents
            .clone()
            .into()
    }

    #[setter]
    pub fn set_contents(&self, value: String) {
        self.0
            .as_text()
            .expect("PyTextData holds a node other than text")
            .contents = value.into();
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
    pub fn new(data: String, target: String) -> Self {
        let node = arcdom::Node::new(arcdom::ProcessingInstructionData::new(
            data.into(),
            target.into(),
        ));

        Self(node)
    }

    /// Converts self into `Node`
    pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = PyNode(self.0.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
    }

    #[getter]
    pub fn data(&self) -> String {
        self.0
            .as_processing_instruction()
            .expect("PyProcessingInstructionData holds a node other than processing instruction")
            .data
            .clone()
            .into()
    }

    #[setter]
    pub fn set_data(&self, value: String) {
        self.0
            .as_processing_instruction()
            .expect("PyProcessingInstructionData holds a node other than processing instruction")
            .data = value.into();
    }

    #[getter]
    pub fn target(&self) -> String {
        self.0
            .as_processing_instruction()
            .expect("PyProcessingInstructionData holds a node other than processing instruction")
            .target
            .clone()
            .into()
    }

    #[setter]
    pub fn set_target(&self, value: String) {
        self.0
            .as_processing_instruction()
            .expect("PyProcessingInstructionData holds a node other than processing instruction")
            .target = value.into();
    }
}

/// An element node data
#[pyo3::pyclass(name = "ElementAttributes", module = "markupselect._rustlib", frozen)]
pub struct PyElementAttributes {
    pub node: arcdom::Node,
    pub index: std::sync::atomic::AtomicUsize,
    pub len: std::sync::atomic::AtomicUsize,
}

#[pyo3::pymethods]
impl PyElementAttributes {
    #[new]
    #[allow(unused_variables)]
    pub fn new(element: pyo3::PyObject) -> pyo3::PyResult<Self> {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Use ElementNodeData.attrs property; don't use this constructor directly.",
        ))
    }

    pub fn __len__(&self) -> usize {
        self.node
            .as_element()
            .expect("PyElementAttributes holds a node other than element")
            .attrs
            .len()
    }

    pub fn __getitem__(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let elem = self
            .node
            .as_element()
            .expect("PyElementAttributes holds a node other than element");

        let n = match elem.attrs.get(index) {
            Some(x) => x,
            None => {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                    "out of range",
                ))
            }
        };

        let tuple = pyo3::types::PyTuple::new(
            py,
            [
                pyo3::Py::new(py, PyQualName(parking_lot::Mutex::new(n.0.clone())))?.into_any(),
                pyo3::types::PyString::new(py, &n.1).into(),
            ],
        )?;

        Ok(tuple.into())
    }

    pub fn __setitem__(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
        value: Vec<pyo3::PyObject>,
    ) -> pyo3::PyResult<()> {
        if value.len() != 2 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "the value must be a tuple (or list) with size 2",
            ));
        }

        let mut elem = self
            .node
            .as_element()
            .expect("PyElementAttributes holds a node other than element");

        if index >= elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        let qual = unsafe { qualname_from_pyobject(py, &value[0])? };

        if unsafe { pyo3::ffi::PyUnicode_Check(value[1].as_ptr()) == 0 } {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "the value argument #2 must be str",
            ));
        }

        let val = unsafe { value[1].extract::<String>(py).unwrap_unchecked() };

        if &*qual.local == "class" {
            elem.clear_classes();
        } else if &*qual.local == "id" {
            elem.clear_id();
        }

        elem.attrs[index] = (qual, val.into());
        Ok(())
    }

    pub fn __delitem__(&self, index: usize) -> pyo3::PyResult<()> {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementAttributes holds a node other than element");

        if index >= elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        let (qual, _) = elem.attrs.remove(index);

        if &*qual.local == "class" {
            elem.clear_classes();
        } else if &*qual.local == "id" {
            elem.clear_id();
        }

        Ok(())
    }

    pub fn sort(&self) {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementAttributes holds a node other than element");

        elem.attrs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    }

    pub fn clear(&self) {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementAttributes holds a node other than element");

        elem.attrs.clear();
        elem.clear_classes();
        elem.clear_id();
    }

    pub fn append(&self, py: pyo3::Python<'_>, value: Vec<pyo3::PyObject>) -> pyo3::PyResult<()> {
        if value.len() != 2 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "the value must be a tuple (or list) with size 2",
            ));
        }

        let mut elem = self
            .node
            .as_element()
            .expect("PyElementAttributes holds a node other than element");

        let qual = unsafe { qualname_from_pyobject(py, &value[0])? };

        if unsafe { pyo3::ffi::PyUnicode_Check(value[1].as_ptr()) == 0 } {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "the value argument #2 must be str",
            ));
        }

        let val = unsafe { value[1].extract::<String>(py).unwrap_unchecked() };

        if &*qual.local == "class" {
            elem.clear_classes();
        } else if &*qual.local == "id" {
            elem.clear_id();
        }

        elem.attrs.push((qual, val.into()));
        Ok(())
    }

    pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::PyRef<'_, Self>> {
        if slf.len.load(std::sync::atomic::Ordering::Relaxed) != 0 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "you can only call PyElementAttributes' __iter__() once in a time.",
            ));
        }

        slf.index.store(0, std::sync::atomic::Ordering::Relaxed);
        slf.len
            .store(slf.__len__(), std::sync::atomic::Ordering::Relaxed);
        Ok(slf)
    }

    pub fn __next__(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        let elem = slf
            .node
            .as_element()
            .expect("PyElementAttributes holds a node other than element");

        if slf.len.load(std::sync::atomic::Ordering::Relaxed) != elem.attrs.len() {
            std::mem::drop(elem);
            slf.len.store(0, std::sync::atomic::Ordering::Relaxed);
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "node attrs size changed during iteration",
            ));
        }

        if slf.index.load(std::sync::atomic::Ordering::Relaxed) >= elem.attrs.len() {
            std::mem::drop(elem);
            slf.len.store(0, std::sync::atomic::Ordering::Relaxed);
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()));
        }

        let n = &elem.attrs[slf.index.load(std::sync::atomic::Ordering::Relaxed)];
        let tuple = pyo3::types::PyTuple::new(
            py,
            [
                pyo3::Py::new(py, PyQualName(parking_lot::Mutex::new(n.0.clone())))?.into_any(),
                pyo3::types::PyString::new(py, &n.1).into(),
            ],
        )?;

        std::mem::drop(elem);
        slf.index.store(
            slf.index.load(std::sync::atomic::Ordering::Relaxed) + 1,
            std::sync::atomic::Ordering::Relaxed,
        );

        Ok(tuple.into_ptr())
    }
}

/// An element node data
#[pyo3::pyclass(name = "ElementData", module = "markupselect._rustlib", frozen)]
pub struct PyElementData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyElementData {
    #[new]
    #[pyo3(signature=(name, attrs, template=false, mathml_annotation_xml_integration_point=false, /))]
    pub fn new(
        py: pyo3::Python<'_>,
        name: pyo3::PyObject,
        attrs: Vec<(pyo3::PyObject, String)>,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> pyo3::PyResult<Self> {
        let name = unsafe { qualname_from_pyobject(py, &name)? };

        let mut attributes: Vec<(markup5ever::QualName, crate::core::send::AtomicTendril)> =
            Vec::new();

        attributes
            .try_reserve(attrs.len())
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyMemoryError, _>(e.to_string()))?;

        for (key, val) in attrs.into_iter() {
            let key = unsafe { qualname_from_pyobject(py, &key)? };
            attributes.push((key, val.into()));
        }

        let node = arcdom::Node::new(arcdom::ElementData::new(
            name,
            attributes,
            template,
            mathml_annotation_xml_integration_point,
        ));

        Ok(Self(node))
    }

    /// Converts self into `Node`
    pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let node = PyNode(self.0.clone());
        pyo3::Py::new(py, node).map(|x| x.into_any())
    }

    #[getter]
    pub fn name(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let qual = PyQualName(parking_lot::Mutex::new(
            self.0
                .as_element()
                .expect("PyElementData holds a node other than element")
                .name
                .clone(),
        ));
        pyo3::Py::new(py, qual).map(|x| x.into_any())
    }

    #[setter]
    pub fn set_name(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<()> {
        let value = unsafe { qualname_from_pyobject(py, &value)? };

        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .name = value;

        Ok(())
    }

    #[getter]
    pub fn attrs(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let attrs = PyElementAttributes {
            node: self.0.clone(),
            index: std::sync::atomic::AtomicUsize::new(0),
            len: std::sync::atomic::AtomicUsize::new(0),
        };

        pyo3::Py::new(py, attrs).map(|x| x.into_any())
    }

    #[getter]
    pub fn id(&self) -> Option<String> {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .id()
            .map(String::from)
    }

    #[getter]
    pub fn classes(&self) -> Vec<String> {
        let mut classes = Vec::new();

        for cls in self
            .0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .classes()
        {
            classes.push(String::from(cls.as_ref()));
        }

        classes
    }

    #[getter]
    pub fn template(&self) -> bool {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .template
    }

    #[setter]
    pub fn set_template(&self, value: bool) {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .template = value;
    }

    #[getter]
    pub fn mathml_annotation_xml_integration_point(&self) -> bool {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .mathml_annotation_xml_integration_point
    }

    #[setter]
    pub fn set_mathml_annotation_xml_integration_point(&self, value: bool) {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .mathml_annotation_xml_integration_point = value;
    }
}

/// An element node data
#[pyo3::pyclass(name = "Children", module = "markupselect._rustlib", frozen)]
pub struct PyChildren {
    pub node: arcdom::Node,
    pub index: std::sync::atomic::AtomicUsize,
    pub len: std::sync::atomic::AtomicUsize,
}

#[pyo3::pymethods]
impl PyChildren {
    #[new]
    #[allow(unused_variables)]
    pub fn new(element: pyo3::PyObject) -> pyo3::PyResult<Self> {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Use Node.children property; don't use this constructor directly.",
        ))
    }

    pub fn __len__(&self) -> usize {
        self.node.children().len()
    }

    pub fn __getitem__(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let node = match self.node.children().vec.get(index) {
            Some(x) => PyNode(x.clone()),
            None => {
                return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                    "out of range",
                ))
            }
        };

        Ok(pyo3::Py::new(py, node)?.into_any())
    }

    pub fn __setitem__(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
        value: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let mut children = self.node.children();

        if index >= children.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        children.vec[index] = get_node_from_pyobject(value.bind(py))?;
        Ok(())
    }

    pub fn __delitem__(&self, index: usize) -> pyo3::PyResult<()> {
        let mut children = self.node.children();

        if index >= children.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        children.remove(index);
        Ok(())
    }

    pub fn clear(&self) {
        let mut children = self.node.children();
        children.clear();
    }

    pub fn append(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<()> {
        let mut children = self.node.children();
        let node = get_node_from_pyobject(value.bind(py))?;

        children.vec.push(node);
        Ok(())
    }

    pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::PyRef<'_, Self>> {
        if slf.len.load(std::sync::atomic::Ordering::Relaxed) != 0 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "you can only call Children's __iter__() once in a time.",
            ));
        }

        slf.index.store(0, std::sync::atomic::Ordering::Relaxed);
        slf.len
            .store(slf.__len__(), std::sync::atomic::Ordering::Relaxed);
        Ok(slf)
    }

    pub fn __next__(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let children = slf.node.children();

        if slf.len.load(std::sync::atomic::Ordering::Relaxed) != children.len() {
            std::mem::drop(children);
            slf.len.store(0, std::sync::atomic::Ordering::Relaxed);
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "node chidlren size changed during iteration",
            ));
        }
        if slf.index.load(std::sync::atomic::Ordering::Relaxed) >= children.len() {
            std::mem::drop(children);
            slf.len.store(0, std::sync::atomic::Ordering::Relaxed);
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()));
        }

        let node = PyNode(children[slf.index.load(std::sync::atomic::Ordering::Relaxed)].clone());

        std::mem::drop(children);

        slf.index.store(
            slf.index.load(std::sync::atomic::Ordering::Relaxed) + 1,
            std::sync::atomic::Ordering::Relaxed,
        );
        Ok(pyo3::Py::new(py, node)?.into_any())
    }
}

/// A node
#[pyo3::pyclass(name = "Node", module = "markupselect._rustlib", frozen)]
pub struct PyNode(pub arcdom::Node);

#[pyo3::pymethods]
impl PyNode {
    #[new]
    #[pyo3(signature=(data, /))]
    pub fn new(py: pyo3::Python<'_>, data: pyo3::PyObject) -> pyo3::PyResult<Self> {
        let data = data.bind(py);

        Ok(Self(get_node_from_pyobject(data)?))
    }

    /// Returns the node data as `Py*Data` classes
    pub fn data(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let data = self.0.as_nodedata();

        let result = match &*data {
            arcdom::NodeData::Document(..) => {
                let r = pyo3::Py::new(py, PyDocumentData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Fragment(..) => {
                let r = pyo3::Py::new(py, PyFragmentData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Doctype(..) => {
                let r = pyo3::Py::new(py, PyDoctypeData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Text(..) => {
                let r = pyo3::Py::new(py, PyTextData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Comment(..) => {
                let r = pyo3::Py::new(py, PyCommentData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::Element(..) => {
                let r = pyo3::Py::new(py, PyElementData(self.0.clone()))?;
                r.into_any()
            }
            arcdom::NodeData::ProcessingInstruction(..) => {
                let r = pyo3::Py::new(py, PyProcessingInstructionData(self.0.clone()))?;
                r.into_any()
            }
        };

        Ok(result)
    }

    /// Returns `True` if the node is a document
    pub fn is_document(&self) -> bool {
        self.0.is_document()
    }

    /// Returns `True` if the node is a fragment
    pub fn is_fragment(&self) -> bool {
        self.0.is_fragment()
    }

    /// Returns `True` if the node is a doctype
    pub fn is_doctype(&self) -> bool {
        self.0.is_doctype()
    }

    /// Returns `True` if the node is a comment
    pub fn is_comment(&self) -> bool {
        self.0.is_comment()
    }

    /// Returns `True` if the node is a text
    pub fn is_text(&self) -> bool {
        self.0.is_text()
    }

    /// Returns `True` if the node is an element
    pub fn is_element(&self) -> bool {
        self.0.is_element()
    }

    /// Returns `True` if the node is a processing instruction
    pub fn is_processing_instruction(&self) -> bool {
        self.0.is_processing_instruction()
    }

    /// Returns the parent if exist
    pub fn parent(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<Option<pyo3::PyObject>> {
        let parent = self.0.parent();

        if parent.is_none() {
            return Ok(None);
        }

        let parent = unsafe {
            parent
                .clone()
                .unwrap_unchecked()
                .upgrade()
                .expect("dangling weak pointer")
        };

        Ok(Some(
            pyo3::Py::new(py, PyNode(parent)).map(|x| x.into_any())?,
        ))
    }

    pub fn children(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let children = PyChildren {
            node: self.0.clone(),
            index: std::sync::atomic::AtomicUsize::new(0),
            len: std::sync::atomic::AtomicUsize::new(0),
        };

        Ok(pyo3::Py::new(py, children)?.into_any())
    }

    /// Removes this node from its parent
    pub fn unlink(&self) {
        self.0.unlink();
    }
}
