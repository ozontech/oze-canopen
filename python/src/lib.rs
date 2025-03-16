mod oze_co;

use pyo3::{prelude::*};

/// A Python module implemented in Rust.
#[pymodule]
fn oze_canopen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<oze_co::OzeCO>()?;
    m.add_class::<oze_co::NmtCmd>()?;
    Ok(())
}
