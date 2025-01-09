use super::qualname::make_qualname_from_pyobject;
use super::qualname::PyQualName;
use super::utils::{get_node_from_pyobject, make_repr};
use crate::core::arcdom;

/// An element node data
#[pyo3::pyclass(name = "ElementData", module = "markupselect._rustlib", frozen)]
pub struct PyElementData(pub arcdom::Node);

#[pyo3::pymethods]
impl PyElementData {
    #[new]
    #[pyo3(signature=(name, attrs, template=false, mathml_annotation_xml_integration_point=false, /))]
    pub(super) fn new(
        py: pyo3::Python<'_>,
        name: pyo3::PyObject,
        attrs: Vec<(pyo3::PyObject, String)>,
        template: bool,
        mathml_annotation_xml_integration_point: bool,
    ) -> pyo3::PyResult<Self> {
        let name = make_qualname_from_pyobject(py, &name)?;

        let mut attributes: arcdom::SizedSmallVec<(
            markup5ever::QualName,
            crate::core::send::AtomicTendril,
        )> = arcdom::SizedSmallVec::new();

        attributes
            .try_reserve(attrs.len())
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyMemoryError, _>(e.to_string()))?;

        for (key, val) in attrs.into_iter() {
            let key = make_qualname_from_pyobject(py, &key)?;
            attributes.push((key, val.into()));
        }

        let node = arcdom::Node::new(arcdom::ElementData::new(
            name,
            attributes.into_iter(),
            template,
            mathml_annotation_xml_integration_point,
        ));

        Ok(Self(node))
    }

    /// Copies the `self` and returns a new one
    pub(super) fn copy(&self) -> Self {
        Self(arcdom::Node::copy(&self.0))
    }

    #[getter]
    pub(super) fn name(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
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
    pub(super) fn set_name(
        &self,
        py: pyo3::Python<'_>,
        value: pyo3::PyObject,
    ) -> pyo3::PyResult<()> {
        let value = make_qualname_from_pyobject(py, &value)?;

        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .name = value;

        Ok(())
    }

    #[getter]
    pub(super) fn attrs(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let attrs = PyElementDataAttributes {
            node: self.0.clone(),
            index: std::sync::atomic::AtomicUsize::new(0),
            len: std::sync::atomic::AtomicUsize::new(0),
        };

        pyo3::Py::new(py, attrs).map(|x| x.into_any())
    }

    #[setter]
    pub(super) fn set_attrs(
        &self,
        py: pyo3::Python<'_>,
        attrs: Vec<(pyo3::PyObject, String)>,
    ) -> pyo3::PyResult<()> {
        let mut attributes: arcdom::SizedSmallVec<(
            markup5ever::QualName,
            crate::core::send::AtomicTendril,
        )> = arcdom::SizedSmallVec::new();

        attributes
            .try_reserve(attrs.len())
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyMemoryError, _>(e.to_string()))?;

        for (key, val) in attrs.into_iter() {
            let key = make_qualname_from_pyobject(py, &key)?;
            attributes.push((key, val.into()));
        }

        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .attrs = arcdom::ElementAttributeTrigger::new(attributes.into_iter());

        Ok(())
    }

    #[getter]
    pub(super) fn id(&self) -> Option<String> {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .attrs
            .id()
            .map(String::from)
    }

    #[getter]
    pub(super) fn classes(&self) -> Vec<String> {
        let mut classes = Vec::new();

        for cls in self
            .0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .attrs
            .classes()
        {
            classes.push(String::from(cls.as_ref()));
        }

        classes
    }

    #[getter]
    pub(super) fn template(&self) -> bool {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .template
    }

    #[setter]
    pub(super) fn set_template(&self, value: bool) {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .template = value;
    }

    #[getter]
    pub(super) fn mathml_annotation_xml_integration_point(&self) -> bool {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .mathml_annotation_xml_integration_point
    }

    #[setter]
    pub(super) fn set_mathml_annotation_xml_integration_point(&self, value: bool) {
        self.0
            .as_element()
            .expect("PyElementData holds a node other than element")
            .mathml_annotation_xml_integration_point = value;
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

/// An element node data
#[pyo3::pyclass(
    name = "ElementDataAttributes",
    module = "markupselect._rustlib",
    frozen
)]
pub struct PyElementDataAttributes {
    node: arcdom::Node,
    len: std::sync::atomic::AtomicUsize,
    index: std::sync::atomic::AtomicUsize,
}

