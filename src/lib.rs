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
use chainblocks::types::Types;
use chainblocks::chainblocksc::CBContext;
use chainblocks::chainblocksc::CBVar;
use chainblocks::types::common_type;
use chainblocks::core::registerBlock;
use chainblocks::block::Block;
use chainblocks::core::init;

struct PyBlock {
    inputTypes: Types,
    outputTypes: Types,
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
            inputTypes: Types::from(vec![common_type::any()]),
            outputTypes: Types::from(vec![common_type::any()]),
            locals: locals.to_object(py),
        }
    }
}

impl Block for PyBlock {
    fn name(&self) -> &str { "Py" }
    fn inputTypes(&self) -> &Types { &self.inputTypes  }
    fn outputTypes(&self) -> &Types { &self.outputTypes }
    fn activate(&self, _context: &CBContext, _input: &CBVar) -> CBVar { CBVar::default() }
}

#[ctor]
fn attach() {
    init();
    registerBlock::<PyBlock>("Py");
}

