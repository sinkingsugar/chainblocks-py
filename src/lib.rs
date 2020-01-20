#![allow(unused_imports)]

#[macro_use]

extern crate chainblocks;
extern crate pyo3;
extern crate ctor;

use ctor::ctor;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use chainblocks::core::Core;
use chainblocks::types::BaseArray;
use chainblocks::types::Types;
use chainblocks::chainblocksc::CBContext;
use chainblocks::chainblocksc::CBVar;
use chainblocks::types::common_type;
use chainblocks::core::registerBlock;
use chainblocks::block::Block;
use chainblocks::core::init;

struct MyTypes(Types);

impl pyo3::FromPyObject<'_> for MyTypes {
    fn extract(_: &'_ pyo3::types::PyAny)
               -> std::result::Result<Self, pyo3::PyErr> {
        unimplemented!()
    }
}

struct PyBlock {
    input_types: Box<Types>,
    output_types: Box<Types>,
    locals: PyObject,
}

impl Default for PyBlock {
    fn default() -> Self {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        let locals = PyDict::new(py);
        
        if let Err(e) = py.eval("print(\"Hello from Python...\")",
                                None,
                                Some(locals)) {
           e.print(py);
        }

        Self{
            input_types: Box::new(Types::from(vec![common_type::any()])),
            output_types: Box::new(Types::from(vec![common_type::any()])),
            locals: locals.to_object(py),
        }
    }
}

impl Block for PyBlock {
    fn name(&mut self) -> &str { "Py" }
    fn inputTypes(&mut self) -> &Types {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        let locals = self.locals.cast_as::<PyDict>(py).unwrap();
        if locals.contains("inputTypes").unwrap() {
            let types: MyTypes = locals
                .get_item("inputTypes")
                .unwrap()
                .extract()
                .unwrap();
            self.input_types = Box::new(types.0);
        }
        &self.input_types
    }
    fn outputTypes(&mut self) -> &Types { &self.output_types }
    fn activate(&mut self, _context: &CBContext, _input: &CBVar) -> CBVar { CBVar::default() }
}

#[ctor]
fn attach() {
    init();
    registerBlock::<PyBlock>("Py");
}

