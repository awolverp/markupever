use pyo3::types::{PyStringMethods, PyTypeMethods};

pub unsafe fn get_type_name(py: pyo3::Python<'_>, obj: *mut pyo3::ffi::PyObject) -> String {
    let type_ = unsafe { (*obj).ob_type };

    if type_.is_null() {
        String::from("<unknown>")
    } else {
        let obj = unsafe { pyo3::types::PyType::from_borrowed_type_ptr(py, type_) };

        obj.name().unwrap().to_str().unwrap().into()
    }
}

pub fn qualname_from_pyobject(
    py: pyo3::Python<'_>,
    object: &pyo3::PyObject,
) -> pyo3::PyResult<treedom::markup5ever::QualName> {
    use pyo3::types::PyAnyMethods;

    unsafe {
        if pyo3::ffi::PyUnicode_Check(object.as_ptr()) == 1 {
            Ok(treedom::markup5ever::QualName::new(
                None,
                treedom::markup5ever::namespace_url!(""),
                object
                    .bind(py)
                    .extract::<String>()
                    .unwrap_unchecked()
                    .into(),
            ))
        } else {
            let pyqual: pyo3::PyRef<'_, crate::dom::PyQualName> = object
                .bind(py)
                .extract::<pyo3::PyRef<'_, crate::dom::PyQualName>>()?;

            Ok(pyqual.name.clone())
        }
    }
}
