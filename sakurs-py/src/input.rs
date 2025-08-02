use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::path::PathBuf;

/// Enum to represent different input types from Python
pub enum PyInput {
    Text(String),
    Bytes(Vec<u8>),
    Path(PathBuf),
    FileObject(PyObject),
}

impl PyInput {
    /// Extract input from a Python object, detecting its type
    pub fn from_py_object(_py: Python, obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        // Check if it's bytes first (before string)
        if let Ok(bytes) = obj.downcast::<PyBytes>() {
            return Ok(PyInput::Bytes(bytes.as_bytes().to_vec()));
        }

        // Check if it's a Path object (pathlib.Path)
        if let Ok(fspath_method) = obj.getattr("__fspath__") {
            if let Ok(path_str) = fspath_method.call0().and_then(|r| r.extract::<String>()) {
                return Ok(PyInput::Path(PathBuf::from(path_str)));
            }
        }

        // Check if it's a file-like object (has read() method)
        if obj.hasattr("read")? {
            return Ok(PyInput::FileObject(obj.clone().unbind()));
        }

        // Check if it's a string - this should be last as strings can represent paths
        if let Ok(text) = obj.extract::<String>() {
            // Check if it's an existing file path
            let path = PathBuf::from(&text);
            if path.exists() && path.is_file() {
                return Ok(PyInput::Path(path));
            }
            // Otherwise treat it as regular text
            return Ok(PyInput::Text(text));
        }

        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected str, bytes, Path, or file-like object with read() method",
        ))
    }

    /// Convert PyInput to text content and return a tuple with unused first element for compatibility
    pub fn into_core_input_and_text(self, py: Python, encoding: &str) -> PyResult<((), String)> {
        match self {
            PyInput::Text(text) => Ok(((), text)),

            PyInput::Bytes(bytes) => {
                // Decode bytes using the specified encoding
                let text = decode_bytes(&bytes, encoding)?;
                Ok(((), text))
            }

            PyInput::Path(path) => {
                // Read the file content as bytes first
                let bytes = std::fs::read(&path).map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                        "Failed to read file '{}': {}",
                        path.display(),
                        e
                    ))
                })?;
                // Decode with the specified encoding
                let text = decode_bytes(&bytes, encoding)?;
                Ok(((), text))
            }

            PyInput::FileObject(obj) => {
                // Read from file-like object
                let content = read_file_object(py, &obj, encoding)?;
                Ok(((), content))
            }
        }
    }
}

/// Read content from a Python file-like object
fn read_file_object(py: Python, obj: &PyObject, encoding: &str) -> PyResult<String> {
    // Call the read() method
    let read_result = obj.call_method0(py, "read")?;

    // Check if the result is bytes or string
    if let Ok(text) = read_result.extract::<String>(py) {
        // Text mode file object
        Ok(text)
    } else if let Ok(bytes_bound) = read_result.downcast_bound::<PyBytes>(py) {
        // Binary mode file object
        decode_bytes(bytes_bound.as_bytes(), encoding)
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "file.read() returned neither str nor bytes",
        ))
    }
}

/// Decode bytes to string using the specified encoding
fn decode_bytes(bytes: &[u8], encoding: &str) -> PyResult<String> {
    match encoding.to_lowercase().as_str() {
        "utf-8" | "utf8" => String::from_utf8(bytes.to_vec()).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Failed to decode bytes as UTF-8: {e}"
            ))
        }),
        "ascii" => {
            if bytes.is_ascii() {
                Ok(String::from_utf8_lossy(bytes).to_string())
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Failed to decode bytes as ASCII: non-ASCII characters found",
                ))
            }
        }
        "latin-1" | "latin1" | "iso-8859-1" | "iso8859-1" => {
            // Latin-1 is a single-byte encoding where each byte maps directly to a Unicode code point
            Ok(bytes.iter().map(|&b| b as char).collect())
        }
        _ => {
            // For other encodings, we could use Python's codec system
            // For now, return an error
            Err(PyErr::new::<pyo3::exceptions::PyLookupError, _>(format!(
                "Unsupported encoding: {encoding}. Supported encodings: utf-8, ascii, latin-1"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf8() {
        let bytes = "Hello, 世界!".as_bytes();
        let result = decode_bytes(bytes, "utf-8").unwrap();
        assert_eq!(result, "Hello, 世界!");
    }

    #[test]
    fn test_decode_ascii() {
        let bytes = b"Hello, World!";
        let result = decode_bytes(bytes, "ascii").unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_decode_latin1() {
        let bytes = &[72, 101, 108, 108, 111, 32, 233]; // "Hello é" in Latin-1
        let result = decode_bytes(bytes, "latin-1").unwrap();
        assert_eq!(result, "Hello é");
    }
}
