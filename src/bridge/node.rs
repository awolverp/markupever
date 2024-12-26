// #![allow(clippy::new_without_default)]

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
    #[pyo3(signature=(local, namespace, prefix=None, /))]
    pub fn new(local: String, namespace: String, prefix: Option<String>) -> pyo3::PyResult<Self> {
        let namespace = match &*namespace {
            "html" => ns!(html),
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

    // /// Converts self into `Node`
    // pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    //     let node = PyNode(self.0.clone());
    //     pyo3::Py::new(py, node).map(|x| x.into_any())
    // }

    // pub fn __eq__(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<bool> {
    //     let value = value.bind(py);

    //     if PyDocumentData::is_type_of(value) {
    //         let data = value.extract::<pyo3::PyRef<'_, PyDocumentData>>()?;
    //         Ok(self.0.ptr_eq(&data.0))
    //     } else {
    //         Ok(false)
    //     }
    // }
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

    // /// Converts self into `Node`
    // pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    //     let node = PyNode(self.0.clone());
    //     pyo3::Py::new(py, node).map(|x| x.into_any())
    // }

    // pub fn __eq__(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<bool> {
    //     let value = value.bind(py);

    //     if PyFragmentData::is_type_of(value) {
    //         let data = value.extract::<pyo3::PyRef<'_, PyFragmentData>>()?;
    //         Ok(self.0.ptr_eq(&data.0))
    //     } else {
    //         Ok(false)
    //     }
    // }
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

    // /// Converts self into `Node`
    // pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    //     let node = PyNode(self.0.clone());
    //     pyo3::Py::new(py, node).map(|x| x.into_any())
    // }

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

    // pub fn __eq__(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<bool> {
    //     let value = value.bind(py);

    //     if PyDoctypeData::is_type_of(value) {
    //         let data = value.extract::<pyo3::PyRef<'_, PyDoctypeData>>()?;
    //         Ok(self.0.ptr_eq(&data.0))
    //     } else {
    //         Ok(false)
    //     }
    // }
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

    // /// Converts self into `Node`
    // pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    //     let node = PyNode(self.0.clone());
    //     pyo3::Py::new(py, node).map(|x| x.into_any())
    // }

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

    // pub fn __eq__(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<bool> {
    //     let value = value.bind(py);

    //     if PyCommentData::is_type_of(value) {
    //         let data = value.extract::<pyo3::PyRef<'_, PyCommentData>>()?;
    //         Ok(self.0.ptr_eq(&data.0))
    //     } else {
    //         Ok(false)
    //     }
    // }
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

    // /// Converts self into `Node`
    // pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    //     let node = PyNode(self.0.clone());
    //     pyo3::Py::new(py, node).map(|x| x.into_any())
    // }

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
    name = "ProcessingInstructionNodeData",
    module = "markupselect._rustlib",
    frozen
)]
pub struct PyProcessingInstructionNodeData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyProcessingInstructionNodeData {
    #[new]
    #[pyo3(signature=(data, target, /))]
    pub fn new(data: String, target: String) -> Self {
        let node = arcdom::Node::new(arcdom::ProcessingInstructionData::new(
            data.into(),
            target.into(),
        ));

        Self(node)
    }

    // /// Converts self into `Node`
    // pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
    //     let node = PyNode(self.0.clone());
    //     pyo3::Py::new(py, node).map(|x| x.into_any())
    // }

    #[getter]
    pub fn data(&self) -> String {
        self.0
            .as_processing_instruction()
            .expect(
                "PyProcessingInstructionNodeData holds a node other than processing instruction",
            )
            .data
            .clone()
            .into()
    }

    #[setter]
    pub fn set_data(&self, value: String) {
        self.0
            .as_processing_instruction()
            .expect(
                "PyProcessingInstructionNodeData holds a node other than processing instruction",
            )
            .data = value.into();
    }

    #[getter]
    pub fn target(&self) -> String {
        self.0
            .as_processing_instruction()
            .expect(
                "PyProcessingInstructionNodeData holds a node other than processing instruction",
            )
            .target
            .clone()
            .into()
    }

    #[setter]
    pub fn set_target(&self, value: String) {
        self.0
            .as_processing_instruction()
            .expect(
                "PyProcessingInstructionNodeData holds a node other than processing instruction",
            )
            .target = value.into();
    }
}

