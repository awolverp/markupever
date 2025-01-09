use markup5ever::{namespace_url, ns};
use pyo3::types::PyAnyMethods;
use pyo3::PyTypeInfo;

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
pub struct PyQualName(pub parking_lot::Mutex<markup5ever::QualName>);

#[pyo3::pymethods]
impl PyQualName {
    #[new]
    #[pyo3(signature=(local, namespace=String::new(), prefix=None, /))]
    pub(super) fn new(
        local: String,
        namespace: String,
        prefix: Option<String>,
    ) -> pyo3::PyResult<Self> {
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
    pub(super) fn local(&self) -> String {
        let lock = self.0.lock();
        lock.local.to_string()
    }

    #[getter]
    pub(super) fn namespace(&self) -> String {
        let lock = self.0.lock();
        lock.ns.to_string()
    }

    #[getter]
    pub(super) fn prefix(&self) -> Option<String> {
        let lock = self.0.lock();
        lock.prefix.clone().map(|x| x.to_string())
    }

    /// Copies the `self` and returns a new one
    pub(super) fn copy(&self) -> Self {
        let q = self.0.lock().clone();
        Self(parking_lot::Mutex::new(q))
    }

    pub(super) fn __eq__(
        &self,
        py: pyo3::Python<'_>,
        value: pyo3::PyObject,
    ) -> pyo3::PyResult<bool> {
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

    pub(super) fn __repr__(&self) -> String {
        let lock = self.0.lock();
        format!(
            "<QualName local={:?} namespace={:?} prefix={:?}>",
            &*lock.local,
            &*lock.ns,
            lock.prefix.as_deref()
        )
    }
}

pub(super) fn make_qualname_from_pyobject(
    py: pyo3::Python<'_>,
    object: &pyo3::PyObject,
) -> pyo3::PyResult<markup5ever::QualName> {
    unsafe {
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
}
