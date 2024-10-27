use std::fs;
use std:: {
    path::{ self, Path }
};

use clap::{Arg, ArgAction, Command};
use colored::Colorize;
use pyo3::prelude::*;
use pyo3::types::PyList;


fn cli() -> Command {
    return Command::new("rust-py-bridge")
        .about("Experimental Python check loading library")
        .version("0.1.0")
        .arg(
            Arg::new("file")
                .long("file")
                .value_name("file")
                .action(ArgAction::Set)
                .help("File to load")
                .default_value("app.py")
                .default_missing_value("app.py"),
        );
}

fn main() -> PyResult<()> {
    let matches = cli().get_matches();
    let filename = matches.get_one::<String>("file").unwrap();

    println!("Starting...");
    println!("File: {:?}", filename);

    let raw_path = Path::new("./e2e");
    let path = path::absolute(raw_path)?;
    println!("Absolute path: {}", path.display());

    let py_app_path = path.join(filename);
    let py_file_result = fs::read_to_string(py_app_path.clone());

    let py_app = match py_file_result {
        Ok(file) => file,
        Err(error) => {
            let err_message = format!("Problem opening the file {py_app_path:?}: {error_message}",
                error_message = error.to_string());
            panic!("{}", err_message.bold().red());
        }
    };

    println!("Trying to load: {:?}", py_app_path);

    let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
        let syspath = py
            .import_bound("sys")?
            .getattr("path")?
            .downcast_into::<PyList>()?;

        syspath.insert(0, &path)?;
        println!("{}: {}", "sys.path".green(), syspath);

        let py_module_result = PyModule::from_code_bound(py, &py_app, "", "");
        let py_module = match py_module_result {
            Ok(module) => module,
            Err(error) => {
                let err_message = format!("Problem creating module from {py_app_path:?}: {error_message}",
                    error_message = error.to_string());
                panic!("{}", err_message.bold().red());
            }
        };


        let app: Py<PyAny> = py_module
            .getattr("foo")?
            .into();
        app.call0(py)
    });

    println!("py: {}", from_python?);

    println!("Exiting!");

    Ok(())
}
