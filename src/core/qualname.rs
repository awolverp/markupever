use std::hash::Hasher;

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
#[pyo3::pyclass(name = "QualName", module = "xmarkup._rustlib", frozen)]
pub struct PyQualName {
    pub name: treedom::markup5ever::QualName,
}

#[pyo3::pymethods]
impl PyQualName {
    #[new]
    #[pyo3(signature=(local, ns=String::new(), prefix=None))]
    fn new(local: String, ns: String, prefix: Option<String>) -> pyo3::PyResult<Self> {
        let ns = match &*ns {
            "html" => treedom::markup5ever::namespace_url!("http://www.w3.org/1999/xhtml"),
            "xhtml" => treedom::markup5ever::namespace_url!("http://www.w3.org/1999/xhtml"),
            "xml" => treedom::markup5ever::namespace_url!("http://www.w3.org/XML/1998/namespace"),
            "xmlns" => treedom::markup5ever::namespace_url!("http://www.w3.org/2000/xmlns/"),
            "xlink" => treedom::markup5ever::namespace_url!("http://www.w3.org/1999/xlink"),
            "svg" => treedom::markup5ever::namespace_url!("http://www.w3.org/2000/svg"),
            "mathml" => treedom::markup5ever::namespace_url!("http://www.w3.org/1998/Math/MathML"),
            "*" => treedom::markup5ever::namespace_url!("*"),
            "" => treedom::markup5ever::namespace_url!(""),
            _ => treedom::markup5ever::Namespace::from(ns),
        };

        let name = treedom::markup5ever::QualName::new(
            prefix.map(treedom::markup5ever::Prefix::from),
            ns,
            treedom::markup5ever::LocalName::from(local),
        );

        Ok(Self { name })
    }

    #[getter]
    fn local(&self) -> String {
        self.name.local.to_string()
    }

    #[getter]
    fn ns(&self) -> String {
        self.name.ns.to_string()
    }

    #[getter]
    fn prefix(&self) -> Option<String> {
        self.name.prefix.as_ref().map(|x| x.to_string())
    }

    fn copy(&self) -> Self {
        Self {
            name: self.name.clone(),
        }
    }

    fn __richcmp__(
        self_: pyo3::PyRef<'_, Self>,
        other: pyo3::PyObject,
        cmp: pyo3::basic::CompareOp,
    ) -> pyo3::PyResult<bool> {
        if matches!(cmp, pyo3::basic::CompareOp::Eq)
            && std::ptr::addr_eq(self_.as_ptr(), other.as_ptr())
        {
            return Ok(true);
        }

        macro_rules! create_error {
            ($token:expr, $selfobj:expr, $otherobj:expr) => {
                unsafe {
                    Err(pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                        format!(
                            "'{}' not supported between '{}' and '{}'",
                            $token,
                            crate::tools::get_type_name($selfobj.py(), $selfobj.as_ptr()),
                            crate::tools::get_type_name($selfobj.py(), $otherobj.as_ptr()),
                        ),
                    ))
                }
            };
        }

        match cmp {
            pyo3::basic::CompareOp::Eq => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(qual) => qual,
                    Err(_) => return Ok(false),
                };

                Ok(self_.name == other.name)
            }
            pyo3::basic::CompareOp::Ne => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(qual) => qual,
                    Err(_) => return Ok(true),
                };

                Ok(self_.name != other.name)
            }
            pyo3::basic::CompareOp::Gt => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(qual) => qual,
                    Err(_) => return create_error!('>', self_, other),
                };

                Ok(self_.name > other.name)
            }
            pyo3::basic::CompareOp::Lt => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(qual) => qual,
                    Err(_) => return create_error!('<', self_, other),
                };

                Ok(self_.name < other.name)
            }
            pyo3::basic::CompareOp::Le => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(qual) => qual,
                    Err(_) => return create_error!("<=", self_, other),
                };

                Ok(self_.name <= other.name)
            }
            pyo3::basic::CompareOp::Ge => {
                let other = match other.extract::<pyo3::PyRef<'_, Self>>(self_.py()) {
                    Ok(qual) => qual,
                    Err(_) => return create_error!(">=", self_, other),
                };

                Ok(self_.name >= other.name)
            }
        }
    }

    fn __hash__(&self) -> u64 {
        let mut state = std::hash::DefaultHasher::new();
        std::hash::Hash::hash(&self.name, &mut state);
        state.finish()
    }

    fn __repr__(&self) -> String {
        format!(
            "xmarkup._rustlib.QualName(local={:?}, ns={:?}, prefix={:?})",
            self.name.local.as_ref(),
            self.name.ns.as_ref(),
            self.name.prefix.as_ref().map(|x| x.as_ref())
        )
    }
}
