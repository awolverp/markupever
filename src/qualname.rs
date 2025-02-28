use std::hash::Hasher;

#[inline(always)]
pub(super) fn repr_qualname(q: &treedom::markup5ever::QualName) -> String {
    format!(
        "QualName(local={:?}, ns={:?}, prefix={:?})",
        q.local.as_ref(),
        q.ns.as_ref(),
        q.prefix.as_ref().map(|x| x.as_ref())
    )
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
/// # Note
/// This type is immutable.
#[pyo3::pyclass(name = "QualName", module = "markupever._rustlib", frozen)]
pub struct PyQualName {
    pub name: treedom::markup5ever::QualName,
}

#[pyo3::pymethods]
impl PyQualName {
    /// Creates a new [`PyQualName`] instance
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

    /// The local name (e.g. `table` in `<furn:table>` above).
    #[getter]
    fn local(&self) -> String {
        self.name.local.to_string()
    }

    /// The namespace after resolution (e.g. https://furniture.rs in example above).
    #[getter]
    fn ns(&self) -> String {
        self.name.ns.to_string()
    }

    /// The prefix of qualified (e.g. furn in <furn:table> above).
    /// Optional (since some namespaces can be empty or inferred),
    /// and only useful for namespace resolution (since different prefix can still resolve to same namespace)
    #[getter]
    fn prefix(&self) -> Option<String> {
        self.name.prefix.as_ref().map(|x| x.to_string())
    }

    /// Copies the QualName
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
                match crate::tools::qualname_from_pyobject(self_.py(), &other) {
                    crate::tools::QualNameFromPyObjectResult::QualName(x) => {
                        Ok(x.name == self_.name)
                    }
                    crate::tools::QualNameFromPyObjectResult::Str(x) => Ok(self_.name.local == x),
                    crate::tools::QualNameFromPyObjectResult::Err(_) => Ok(false),
                }
            }
            pyo3::basic::CompareOp::Ne => {
                match crate::tools::qualname_from_pyobject(self_.py(), &other) {
                    crate::tools::QualNameFromPyObjectResult::QualName(x) => {
                        Ok(x.name != self_.name)
                    }
                    crate::tools::QualNameFromPyObjectResult::Str(x) => Ok(self_.name.local != x),
                    crate::tools::QualNameFromPyObjectResult::Err(_) => Ok(true),
                }
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
        repr_qualname(&self.name)
    }
}