// /// The element node data's attributes
// #[pyo3::pyclass(name = "ElementNodeDataAttributes", module = "markupselect._rustlib")]
// pub struct PyElementNodeDataAttributes {
//     pub node: Node,
//     index: usize,
//     len: usize,
// }

// #[pyo3::pymethods]
// impl PyElementNodeDataAttributes {
//     #[new]
//     #[allow(unused_variables)]
//     pub fn new(element: pyo3::PyObject) -> pyo3::PyResult<Self> {
//         Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
//             "Use ElementNodeData.attrs property; don't use this constructor directly.",
//         ))
//     }

//     pub fn __len__(&self) -> usize {
//         if let NodeData::Element { attrs, .. } = &*self.node.data() {
//             attrs.len()
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*self.node.data()
//             );
//         }
//     }

//     pub fn __getitem__(
//         &self,
//         py: pyo3::Python<'_>,
//         index: usize,
//     ) -> pyo3::PyResult<pyo3::PyObject> {
//         if let NodeData::Element { attrs, .. } = &*self.node.data() {
//             let n = match attrs.get(index) {
//                 Some(x) => x,
//                 None => {
//                     return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
//                         "out of range",
//                     ))
//                 }
//             };

//             let tuple = pyo3::types::PyTuple::new(
//                 py,
//                 [
//                     pyo3::Py::new(py, PyQualName(parking_lot::Mutex::new(n.0.clone())))?.into_any(),
//                     pyo3::types::PyString::new(py, &n.1).into(),
//                 ],
//             )?;

//             Ok(tuple.into())
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*self.node.data()
//             );
//         }
//     }

//     pub fn __setitem__(
//         &self,
//         py: pyo3::Python<'_>,
//         index: usize,
//         value: Vec<pyo3::PyObject>,
//     ) -> pyo3::PyResult<()> {
//         if value.len() != 2 {
//             return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
//                 "the value must be a tuple (or list) with size 2",
//             ));
//         }

//         if let NodeData::Element {
//             attrs,
//             _classes,
//             _id,
//             ..
//         } = &mut *self.node.data()
//         {
//             if index >= attrs.len() {
//                 return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
//                     "out of range",
//                 ));
//             }

//             let qual = unsafe { qualname_from_pyobject(py, &value[0])? };

//             if unsafe { pyo3::ffi::PyUnicode_Check(value[1].as_ptr()) == 0 } {
//                 return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                     "the value argument #2 must be str",
//                 ));
//             }

//             let val = unsafe { value[1].extract::<String>(py).unwrap_unchecked() };

//             if &*qual.local == "class" {
//                 _classes.take();
//             } else if &*qual.local == "id" {
//                 _id.take();
//             }

//             attrs[index] = (qual, val.into());
//             Ok(())
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*self.node.data()
//             );
//         }
//     }

//     pub fn __delitem__(&self, index: usize) -> pyo3::PyResult<()> {
//         if let NodeData::Element {
//             attrs,
//             _classes,
//             _id,
//             ..
//         } = &mut *self.node.data()
//         {
//             if index >= attrs.len() {
//                 Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
//                     "out of range",
//                 ))
//             } else {
//                 _classes.take();
//                 _id.take();
//                 attrs.remove(index);
//                 Ok(())
//             }
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*self.node.data()
//             );
//         }
//     }

//     pub fn sort(&self) {
//         if let NodeData::Element { attrs, .. } = &mut *self.node.data() {
//             attrs.sort_unstable();
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*self.node.data()
//             );
//         }
//     }

//     pub fn clear(&self) {
//         if let NodeData::Element {
//             attrs,
//             _classes,
//             _id,
//             ..
//         } = &mut *self.node.data()
//         {
//             _classes.take();
//             _id.take();
//             attrs.clear();
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*self.node.data()
//             );
//         }
//     }

//     pub fn append(&self, py: pyo3::Python<'_>, value: Vec<pyo3::PyObject>) -> pyo3::PyResult<()> {
//         if value.len() != 2 {
//             return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
//                 "the value must be a tuple (or list) with size 2",
//             ));
//         }

//         if let NodeData::Element {
//             attrs,
//             _classes,
//             _id,
//             ..
//         } = &mut *self.node.data()
//         {
//             let qual = unsafe { qualname_from_pyobject(py, &value[0])? };

//             if unsafe { pyo3::ffi::PyUnicode_Check(value[1].as_ptr()) == 0 } {
//                 return Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                     "the value argument #2 must be str",
//                 ));
//             }

