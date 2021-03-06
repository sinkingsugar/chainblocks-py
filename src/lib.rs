// On windows build with something like
// PYTHON_SYS_EXECUTABLE=C:/Python38/python.exe LIB=C:/Python38/Libs/python38.lib cargo +nightly build

#![allow(unused_imports)]
#[macro_use]
extern crate chainblocks;
extern crate ctor;
extern crate pyo3;

use chainblocks::block::Block;
use chainblocks::core::getRootPath;
use chainblocks::core::init;
use chainblocks::core::registerBlock;
use chainblocks::core::Core;
use chainblocks::types::common_type;
use chainblocks::types::Context;
use chainblocks::types::ParameterInfo;
use chainblocks::types::Parameters;
use chainblocks::types::Type;
use chainblocks::types::Types;
use chainblocks::types::Var;
use ctor::ctor;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyAny;
use pyo3::types::PyBool;
use pyo3::types::PyDict;
use pyo3::types::PyFloat;
use pyo3::types::PyInt;
use pyo3::types::PyList;
use pyo3::types::PyLong;
use pyo3::types::PyModule;
use pyo3::types::PyString;
use pyo3::types::PyTuple;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs;
use std::path::Path;

#[repr(transparent)] // force it same size of original
struct MyVarRef<'a>(&'a Var);

