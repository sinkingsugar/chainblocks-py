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
use pyo3::types::PyLong;
use pyo3::types::PyFloat;
use pyo3::types::PyString;
use pyo3::types::PyTuple;
use pyo3::types::PyModule;
use pyo3::types::PyList;
use chainblocks::core::Core;
use chainblocks::types::BaseArray;
use chainblocks::types::Types;
use chainblocks::types::Type;
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
struct MyVarRef<'a>(&'a Var);

impl pyo3::FromPyObject<'_> for MyVar {
    fn extract(o: &'_ pyo3::types::PyAny)
               -> std::result::Result<Self, pyo3::PyErr> {
        if let Ok(v) = o.downcast_ref::<PyInt>() {
            let value: i64 = v.extract().unwrap();
            Ok(MyVar(Var::from(value)))
        } else if let Ok(v) = o.downcast_ref::<PyLong>() {
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
            Ok(MyVar(Var::from(())))
        }
    }
}

impl pyo3::ToPyObject for MyVarRef<'_> {
    fn to_object(&self, py: pyo3::Python<'_>) -> pyo3::PyObject {
        if let Ok(v) = i64::try_from(self.0) {
            v.to_object(py)
        } else if let Ok(v) = String::try_from(self.0) {
            v.to_object(py)
        } else if let Ok(v) = f64::try_from(self.0) {
            v.to_object(py)
        } else {
            py.None()
        }
    }
}

struct PyBlock {
    input_types: Box<Types>,
    output_types: Box<Types>,
    parameters: Box<Parameters>,
    module: Option<PyObject>,
    instance: PyObject,
    activate: Option<PyObject>,
    result: Option<PyObject>,
    script_path: Option<CString>,
}

impl Default for PyBlock {
    fn default() -> Self {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
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
            module: None,
            instance: PyDict::new(py).to_object(py),
            activate: None,
            result: None,
            script_path: None,
        }
    }
}

fn match_type(name: &str) -> Type {
    match name {
        "String" => { common_type::string() }
        "None" => { common_type::none() }
        "Any" => { common_type::any() }
        "Int" => { common_type::int() }
        _ => { unimplemented!(); }
    }
}

fn iterate_types(list: Vec<&str>) -> Vec<Type> {
    let mut types = Vec::<Type>::new();
    for type_name in list {
        types.push(match_type(type_name));      
    }
    types
}

fn iterate_params(list: &PyList) -> Vec<ParameterInfo> {
    let mut params = Vec::<ParameterInfo>::new();
    // always inject this as first param
    params.push(ParameterInfo::from(
                    (
                        "Script",
                        "The relative path to the python's block script.",
                        Types::from(vec![common_type::string()])
                    )));
    for t in list {
        if let Ok(param_info) = t.extract::<(&str, &str, Vec<&str>)>() {
            params.push(ParameterInfo::from((
                param_info.0,
                param_info.1,
                Types::from(iterate_types(param_info.2)))));
        }
    }
    params
}

impl Block for PyBlock {
    fn name(&mut self) -> &str { "Py" }

    fn inputTypes(&mut self) -> &Types {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        if self.module.is_some() {
            if let Ok(module) = self.module.as_ref().unwrap().cast_as::<PyModule>(py) {
                if let Ok(input_types) = module.get("inputTypes") {
                    if let Ok(ares) = input_types.call1(PyTuple::new(py, vec![self.instance.clone_ref(py)])) {
                        if let Ok(list) = ares.extract::<Vec<&str>>() {
                            self.input_types = Box::new(Types::from(iterate_types(list)));
                        }}}}
        }       
        &self.input_types
    }

    fn outputTypes(&mut self) -> &Types {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        if self.module.is_some() {
            if let Ok(module) = self.module.as_ref().unwrap().cast_as::<PyModule>(py) {
                if let Ok(output_types) = module.get("outputTypes") {
                    if let Ok(ares) = output_types.call1(PyTuple::new(py, vec![self.instance.clone_ref(py)])) {
                        if let Ok(list) = ares.extract::<Vec<&str>>() {
                            self.output_types = Box::new(Types::from(iterate_types(list)));
                        }}}}
        }       
        &self.output_types
    }

    fn parameters(&mut self) -> Option<&Parameters> {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        if self.module.is_some() {
            if let Ok(module) = self.module.as_ref().unwrap().cast_as::<PyModule>(py) {
                if let Ok(output_types) = module.get("parameters") {
                    if let Ok(ares) = output_types.call1(PyTuple::new(py, vec![self.instance.clone_ref(py)])) {
                        if let Ok(list) = ares.downcast_ref::<PyList>() {
                            self.parameters = Box::new(Parameters::from(iterate_params(list)));
                        }}}}
        }       
        Some(&self.parameters)
    }

    fn setParam(&mut self, idx: i32, value: &Var) {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        match idx {
            0 => {
                let path = Path::new(getRootPath());
                let vstr = CString::try_from(value).unwrap();
                let script_name = vstr.to_str().unwrap();
                let script = Path::new(script_name);
                let fullpath = path.join(script);
                let code = fs::read_to_string(fullpath).unwrap();
                let mres = PyModule::from_code(
                    py,
                    &code,
                    &script_name,
                    &script
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap());
                match mres {
                    Err(e) => {
                        e.print(py);
                    }
                    Ok(module) => {
                        if let Ok(activation) = module.get("activate") {
                            self.activate = Some(PyObject::from(activation));
                        }
                        // also call setup here
                        if let Ok(output_types) = module.get("setup") {
                            if let Err(e) = output_types.call1(
                                PyTuple::new(py, vec![self.instance.clone_ref(py)])) {
                                e.print(py);
                            }
                        }
                        self.module = Some(module.to_object(py));
                    }
                }
                // finally store the string
                self.script_path = Some(vstr);
            }
            _ => {
                // send to py side
                if self.module.is_some() {
                    if let Ok(module) = self.module.as_ref().unwrap().cast_as::<PyModule>(py) {
                        if let Ok(output_types) = module.get("setParam") {
                            let arg = MyVarRef(value);
                            if let Err(e) = output_types.call1(
                                PyTuple::new(py, vec![
                                    self.instance.clone_ref(py),
                                    idx.to_object(py),
                                    arg.to_object(py)])) {
                                e.print(py);
                            }
                        }
                    }
                }
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
        let arg = MyVarRef(input);
        let call =  self.activate.as_ref().unwrap();
        let ares = call.call1(py, PyTuple::new(py, vec![self.instance.clone_ref(py), arg.to_object(py)]));
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