//             let val = unsafe { value[1].extract::<String>(py).unwrap_unchecked() };

//             if &*qual.local == "class" {
//                 _classes.take();
//             } else if &*qual.local == "id" {
//                 _id.take();
//             }

//             attrs.push((qual, val.into()));
//             Ok(())
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*self.node.data()
//             );
//         }
//     }

//     pub fn __iter__(mut slf: pyo3::PyRefMut<'_, Self>) -> pyo3::PyResult<pyo3::PyRefMut<'_, Self>> {
//         if slf.len != 0 {
//             return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
//                 // TODO: fix this text
//                 "you can iterate PyElementNodeDataAttributes instance once in a time",
//             ));
//         }

//         slf.index = 0;
//         slf.len = slf.__len__();
//         Ok(slf)
//     }

//     pub fn __next__(
//         mut slf: pyo3::PyRefMut<'_, Self>,
//         py: pyo3::Python<'_>,
//     ) -> pyo3::PyResult<pyo3::PyObject> {
//         let real_len = slf.__len__();

//         if slf.len != real_len {
//             slf.len = 0;
//             return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
//                 "node attrs size changed during iteration",
//             ));
//         }

//         if slf.index >= real_len {
//             slf.len = 0;
//             return Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(()));
//         }

//         let tuple = if let NodeData::Element { attrs, .. } = &*slf.node.data() {
//             let n = &attrs[slf.index];

//             pyo3::types::PyTuple::new(
//                 py,
//                 [
//                     pyo3::Py::new(py, PyQualName(parking_lot::Mutex::new(n.0.clone())))?.into_any(),
//                     pyo3::types::PyString::new(py, &n.1).into(),
//                 ],
//             )?
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*slf.node.data()
//             );
//         };

//         slf.index += 1;
//         Ok(tuple.into())
//     }

//     pub fn __eq__(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<bool> {
//         let value = value.bind(py);

//         if PyElementNodeDataAttributes::is_type_of(value) {
//             let data = value.extract::<pyo3::PyRef<'_, PyElementNodeDataAttributes>>()?;
//             Ok(self.node.ptr_eq(&data.node))
//         } else {
//             Ok(false)
//         }
//     }

//     pub fn __repr__(&self) -> String {
//         self.to_string()
//     }
// }

// impl std::fmt::Display for PyElementNodeDataAttributes {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if let NodeData::Element { attrs, .. } = &*self.node.data() {
//             write!(f, "ElementNodeDataAttributes([")?;

//             let mut iter_ = attrs.iter();

//             if let Some((key, val)) = iter_.next() {
//                 write!(f, "({:?}, {:?})", &*key.local, val.as_ref())?
//             }

//             for (key, val) in iter_ {
//                 write!(f, ", ({:?}, {:?})", &*key.local, val.as_ref())?;
//             }

//             write!(f, "])")
//         } else {
//             unreachable!(
//                 "PyElementNodeDataAttributes holds a node other than element: {:?}",
//                 &*self.node.data()
//             );
//         }
//     }
// }

// /// An element node data
// #[pyo3::pyclass(name = "ElementNodeData", module = "markupselect._rustlib", frozen)]
// pub struct PyElementNodeData(pub Node);

// #[pyo3::pymethods]
// impl PyElementNodeData {
//     #[new]
//     #[pyo3(signature=(name, attrs, template=false, mathml_annotation_xml_integration_point=false, /))]
//     pub fn new(
//         py: pyo3::Python<'_>,
//         name: pyo3::PyObject,
//         attrs: Vec<(pyo3::PyObject, String)>,
//         template: bool,
//         mathml_annotation_xml_integration_point: bool,
//     ) -> pyo3::PyResult<Self> {
//         let name = unsafe { qualname_from_pyobject(py, &name)? };

//         let mut attributes: Vec<(QualName, crate::markuplib::AtomicTendril)> = Vec::new();
//         attributes
//             .try_reserve(attrs.len())
//             .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyMemoryError, _>(e.to_string()))?;

//         for (key, val) in attrs.into_iter() {
//             let key = unsafe { qualname_from_pyobject(py, &key)? };
//             attributes.push((key, val.into()));
//         }

//         let node = Node::new(NodeData::Element {
//             name,
//             attrs: attributes,
//             template,
//             mathml_annotation_xml_integration_point,
//             _id: std::sync::OnceLock::new(),
//             _classes: std::sync::OnceLock::new(),
//         });

