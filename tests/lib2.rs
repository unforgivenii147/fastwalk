use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::path::PathBuf;

fn to_py_path<'py>(py: Python<'py>, path: PathBuf) -> PyResult<&'py PyAny> {
    let pathlib = py.import("pathlib")?;
    pathlib.call_method1("Path", (path,))
}

#[pyfunction]
fn walk(py: Python<'_>, path: PathBuf, follow_links: Option<bool>) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);

    let entries = py.allow_threads(|| {
        let walker = jwalk::WalkDir::new(&path).follow_links(follow);
        let mut raw_paths = Vec::new();
        for entry in walker {
            match entry {
                Ok(e) => raw_paths.push(e.path()),
                Err(e) => return Err(e),
            }
        }
        Ok(raw_paths)
    });

    let raw_paths = entries.map_err(|e| {
        PyOSError::new_err(format!("Error walking directory: {}", e))
    })?;

    let mut results = Vec::with_capacity(raw_paths.len());
    for path in raw_paths {
        results.push(to_py_path(py, path)?);
    }

    Ok(results)
}

#[pyfunction]
fn walk_files(py: Python<'_>, path: PathBuf, follow_links: Option<bool>) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);

    let entries = py.allow_threads(|| {
        let walker = jwalk::WalkDir::new(&path).follow_links(follow);
        let mut raw_paths = Vec::new();
        for entry in walker {
            match entry {
                Ok(e) => {
                    if e.file_type().is_file() {
                        raw_paths.push(e.path());
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(raw_paths)
    });

    let raw_paths = entries.map_err(|e| {
        PyOSError::new_err(format!("Error walking directory: {}", e))
    })?;

    let mut results = Vec::with_capacity(raw_paths.len());
    for path in raw_paths {
        results.push(to_py_path(py, path)?);
    }

    Ok(results)
}

#[pyfunction]
fn walk_dirs(py: Python<'_>, path: PathBuf, follow_links: Option<bool>) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);

    let entries = py.allow_threads(|| {
        let walker = jwalk::WalkDir::new(&path).follow_links(follow);
        let mut raw_paths = Vec::new();
        for entry in walker {
            match entry {
                Ok(e) => {
                    if e.file_type().is_dir() {
                        raw_paths.push(e.path());
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(raw_paths)
    });

    let raw_paths = entries.map_err(|e| {
        PyOSError::new_err(format!("Error walking directory: {}", e))
    })?;

    let mut results = Vec::with_capacity(raw_paths.len());
    for path in raw_paths {
        results.push(to_py_path(py, path)?);
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

struct RawEntry {
    path: PathBuf,
    is_file: bool,
    is_dir: bool,
    is_symlink: bool,
    depth: usize,
}

#[pyfunction]
fn walk_with_metadata(
    py: Python<'_>,
    path: PathBuf,
    follow_links: Option<bool>,
    max_depth: Option<usize>,
    min_depth: Option<usize>,
) -> PyResult<Vec<Entry>> {
    let follow = follow_links.unwrap_or(false);

    let entries = py.allow_threads(|| {
        let mut walker = jwalk::WalkDir::new(&path).follow_links(follow);
        if let Some(max) = max_depth { walker = walker.max_depth(max); }
        if let Some(min) = min_depth { walker = walker.min_depth(min); }

        let mut raw_entries = Vec::new();
        for entry in walker {
            match entry {
                Ok(e) => {
                    let file_type = e.file_type();
                    raw_entries.push(RawEntry {
                        path: e.path(),
                        is_file: file_type.is_file(),
                        is_dir: file_type.is_dir(),
                        is_symlink: file_type.is_symlink(),
                        depth: e.depth(),
                    });
                }
                Err(e) => return Err(e),
            }
        }
        Ok(raw_entries)
    });

    let raw_entries = entries.map_err(|e| {
        PyOSError::new_err(format!("Error walking directory: {}", e))
    })?;

    let mut results = Vec::with_capacity(raw_entries.len());
    for re in raw_entries {
        let py_path = to_py_path(py, re.path)?.into();
        results.push(Entry {
            path: py_path,
            is_file: re.is_file,
            is_dir: re.is_dir,
            is_symlink: re.is_symlink,
            depth: re.depth,
        });
    }

    Ok(results)
}

#[pyfunction]
fn walk_parallel(
    py: Python<'_>,
    path: PathBuf,
    follow_links: Option<bool>,
    num_threads: Option<usize>,
) -> PyResult<Vec<&PyAny>> {
    let follow = follow_links.unwrap_or(false);
    let threads = num_threads.unwrap_or(0);

    let entries = py.allow_threads(|| {
        let walker = jwalk::WalkDir::new(&path)
            .follow_links(follow)
            .parallelism(jwalk::Parallelism::RayonNewPool(threads));
        
        let mut raw_paths = Vec::new();
        for entry in walker {
            match entry {
                Ok(e) => raw_paths.push(e.path()),
                Err(e) => return Err(e),
            }
        }
        Ok(raw_paths)
    });

    let raw_paths = entries.map_err(|e| {
        PyOSError::new_err(format!("Error walking directory: {}", e))
    })?;

    let mut results = Vec::with_capacity(raw_paths.len());
    for path in raw_paths {
        results.push(to_py_path(py, path)?);
    }

    Ok(results)
}

#[pymodule]
fn fastwalk(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(walk, m)?)?;
    m.add_function(wrap_pyfunction!(wrap_pyfunction!(walk_files, m)?)?)?;
    m.add_function(wrap_pyfunction!(walk_dirs, m)?)?;
    m.add_function(wrap_pyfunction!(walk_with_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(walk_parallel, m)?)?;
    m.add_class::<Entry>()?;
    Ok(())
}
