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