//         Ok(Self(node))
//     }

//     /// Converts self into `Node`
//     pub fn as_node(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
//         let node = PyNode(self.0.clone());
//         pyo3::Py::new(py, node).map(|x| x.into_any())
//     }

//     #[getter]
//     pub fn name(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
//         if let NodeData::Element { name, .. } = &*self.0.data() {
//             let qual = PyQualName(parking_lot::Mutex::new(name.clone()));
//             pyo3::Py::new(py, qual).map(|x| x.into_any())
//         } else {
//             unreachable!(
//                 "PyElementNodeData holds a node other than element: {:?}",
//                 &*self.0.data()
//             );
//         }
//     }

//     #[setter]
//     pub fn set_name(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<()> {
//         let value = unsafe { qualname_from_pyobject(py, &value)? };

//         if let NodeData::Element { name, .. } = &mut *self.0.data() {
//             *name = value;
//         } else {
//             unreachable!(
//                 "PyElementNodeData holds a node other than element: {:?}",
//                 &*self.0.data()
//             );
//         }

//         Ok(())
//     }

//     #[getter]
//     pub fn attrs(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
//         let attrs = PyElementNodeDataAttributes {
//             node: self.0.clone(),
//             index: 0,
//             len: 0,
//         };

//         pyo3::Py::new(py, attrs).map(|x| x.into_any())
//     }

//     #[getter]
//     pub fn id(&self) -> Option<String> {
//         self.0.data().id().map(String::from)
//     }

//     #[getter]
//     pub fn classes(&self) -> Vec<String> {
//         let mut classes = Vec::new();

//         for cls in self.0.data().classes() {
//             classes.push(String::from(cls.as_ref()));
//         }

//         classes
//     }

//     #[getter]
//     pub fn template(&self) -> bool {
//         if let NodeData::Element { template, .. } = &*self.0.data() {
//             *template
//         } else {
//             unreachable!(
//                 "PyElementNodeData holds a node other than element: {:?}",
//                 &*self.0.data()
//             );
//         }
//     }

//     #[setter]
//     pub fn set_template(&self, value: bool) -> pyo3::PyResult<()> {
//         if let NodeData::Element { template, .. } = &mut *self.0.data() {
//             *template = value;
//         } else {
//             unreachable!(
//                 "PyElementNodeData holds a node other than element: {:?}",
//                 &*self.0.data()
//             );
//         }

//         Ok(())
//     }

//     #[getter]
//     pub fn mathml_annotation_xml_integration_point(&self) -> bool {
//         if let NodeData::Element {
//             mathml_annotation_xml_integration_point,
//             ..
//         } = &*self.0.data()
//         {
//             *mathml_annotation_xml_integration_point
//         } else {
//             unreachable!(
//                 "PyElementNodeData holds a node other than element: {:?}",
//                 &*self.0.data()
//             );
//         }
//     }

//     #[setter]
//     pub fn set_mathml_annotation_xml_integration_point(&self, value: bool) -> pyo3::PyResult<()> {
//         if let NodeData::Element {
//             mathml_annotation_xml_integration_point,
//             ..
//         } = &mut *self.0.data()
//         {
//             *mathml_annotation_xml_integration_point = value;
//         } else {
//             unreachable!(
//                 "PyElementNodeData holds a node other than element: {:?}",
//                 &*self.0.data()
//             );
//         }

//         Ok(())
//     }

//     pub fn __eq__(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<bool> {
//         let value = value.bind(py);

//         if PyElementNodeData::is_type_of(value) {
//             let data = value.extract::<pyo3::PyRef<'_, PyElementNodeData>>()?;
//             Ok(self.0.ptr_eq(&data.0))
//         } else {
//             Ok(false)
//         }
//     }

//     pub fn __repr__(&self) -> String {
//         self.to_string()
//     }
// }

// impl std::fmt::Display for PyElementNodeData {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if let NodeData::Element { name, attrs, .. } = &*self.0.data() {
//             write!(
//                 f,
//                 "ElementNodeData(name=QualName(local={:?}, namespace={:?}, prefix={:?}), attrs=ElementNodeDataAttributes([",
//                 name.local.as_ref(),
//                 name.ns.as_ref(),
//                 name.prefix.as_deref(),
//             )?;

//             let mut iter_ = attrs.iter();