impl pyo3::ToPyObject for MyVarRef<'_> {
    fn to_object(&self, py: pyo3::Python<'_>) -> pyo3::PyObject {
        if let Ok(v) = i64::try_from(self.0) {
            v.to_object(py)
        } else if let Ok(v) = String::try_from(self.0) {
            v.to_object(py)
        } else if let Ok(v) = f64::try_from(self.0) {
            v.to_object(py)
        } else if let Ok(v) = bool::try_from(self.0) {
            v.to_object(py)
        } else if let Ok(v) = <&[Var]>::try_from(self.0) {
            let mut pov = Vec::<PyObject>::new();
            for var in v {
                let mv = MyVarRef(&var);
                pov.push(mv.to_object(py));
            }
            PyList::new(py, pov).to_object(py)
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
    script_path: Option<CString>,
    result: Option<PyObject>,
    result_seq: Vec<Var>,
}

impl Default for PyBlock {
    fn default() -> Self {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        Self {
            input_types: Box::new(Types::from(vec![common_type::any])),
            output_types: Box::new(Types::from(vec![common_type::any])),
            parameters: Box::new(Parameters::from(vec![ParameterInfo::from((
                "Script",
                "The relative path to the python's block script.",
                Types::from(vec![common_type::string]),
            ))])),
            module: None,
            instance: PyDict::new(py).to_object(py),
            activate: None,
            script_path: None,
            result: None,
            result_seq: vec![],
        }
    }
}

fn match_type(t: &PyAny) -> Type {
    if let Ok(name) = t.extract::<&str>() {
        match name {
            "None" => common_type::none,
            "Any" => common_type::any,
            "Int" => common_type::int,
            "Float" => common_type::float,
            "Bool" => common_type::bool,
            "String" => common_type::string,
            _ => {
                unimplemented!();
            }
        }
    } else if let Ok(list) = t.extract::<Vec<&str>>() {
        match list.as_slice() {
            ["Int"] => common_type::ints,
            ["Float"] => common_type::floats,
            ["Bool"] => common_type::bools,
            ["String"] => common_type::strings,
            _ => {
                unimplemented!();
            }
        }
    } else if let Ok(_list) = t.extract::<Vec<&PyAny>>() {
        unimplemented!();
    } else {
        unimplemented!();
    }
}

fn iterate_types(list: Vec<&PyAny>) -> Vec<Type> {
    let mut types = Vec::<Type>::new();
    for type_name in list {
        types.push(match_type(type_name));
    }
    types
}

fn iterate_params(list: &PyList) -> Vec<ParameterInfo> {
    let mut params = Vec::<ParameterInfo>::new();
    // always inject this as first param
    params.push(ParameterInfo::from((
        "Script",
        "The relative path to the python's block script.",
        Types::from(vec![common_type::string]),
    )));
    for t in list {
        if let Ok(param_info) = t.extract::<(&str, &str, Vec<&PyAny>)>() {
            params.push(ParameterInfo::from((
                param_info.0,
                param_info.1,
                Types::from(iterate_types(param_info.2)),
            )));
        }
    }
    params
}

impl PyBlock {
    fn to_var(&mut self, py: pyo3::Python, o: PyObject) -> Var {
        if let Ok(v) = o.cast_as::<PyBool>(py) {
            let value: bool = v.extract().unwrap();
            Var::from(value)
        } else if let Ok(v) = o.cast_as::<PyInt>(py) {
            let value: i64 = v.extract().unwrap();
            Var::from(value)
        } else if let Ok(v) = o.cast_as::<PyLong>(py) {
            let value: i64 = v.extract().unwrap();
            Var::from(value)
        } else if let Ok(v) = o.cast_as::<PyFloat>(py) {
            let value: f64 = v.extract().unwrap();
            Var::from(value)
        } else if let Ok(v) = o.cast_as::<PyString>(py) {
            unsafe {
                let value: &[u8] = v.as_bytes().unwrap();
                let cstr = CStr::from_bytes_with_nul_unchecked(value);
                Var::from(cstr)
            }
        } else if let Ok(v) = o.cast_as::<PyList>(py) {
            for value in v {
                let var = self.to_var(py, value.to_object(py));
                self.result_seq.push(var)
            }
            Var::from(&self.result_seq)
        } else {
            Var::default()
        }
    }
}

impl Block for PyBlock {
    fn name(&mut self) -> &str {
        "Py"
    }

    fn inputTypes(&mut self) -> &Types {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        if self.module.is_some() {
            if let Ok(module) = self.module.as_ref().unwrap().cast_as::<PyModule>(py) {
                if let Ok(input_types) = module.get("inputTypes") {
                    if let Ok(ares) =
                        input_types.call1(PyTuple::new(py, vec![self.instance.clone_ref(py)]))
                    {
                        if let Ok(list) = ares.extract::<Vec<&PyAny>>() {
                            self.input_types = Box::new(Types::from(iterate_types(list)));
                        }
                    }
                }
            }
        }
        &self.input_types
    }

    fn outputTypes(&mut self) -> &Types {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        if self.module.is_some() {
            if let Ok(module) = self.module.as_ref().unwrap().cast_as::<PyModule>(py) {
                if let Ok(output_types) = module.get("outputTypes") {
                    if let Ok(ares) =
                        output_types.call1(PyTuple::new(py, vec![self.instance.clone_ref(py)]))
                    {
                        if let Ok(list) = ares.extract::<Vec<&PyAny>>() {
                            self.output_types = Box::new(Types::from(iterate_types(list)));
                        }
                    }
                }
            }
        }
        &self.output_types
    }

    fn parameters(&mut self) -> Option<&Parameters> {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        if self.module.is_some() {
            if let Ok(module) = self.module.as_ref().unwrap().cast_as::<PyModule>(py) {
                if let Ok(output_types) = module.get("parameters") {
                    if let Ok(ares) =
                        output_types.call1(PyTuple::new(py, vec![self.instance.clone_ref(py)]))
                    {
                        if let Ok(list) = ares.downcast_ref::<PyList>() {
                            self.parameters = Box::new(Parameters::from(iterate_params(list)));
                        }
                    }
                }
            }
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
                    &script.file_name().unwrap().to_str().unwrap(),
                );
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
                            if let Err(e) = output_types
                                .call1(PyTuple::new(py, vec![self.instance.clone_ref(py)]))
                            {
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
                            if let Err(e) = output_types.call1(PyTuple::new(
                                py,
                                vec![
                                    self.instance.clone_ref(py),
                                    idx.to_object(py),
                                    arg.to_object(py),
                                ],
                            )) {
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
            0 => Var::from(self.script_path.as_ref()),
            _ => {
                let gil = pyo3::Python::acquire_gil();
                let py = gil.python();
                let mut res = Var::default();
                if self.module.is_some() {
                    if let Ok(module) = self.module.as_ref().unwrap().cast_as::<PyModule>(py) {
                        if let Ok(output_types) = module.get("getParam") {
                            let pres = PyObject::from(output_types).call1(
                                py,
                                PyTuple::new(
                                    py,
                                    vec![self.instance.clone_ref(py), idx.to_object(py)],
                                ),
                            );
                            match pres {
                                Ok(o) => {
                                    // Notice we recycle activation result here!
                                    self.result_seq.clear();
                                    self.result = Some(o.clone_ref(py));
                                    res = self.to_var(py, o);
                                }
                                Err(err) => {
                                    err.print(py);
                                }
                            }
                        }
                    }
                }
                res
            }
        }
    }

    fn activate(&mut self, _context: &Context, input: &Var) -> Var {
        let gil = pyo3::Python::acquire_gil();
        let py = gil.python();
        let arg = MyVarRef(input);
        let call = self.activate.as_ref().unwrap();
        let ares = call.call1(
            py,
            PyTuple::new(py, vec![self.instance.clone_ref(py), arg.to_object(py)]),
        );
        match ares {
            Ok(o) => {
                // clear previous activation garbage
                self.result_seq.clear();
                // store/replace result
                self.result = Some(o.clone_ref(py));
                // finally convert
                self.to_var(py, o)
            }
            Err(err) => {
                err.print(py);
                panic!("Py activation failed!")
            }
        }
    }
}

#[ctor]
fn attach() {
    init();
    registerBlock::<PyBlock>("Py");
}
