use pyo3::types::{PyAnyMethods, PyStringMethods, PyTypeMethods};

/// Returns the type name of a [`pyo3::ffi::PyObject`].
///
/// Returns `"<unknown>"` on failure.
pub fn get_type_name(obj: &pyo3::Bound<pyo3::PyAny>) -> String {
    let type_ = obj.get_type();
    type_.name().unwrap().to_str().unwrap().into()
}

#[derive(pyo3::FromPyObject)]
pub enum PyQualNameOrStr<'p> {
    QualName(pyo3::PyRef<'p, crate::qualname::PyQualName>),
    Str(pyo3::pybacked::PyBackedStr),
}

impl PyQualNameOrStr<'_> {
    pub fn into_qualname(self) -> treedom::markup5ever::QualName {
        match self {
            Self::QualName(q) => q.name.clone(),
            Self::Str(s) => treedom::markup5ever::QualName::new(
                None,
                treedom::markup5ever::namespace_url!(""),
                (*s).into(),
            ),
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