//             if let Some((key, val)) = iter_.next() {
//                 write!(f, "({:?}, {:?})", &*key.local, val.as_ref())?
//             }

//             for (key, val) in iter_ {
//                 write!(f, ", ({:?}, {:?})", &*key.local, val.as_ref())?;
//             }

//             write!(f, "]))")
//         } else {
//             unreachable!(
//                 "PyElementNodeData holds a node other than element: {:?}",
//                 &*self.0.data()
//             );
//         }
//     }
// }

// /// Iterate all nodes
// #[pyo3::pyclass(name = "NodesIterator", module = "markupselect._rustlib")]
// pub struct PyNodesIterator(pub NodesIterator);

// #[pyo3::pymethods]
// impl PyNodesIterator {
//     #[new]
//     #[allow(unused_variables)]
//     pub fn new(node: pyo3::PyObject) -> pyo3::PyResult<Self> {
//         Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
//             "Use Node.iterall() method; don't use this constructor directly.",
//         ))
//     }

//     pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyRef<'_, Self> {
//         slf
//     }

//     pub fn __next__(
//         mut slf: pyo3::PyRefMut<'_, Self>,
//         py: pyo3::Python<'_>,
//     ) -> pyo3::PyResult<pyo3::PyObject> {
//         match slf.0.next() {
//             Some(n) => {
//                 let n = PyNode(n);
//                 Ok(pyo3::Py::new(py, n)?.into_any())
//             }
//             None => Err(pyo3::PyErr::new::<pyo3::exceptions::PyStopIteration, _>(())),
//         }
//     }
// }

// /// A node
// #[pyo3::pyclass(name = "Node", module = "markupselect._rustlib", frozen)]
// pub struct PyNode(pub Node);

// #[pyo3::pymethods]
// impl PyNode {
//     #[new]
//     #[pyo3(signature=(data, /))]
//     pub fn new(py: pyo3::Python<'_>, data: pyo3::PyObject) -> pyo3::PyResult<Self> {
//         let data = data.bind(py);

//         if PyNode::is_type_of(data) {
//             let data = data.extract::<pyo3::PyRef<'_, PyNode>>().unwrap();

//             Ok(Self(data.0.clone()))
//         } else if PyDocumentNodeData::is_type_of(data) {
//             let data = data
//                 .extract::<pyo3::PyRef<'_, PyDocumentNodeData>>()
//                 .unwrap();

//             Ok(Self(data.0.clone()))
//         } else if PyFragmentNodeData::is_type_of(data) {
//             let data = data
//                 .extract::<pyo3::PyRef<'_, PyFragmentNodeData>>()
//                 .unwrap();

//             Ok(Self(data.0.clone()))
//         } else if PyDoctypeNodeData::is_type_of(data) {
//             let data = data
//                 .extract::<pyo3::PyRef<'_, PyDoctypeNodeData>>()
//                 .unwrap();

//             Ok(Self(data.0.clone()))
//         } else if PyCommentNodeData::is_type_of(data) {
//             let data = data
//                 .extract::<pyo3::PyRef<'_, PyCommentNodeData>>()
//                 .unwrap();

//             Ok(Self(data.0.clone()))
//         } else if PyTextData::is_type_of(data) {
//             let data = data.extract::<pyo3::PyRef<'_, PyTextData>>().unwrap();

//             Ok(Self(data.0.clone()))
//         } else if PyElementNodeData::is_type_of(data) {
//             let data = data
//                 .extract::<pyo3::PyRef<'_, PyElementNodeData>>()
//                 .unwrap();

//             Ok(Self(data.0.clone()))
//         } else if PyProcessingInstructionNodeData::is_type_of(data) {
//             let data = data
//                 .extract::<pyo3::PyRef<'_, PyProcessingInstructionNodeData>>()
//                 .unwrap();

//             Ok(Self(data.0.clone()))
//         } else {
//             Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
//                 "argument is not acceptable. must be an instance of: Node, PyDocumentNodeData, PyFragmentNodeData, PyDoctypeNodeData, PyCommentNodeData, PytextNodeData, PyElementNodeData, or PyProcessingInstructionNodeData",
//             ))
//         }
//     }

//     /// Returns the node data as `Py*NodeData` classes
//     pub fn data(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
//         let data = self.0.data();

