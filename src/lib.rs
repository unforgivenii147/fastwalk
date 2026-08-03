use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString};
use std::path::PathBuf;

// Helper function to convert PathBuf to Python Path object
fn to_py_path<'py>(py: Python<'py>, path: PathBuf) -> PyResult<&'py PyAny> {
    let pathlib = py.import("pathlib")?;
    pathlib.call_method1("Path", (path,))
}

// Custom type that accepts both Path and str
#[derive(Clone)]
struct PathInput(PathBuf);

impl<'py> FromPyObject<'py> for PathInput {
    fn extract(obj: &'py PyAny) -> PyResult<Self> {
        // If it's a string, convert to PathBuf
        if let Ok(s) = obj.extract::<String>() {
            return Ok(PathInput(PathBuf::from(s)));
        }
        
        // If it's a Path object, extract the string representation
        if let Ok(path_str) = obj.call_method0("__str__") {
            if let Ok(s) = path_str.extract::<String>() {
                return Ok(PathInput(PathBuf::from(s)));
            }
        }
        
        Err(pyo3::exceptions::PyTypeError::new_err(
            "Expected a string or pathlib.Path object",
        ))
    }
}

// Helper function to convert PathInput to PathBuf
impl From<PathInput> for PathBuf {
    fn from(input: PathInput) -> Self {
        input.0
    }
}

#[pyfunction]
fn walk(py: Python<'_>, path: PathInput, follow_links: Option<bool>) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    let walker = jwalk::WalkDir::new(&path_buf).follow_links(follow);
    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                results.push(to_py_path(py, e.path())?);
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

#[pyfunction]
fn walk_files(py: Python<'_>, path: PathInput, follow_links: Option<bool>) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    let walker = jwalk::WalkDir::new(&path_buf).follow_links(follow);
    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                if e.file_type().is_file() {
                    results.push(to_py_path(py, e.path())?);
                }
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

#[pyfunction]
fn walk_dirs(py: Python<'_>, path: PathInput, follow_links: Option<bool>) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    let walker = jwalk::WalkDir::new(&path_buf).follow_links(follow);
    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                if e.file_type().is_dir() {
                    results.push(to_py_path(py, e.path())?);
                }
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

#[pyclass]
#[derive(Clone)]
struct Entry {
    #[pyo3(get)]
    path: Py<PyAny>,
    #[pyo3(get)]
    is_file: bool,
    #[pyo3(get)]
    is_dir: bool,
    #[pyo3(get)]
    is_symlink: bool,
    #[pyo3(get)]
    depth: usize,
}

#[pyfunction]
fn walk_with_metadata(
    py: Python<'_>,
    path: PathInput,
    follow_links: Option<bool>,
    max_depth: Option<usize>,
    min_depth: Option<usize>,
) -> PyResult<Vec<Entry>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    let mut walker = jwalk::WalkDir::new(&path_buf).follow_links(follow);

    if let Some(max) = max_depth {
        walker = walker.max_depth(max);
    }
    if let Some(min) = min_depth {
        walker = walker.min_depth(min);
    }

    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                let file_type = e.file_type();
                let py_path = to_py_path(py, e.path())?.into(); // Convert reference to a persistent Py object
                results.push(Entry {
                    path: py_path,
                    is_file: file_type.is_file(),
                    is_dir: file_type.is_dir(),
                    is_symlink: file_type.is_symlink(),
                    depth: e.depth(),
                });
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

#[pyfunction]
fn walk_parallel(
    py: Python<'_>,
    path: PathInput,
    follow_links: Option<bool>,
    num_threads: Option<usize>,
) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);
    let path_buf: PathBuf = path.into();
    let threads = num_threads.unwrap_or(0); // 0 = auto

    let walker = jwalk::WalkDir::new(&path_buf)
        .follow_links(follow)
        .parallelism(jwalk::Parallelism::RayonNewPool(threads));

    let mut results = Vec::new();

    for entry in walker {
        match entry {
            Ok(e) => {
                results.push(to_py_path(py, e.path())?);
            }
            Err(e) => {
                return Err(PyOSError::new_err(format!(
                    "Error walking directory: {}",
                    e
                )));
            }
        }
    }
    Ok(results)
}

#[pymodule]
fn fastwalk(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(walk, m)?)?;
    m.add_function(wrap_pyfunction!(walk_files, m)?)?;
    m.add_function(wrap_pyfunction!(walk_dirs, m)?)?;
    m.add_function(wrap_pyfunction!(walk_with_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(walk_parallel, m)?)?;
    m.add_class::<Entry>()?;
    Ok(())
}
