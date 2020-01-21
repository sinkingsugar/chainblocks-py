#![allow(unused_imports)]

#[macro_use]

extern crate chainblocks;
extern crate pyo3;
extern crate ctor;

use ctor::ctor;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::types::PyInt;
use pyo3::types::PyFloat;
use pyo3::types::PyString;
use pyo3::types::PyTuple;
use chainblocks::core::Core;
use chainblocks::types::BaseArray;
use chainblocks::types::Types;
use chainblocks::types::Var;
use chainblocks::types::Context;
use chainblocks::types::ParameterInfo;
use chainblocks::types::Parameters;
use chainblocks::types::common_type;
use chainblocks::core::registerBlock;
use chainblocks::core::getRootPath;
use chainblocks::block::Block;
use chainblocks::core::init;
use std::path::Path;
use std::fs;
use std::convert::TryFrom;
use std::ffi::CString;

struct MyVar(Var);

impl pyo3::FromPyObject<'_> for MyVar {
    fn extract(o: &'_ pyo3::types::PyAny)
               -> std::result::Result<Self, pyo3::PyErr> {
        if let Ok(v) = o.downcast_ref::<PyInt>() {
            let value: i64 = v.extract().unwrap();
            Ok(MyVar(Var::from(value)))
        } else if let Ok(v) = o.downcast_ref::<PyFloat>() {
            let value: f64 = v.extract().unwrap();
            Ok(MyVar(Var::from(value)))
        } else if let Ok(v) = o.downcast_ref::<PyString>() {
            let value: &[u8] = v.as_bytes().unwrap();
            let cbstr = value.as_ptr() as chainblocks::types::String;
            Ok(MyVar(Var::from(cbstr)))
        } else {
            unimplemented!()
        }
    }
}

struct PyBlock {
    input_types: Box<Types>,
    output_types: Box<Types>,
    parameters: Box<Parameters>,
    locals: PyObject,
    activate: Option<PyObject>,
    result: Option<PyObject>,
    script_path: Option<CString>,
}

impl Default for PyBlock {
    fn default() -> Self {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        let locals = PyDict::new(py);
        Self{
            input_types: Box::new(Types::from(vec![common_type::any()])),
            output_types: Box::new(Types::from(vec![common_type::any()])),
            parameters: Box::new(Parameters::from(vec![
                ParameterInfo::from(
                    (
                        "Script",
                        "The relative path to the python's block script.",
                        Types::from(vec![common_type::string()])
                    ))
            ])),    
            locals: locals.to_object(py),
            activate: None,
            result: None,
            script_path: None,
        }
    }
}

impl Block for PyBlock {
    fn name(&mut self) -> &str { "Py" }

    fn inputTypes(&mut self) -> &Types {
        // let gil = pyo3::Python::acquire_gil();
        // let py = gil.python();
        // let locals = self.locals.cast_as::<PyDict>(py).unwrap();
        // if locals.contains("inputTypes").unwrap() {
        //     let types: String = locals
        //         .get_item("inputTypes")
        //         .unwrap()
        //         .extract()
        //         .unwrap();
        //     // self.input_types = Box::new(types.0);
        // }
        
        &self.input_types
    }

    fn outputTypes(&mut self) -> &Types {
        // let gil = pyo3::Python::acquire_gil();
        // let py = gil.python();
        // let locals = self.locals.cast_as::<PyDict>(py).unwrap();
        // if locals.contains("outputTypes").unwrap() {
        //     let types: String = locals
        //         .get_item("outputTypes")
        //         .unwrap()
        //         .extract()
        //         .unwrap();
        //     // self.output_types = Box::new(types.0);
        // }
        
        &self.output_types
    }

    fn parameters(&mut self) -> Option<&Parameters> {
        Some(&self.parameters)
    }

    fn setParam(&mut self, idx: i32, value: &Var) {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        let locals = self.locals.cast_as::<PyDict>(py).unwrap();
        match idx {
            0 => {
                // script name fs
                let path = Path::new(getRootPath());
                let vstr = CString::try_from(value).unwrap();
                let script = Path::new(vstr.to_str().unwrap());
                let fullpath = path.join(script);
                let code = fs::read_to_string(fullpath).unwrap();
                if let Err(e) = py.run(&code,
                                       None,
                                       Some(locals)) {
                    e.print(py);
                } else {
                    if locals.contains("activate").unwrap() {
                        let lmbd = locals.get_item("activate").unwrap();
                        self.activate = Some(PyObject::from(lmbd));
                    }
                }
                // finally store the string
                self.script_path = Some(vstr);
            }
            _ => {
                // idx-- and send to py side
            }
        }
    }

    fn getParam(&mut self, idx: i32) -> Var {
        match idx {
            0 => { Var::from(self.script_path.as_ref()) }
            _ => { Var::from(()) }
        }
    }
    
    fn activate(&mut self, _context: &Context, input: &Var) -> Var {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        let args = MyVar(*input);
        let call =  self.activate.as_ref().unwrap();
        let ares = call.call1(py, PyTuple::new(py, vec![PyTuple::empty(py)])); //TODO
        match ares {
            Ok(output) => {
                // convert/extract
                let res: MyVar = output.extract(py).unwrap();
                // store result to keep refs
                // also will dec ref old one
                self.result = Some(output);
                res.0
            }
            Err(err) => {
                err.print(py);
                Var::default()
            }
        }
    }
}

#[ctor]
fn attach() {
    init();
    registerBlock::<PyBlock>("Py");
}