#[pyo3::pymethods]
impl PyElementDataAttributes {
    #[new]
    pub(super) fn new() -> pyo3::PyResult<Self> {
        Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Use ElementNodeData.attrs property; don't use this constructor directly.",
        ))
    }

    /// Returns `len(self)` - length of the attributes vector.
    pub(super) fn __len__(&self) -> usize {
        self.node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element")
            .attrs
            .len()
    }

    /// Returns `bool(self)` - `true` if the vector is not empty
    pub(super) fn __bool__(&self) -> bool {
        !self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element")
            .attrs
            .is_empty()
    }

    /// Clears the attributes vector
    pub(super) fn clear(&self) {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        elem.attrs.clear();
    }

    /// Append a new `(QualName, str)` sequence to the vector
    pub(super) fn append(
        &self,
        py: pyo3::Python<'_>,
        value: (pyo3::PyObject, pyo3::PyObject),
    ) -> pyo3::PyResult<()> {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        let qual = make_qualname_from_pyobject(py, &value.0)?;

        let val = value.1.extract::<String>(py).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "the value argument #2 must be str",
            )
        })?;

        elem.attrs.push((qual, val.into()));
        Ok(())
    }

    /// Removes an item from the end of the vector and returns it.
    ///
    /// Raises `IndexError` if the vector is empty
    pub(super) fn pop(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::PyObject> {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        let item =
            elem.attrs
                .pop()
                .ok_or(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                    "pop from empty attributes",
                ))?;

        // drop guard
        std::mem::drop(elem);

        let tuple = pyo3::types::PyTuple::new(
            py,
            [
                pyo3::Py::new(py, PyQualName(parking_lot::Mutex::new(item.0.clone())))?.into_any(),
                pyo3::types::PyString::new(py, &item.1).into(),
            ],
        )?;

        Ok(tuple.into())
    }

    /// Returns `self[index]`
    pub(super) fn __getitem__(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
    ) -> pyo3::PyResult<pyo3::PyObject> {
        let elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

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

    /// Performs `self[index] = (QualName, str)`
    pub(super) fn __setitem__(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
        value: (pyo3::PyObject, pyo3::PyObject),
    ) -> pyo3::PyResult<()> {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        if index >= elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        let qual = make_qualname_from_pyobject(py, &value.0)?;

        let val = value.1.extract::<String>(py).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "the value argument #2 must be str",
            )
        })?;

        elem.attrs[index] = (qual, val.into());
        Ok(())
    }

    /// Performs `del self[index]`
    pub(super) fn __delitem__(&self, index: usize) -> pyo3::PyResult<()> {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        if index >= elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        elem.attrs.remove(index);
        Ok(())
    }

    /// Performs `del self[index]` but is O(1), because does not reorder the vector, and replace
    /// `self[index]` with last element.
    ///
    /// If the order is not important for you, use this method instead of `del self[index]`
    pub(super) fn swap_remove(&self, index: usize) -> pyo3::PyResult<()> {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        if index >= elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        elem.attrs.swap_remove(index);
        Ok(())
    }

    pub(super) fn insert(
        &self,
        py: pyo3::Python<'_>,
        index: usize,
        value: (pyo3::PyObject, pyo3::PyObject),
    ) -> pyo3::PyResult<()> {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        if index > elem.attrs.len() {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyIndexError, _>(
                "out of range",
            ));
        }

        let qual = make_qualname_from_pyobject(py, &value.0)?;

        let val = value.1.extract::<String>(py).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "the value argument #2 must be str",
            )
        })?;

        elem.attrs.insert(index, (qual, val.into()));
        Ok(())
    }

    /// Return first index of value.
    ///
    /// Raises ValueError if the value is not present.
    #[pyo3(signature=(value, start=0))]
    pub(super) fn index(
        &self,
        py: pyo3::Python<'_>,
        value: (pyo3::PyObject, pyo3::PyObject),
        start: usize,
    ) -> pyo3::PyResult<usize> {
        let elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        let qual = make_qualname_from_pyobject(py, &value.0)?;

        let val = value.1.extract::<String>(py).map_err(|_| {
            pyo3::PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "the value argument #2 must be str",
            )
        })?;

        let val = (qual, Into::<crate::core::send::AtomicTendril>::into(val));
        let mut iter = elem.attrs.iter();

        if start > 0 {
            iter.skip(start - 1)
                .position(|x| x == &val)
                .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(()))
        } else {
            iter.position(|x| x == &val)
                .ok_or_else(|| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(()))
        }
    }

    /// Sorts the slice with a comparison function, without preserving the initial order of equal elements.
    ///
    /// This sort is unstable (i.e., may reorder equal elements), in-place (i.e., does not allocate), and O(n * log(n)) worst-case.
    pub(super) fn sort(&self) {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        elem.attrs.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    }

    /// Removes consecutive duplicate elements.
    pub(super) fn dedup(&self) {
        let mut elem = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        elem.attrs.dedup();
    }

    /// Returns `iter(self)`
    ///
    /// Note that you cannot have multiple `iter(self)` in a same time.
    /// each one must be done before creating next one.
    pub fn __iter__(slf: pyo3::PyRef<'_, Self>) -> pyo3::PyResult<pyo3::PyRef<'_, Self>> {
        if slf.len.load(std::sync::atomic::Ordering::Relaxed) != 0 {
            return Err(pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "you can only call PyElementDataAttributes' __iter__() once in a time.",
            ));
        }

        slf.index.store(0, std::sync::atomic::Ordering::Relaxed);
        slf.len
            .store(slf.__len__(), std::sync::atomic::Ordering::Relaxed);
        Ok(slf)
    }

    /// Returns `next(self)`
    pub fn __next__(
        slf: pyo3::PyRef<'_, Self>,
        py: pyo3::Python<'_>,
    ) -> pyo3::PyResult<*mut pyo3::ffi::PyObject> {
        let elem = slf
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

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

    pub(super) fn __repr__(&self) -> String {
        let element = self
            .node
            .as_element()
            .expect("PyElementDataAttributes holds a node other than element");

        let mut writer = String::from("ElementDataAttributes([");

        let mut iter_ = element.attrs.iter();

        if let Some((key, val)) = iter_.next() {
            writer += &format!("({:?}, {:?})", &*key.local, val.as_ref());
        }

        for (key, val) in iter_ {
            writer += &format!(", ({:?}, {:?})", &*key.local, val.as_ref());
        }

        writer + "])"
    }
}
