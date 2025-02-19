use pyo3::types::{PyStringMethods, PyTypeMethods};

/// Returns the type name of a [`pyo3::ffi::PyObject`].
///
/// Returns `"<unknown>"` on failure.
pub unsafe fn get_type_name(py: pyo3::Python<'_>, obj: *mut pyo3::ffi::PyObject) -> String {
    let type_ = unsafe { (*obj).ob_type };

    if type_.is_null() {
        String::from("<unknown>")
    } else {
        let obj = unsafe { pyo3::types::PyType::from_borrowed_type_ptr(py, type_) };

        obj.name().unwrap().to_str().unwrap().into()
    }
}

pub enum QualNameFromPyObjectResult<'p> {
    QualName(pyo3::PyRef<'p, crate::qualname::PyQualName>),
    Str(String),
    Err(pyo3::PyErr),
}

impl QualNameFromPyObjectResult<'_> {
    pub fn into_qualname(self) -> pyo3::PyResult<treedom::markup5ever::QualName> {
        match self {
            Self::QualName(q) => Ok(q.name.clone()),
            Self::Str(s) => Ok(treedom::markup5ever::QualName::new(
                None,
                treedom::markup5ever::namespace_url!(""),
                s.into(),
            )),
            Self::Err(e) => Err(e),
        }
    }
}

pub fn qualname_from_pyobject<'a>(
    py: pyo3::Python<'a>,
    object: &pyo3::PyObject,
) -> QualNameFromPyObjectResult<'a> {
    use pyo3::types::PyAnyMethods;
    unsafe {
        if pyo3::ffi::PyUnicode_Check(object.as_ptr()) == 1 {
            QualNameFromPyObjectResult::Str(object.bind(py).extract::<String>().unwrap_unchecked())
        } else {
            match object
                .bind(py)
                .extract::<pyo3::PyRef<'_, crate::qualname::PyQualName>>()
            {
                Ok(x) => QualNameFromPyObjectResult::QualName(x),
                Err(e) => QualNameFromPyObjectResult::Err(e),
            }
        }
    }
}

pub const QUIRKS_MODE_FULL: u8 = 0;
pub const QUIRKS_MODE_LIMITED: u8 = 1;
pub const QUIRKS_MODE_OFF: u8 = 2;

pub fn convert_u8_to_quirks_mode(num: u8) -> Option<treedom::markup5ever::interface::QuirksMode> {
    match num {
        QUIRKS_MODE_FULL => Some(treedom::markup5ever::interface::QuirksMode::Quirks),
        QUIRKS_MODE_LIMITED => Some(treedom::markup5ever::interface::QuirksMode::LimitedQuirks),
        QUIRKS_MODE_OFF => Some(treedom::markup5ever::interface::QuirksMode::NoQuirks),
        _ => None,
    }
}

pub fn convert_quirks_mode_to_u8(q: treedom::markup5ever::interface::QuirksMode) -> u8 {
    match q {
        treedom::markup5ever::interface::QuirksMode::Quirks => QUIRKS_MODE_FULL,
        treedom::markup5ever::interface::QuirksMode::LimitedQuirks => QUIRKS_MODE_LIMITED,
        treedom::markup5ever::interface::QuirksMode::NoQuirks => QUIRKS_MODE_OFF,
    }
}