//         let result = match &*data {
//             NodeData::Document => {
//                 let r = pyo3::Py::new(py, PyDocumentNodeData(self.0.clone()))?;
//                 r.into_any()
//             }
//             NodeData::Fragment => {
//                 let r = pyo3::Py::new(py, PyFragmentNodeData(self.0.clone()))?;
//                 r.into_any()
//             }
//             NodeData::Doctype { .. } => {
//                 let r = pyo3::Py::new(py, PyDoctypeNodeData(self.0.clone()))?;
//                 r.into_any()
//             }
//             NodeData::Text { .. } => {
//                 let r = pyo3::Py::new(py, PyTextNodeData(self.0.clone()))?;
//                 r.into_any()
//             }
//             NodeData::Comment { .. } => {
//                 let r = pyo3::Py::new(py, PyCommentNodeData(self.0.clone()))?;
//                 r.into_any()
//             }
//             NodeData::Element { .. } => {
//                 let r = pyo3::Py::new(py, PyElementNodeData(self.0.clone()))?;
//                 r.into_any()
//             }
//             NodeData::ProcessingInstruction { .. } => {
//                 let r = pyo3::Py::new(py, PyProcessingInstructionNodeData(self.0.clone()))?;
//                 r.into_any()
//             }
//         };

//         Ok(result)
//     }

//     /// Returns `True` if the node is a document
//     pub fn is_document(&self) -> bool {
//         self.0.data().is_document()
//     }

//     /// Returns `True` if the node is a fragment
//     pub fn is_fragment(&self) -> bool {
//         self.0.data().is_fragment()
//     }

//     /// Returns `True` if the node is a doctype
//     pub fn is_doctype(&self) -> bool {
//         self.0.data().is_doctype()
//     }

//     /// Returns `True` if the node is a comment
//     pub fn is_comment(&self) -> bool {
//         self.0.data().is_comment()
//     }

//     /// Returns `True` if the node is a text
//     pub fn is_text(&self) -> bool {
//         self.0.data().is_text()
//     }

//     /// Returns `True` if the node is an element
//     pub fn is_element(&self) -> bool {
//         self.0.data().is_element()
//     }

//     /// Returns `True` if the node is a processing instruction
//     pub fn is_processing_instruction(&self) -> bool {
//         self.0.data().is_processing_instruction()
//     }

//     /// Returns the parent node
//     pub fn parent(&self, py: pyo3::Python<'_>) -> Option<pyo3::PyObject> {
//         match &*self.0.parent() {
//             Some(parent) => {
//                 let node = PyNode(
//                     parent
//                         .upgrade()
//                         .expect("dangling weak pointer - node.parent()"),
//                 );
//                 Some(pyo3::Py::new(py, node).unwrap().into_any())
//             }
//             None => None,
//         }
//     }

//     pub fn serialize_xml(&self) -> pyo3::PyResult<Vec<u8>> {
//         let s = crate::markuplib::builder::NodeSerializer::new(&self.0);
//         let mut writer = Vec::new();

//         crate::markuplib::serialize_xml(&mut writer, &s)
//             .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(x.to_string()))?;

//         Ok(writer)
//     }

//     pub fn serialize_html(&self) -> pyo3::PyResult<Vec<u8>> {
//         let s = crate::markuplib::builder::NodeSerializer::new(&self.0);
//         let mut writer = Vec::new();

//         crate::markuplib::serialize_html(&mut writer, &s)
//             .map_err(|x| pyo3::PyErr::new::<pyo3::exceptions::PyIOError, _>(x.to_string()))?;

//         Ok(writer)
//     }

//     #[pyo3(signature=(include_self=true, /))]
//     pub fn iterall(
//         &self,
//         py: pyo3::Python<'_>,
//         include_self: bool,
//     ) -> pyo3::PyResult<pyo3::PyObject> {
//         let iter = NodesIterator::new(&self.0, include_self);
//         Ok(pyo3::Py::new(py, PyNodesIterator(iter))?.into_any())
//     }

//     pub fn __eq__(&self, py: pyo3::Python<'_>, value: pyo3::PyObject) -> pyo3::PyResult<bool> {
//         let value = value.bind(py);

//         if PyNode::is_type_of(value) {
//             let data = value.extract::<pyo3::PyRef<'_, PyNode>>()?;
//             Ok(self.0.ptr_eq(&data.0))
//         } else {
//             Ok(false)
//         }
//     }

//     pub fn __repr__(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<String> {
//         Ok(format!("Node({})", self.data(py)?.to_string()))
//     }
// }
