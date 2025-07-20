//! Language configuration dataclasses for Python bindings

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyType};
use sakurs_core::domain::language::config::{
    AbbreviationConfig as CoreAbbreviationConfig, ContextRule as CoreContextRule,
    EllipsisConfig as CoreEllipsisConfig, EnclosureConfig as CoreEnclosureConfig,
    EnclosurePair as CoreEnclosurePair, ExceptionPattern as CoreExceptionPattern,
    FastPattern as CoreFastPattern, LanguageConfig as CoreLanguageConfig,
    MetadataConfig as CoreMetadataConfig, RegexPattern as CoreRegexPattern,
    SentenceStarterConfig as CoreSentenceStarterConfig, SuppressionConfig as CoreSuppressionConfig,
    TerminatorConfig as CoreTerminatorConfig, TerminatorPattern as CoreTerminatorPattern,
};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::exceptions::InternalError;

/// Metadata configuration for a language
#[pyclass(name = "MetadataConfig")]
#[derive(Clone)]
pub struct MetadataConfig {
    #[pyo3(get, set)]
    pub code: String,
    #[pyo3(get, set)]
    pub name: String,
}

#[pymethods]
impl MetadataConfig {
    #[new]
    fn new(code: String, name: String) -> Self {
        Self { code, name }
    }

    fn __repr__(&self) -> String {
        format!("MetadataConfig(code='{}', name='{}')", self.code, self.name)
    }
}

/// Terminator pattern configuration
#[pyclass(name = "TerminatorPattern")]
#[derive(Clone)]
pub struct TerminatorPattern {
    #[pyo3(get, set)]
    pub pattern: String,
    #[pyo3(get, set)]
    pub name: String,
}

#[pymethods]
impl TerminatorPattern {
    #[new]
    fn new(pattern: String, name: String) -> Self {
        Self { pattern, name }
    }

    fn __repr__(&self) -> String {
        format!(
            "TerminatorPattern(pattern='{}', name='{}')",
            self.pattern, self.name
        )
    }
}

/// Terminator configuration
#[pyclass(name = "TerminatorConfig")]
#[derive(Clone)]
pub struct TerminatorConfig {
    #[pyo3(get, set)]
    pub chars: Vec<String>, // Python expects strings, not chars
    #[pyo3(get, set)]
    pub patterns: Vec<TerminatorPattern>,
}

#[pymethods]
impl TerminatorConfig {
    #[new]
    #[pyo3(signature = (chars, patterns=vec![]))]
    fn new(chars: Vec<String>, patterns: Vec<TerminatorPattern>) -> Self {
        Self { chars, patterns }
    }

    fn __repr__(&self) -> String {
        format!(
            "TerminatorConfig(chars={:?}, patterns=[{} items])",
            self.chars,
            self.patterns.len()
        )
    }
}

/// Context rule for ellipsis handling
#[pyclass(name = "ContextRule")]
#[derive(Clone)]
pub struct ContextRule {
    #[pyo3(get, set)]
    pub condition: String,
    #[pyo3(get, set)]
    pub boundary: bool,
}

#[pymethods]
impl ContextRule {
    #[new]
    fn new(condition: String, boundary: bool) -> Self {
        Self {
            condition,
            boundary,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ContextRule(condition='{}', boundary={})",
            self.condition, self.boundary
        )
    }
}

/// Exception pattern for ellipsis handling
#[pyclass(name = "ExceptionPattern")]
#[derive(Clone)]
pub struct ExceptionPattern {
    #[pyo3(get, set)]
    pub regex: String,
    #[pyo3(get, set)]
    pub boundary: bool,
}

#[pymethods]
impl ExceptionPattern {
    #[new]
    fn new(regex: String, boundary: bool) -> Self {
        Self { regex, boundary }
    }

    fn __repr__(&self) -> String {
        format!(
            "ExceptionPattern(regex='{}', boundary={})",
            self.regex, self.boundary
        )
    }
}

/// Ellipsis configuration
#[pyclass(name = "EllipsisConfig")]
#[derive(Clone)]
pub struct EllipsisConfig {
    #[pyo3(get, set)]
    pub treat_as_boundary: bool,
    #[pyo3(get, set)]
    pub patterns: Vec<String>,
    #[pyo3(get, set)]
    pub context_rules: Vec<ContextRule>,
    #[pyo3(get, set)]
    pub exceptions: Vec<ExceptionPattern>,
}

#[pymethods]
impl EllipsisConfig {
    #[new]
    #[pyo3(signature = (treat_as_boundary=true, patterns=vec!["...".to_string(), "…".to_string()], context_rules=vec![], exceptions=vec![]))]
    fn new(
        treat_as_boundary: bool,
        patterns: Vec<String>,
        context_rules: Vec<ContextRule>,
        exceptions: Vec<ExceptionPattern>,
    ) -> Self {
        Self {
            treat_as_boundary,
            patterns,
            context_rules,
            exceptions,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "EllipsisConfig(treat_as_boundary={}, patterns={:?}, context_rules=[{} items], exceptions=[{} items])",
            self.treat_as_boundary,
            self.patterns,
            self.context_rules.len(),
            self.exceptions.len()
        )
    }
}

/// Enclosure pair configuration
#[pyclass(name = "EnclosurePair")]
#[derive(Clone)]
pub struct EnclosurePair {
    #[pyo3(get, set)]
    pub open: String, // Python expects strings, not chars
    #[pyo3(get, set)]
    pub close: String,
    #[pyo3(get, set)]
    pub symmetric: bool,
}

#[pymethods]
impl EnclosurePair {
    #[new]
    #[pyo3(signature = (open, close, symmetric=false))]
    fn new(open: String, close: String, symmetric: bool) -> Self {
        Self {
            open,
            close,
            symmetric,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "EnclosurePair(open='{}', close='{}', symmetric={})",
            self.open, self.close, self.symmetric
        )
    }
}

/// Enclosure configuration
#[pyclass(name = "EnclosureConfig")]
#[derive(Clone)]
pub struct EnclosureConfig {
    #[pyo3(get, set)]
    pub pairs: Vec<EnclosurePair>,
}

#[pymethods]
impl EnclosureConfig {
    #[new]
    fn new(pairs: Vec<EnclosurePair>) -> Self {
        Self { pairs }
    }

    fn __repr__(&self) -> String {
        format!("EnclosureConfig(pairs=[{} items])", self.pairs.len())
    }
}

/// Fast pattern for suppression
#[pyclass(name = "FastPattern")]
#[derive(Clone)]
pub struct FastPattern {
    #[pyo3(get, set)]
    pub char: String, // Python expects strings, not chars
    #[pyo3(get, set)]
    pub line_start: bool,
    #[pyo3(get, set)]
    pub before: Option<String>,
    #[pyo3(get, set)]
    pub after: Option<String>,
}

#[pymethods]
impl FastPattern {
    #[new]
    #[pyo3(signature = (char, line_start=false, before=None, after=None))]
    fn new(char: String, line_start: bool, before: Option<String>, after: Option<String>) -> Self {
        Self {
            char,
            line_start,
            before,
            after,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "FastPattern(char='{}', line_start={}, before={:?}, after={:?})",
            self.char, self.line_start, self.before, self.after
        )
    }
}

/// Regex pattern for suppression
#[pyclass(name = "RegexPattern")]
#[derive(Clone)]
pub struct RegexPattern {
    #[pyo3(get, set)]
    pub pattern: String,
    #[pyo3(get, set)]
    pub description: Option<String>,
}

#[pymethods]
impl RegexPattern {
    #[new]
    #[pyo3(signature = (pattern, description=None))]
    fn new(pattern: String, description: Option<String>) -> Self {
        Self {
            pattern,
            description,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "RegexPattern(pattern='{}', description={:?})",
            self.pattern, self.description
        )
    }
}

/// Suppression configuration
#[pyclass(name = "SuppressionConfig")]
#[derive(Clone)]
pub struct SuppressionConfig {
    #[pyo3(get, set)]
    pub fast_patterns: Vec<FastPattern>,
    #[pyo3(get, set)]
    pub regex_patterns: Vec<RegexPattern>,
}

#[pymethods]
impl SuppressionConfig {
    #[new]
    #[pyo3(signature = (fast_patterns=vec![], regex_patterns=vec![]))]
    fn new(fast_patterns: Vec<FastPattern>, regex_patterns: Vec<RegexPattern>) -> Self {
        Self {
            fast_patterns,
            regex_patterns,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "SuppressionConfig(fast_patterns=[{} items], regex_patterns=[{} items])",
            self.fast_patterns.len(),
            self.regex_patterns.len()
        )
    }
}

/// Abbreviation configuration
#[pyclass(name = "AbbreviationConfig")]
pub struct AbbreviationConfig {
    #[pyo3(get, set)]
    pub categories: Py<PyDict>,
}

// Manual Clone implementation for AbbreviationConfig
impl Clone for AbbreviationConfig {
    fn clone(&self) -> Self {
        Python::with_gil(|py| Self {
            categories: self.categories.clone_ref(py),
        })
    }
}

#[pymethods]
impl AbbreviationConfig {
    #[new]
    #[pyo3(signature = (**kwargs))]
    fn new(py: Python, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let dict = PyDict::new(py);
        if let Some(kw) = kwargs {
            dict.update(kw.as_mapping())?;
        }
        Ok(Self {
            categories: dict.unbind(),
        })
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        let categories = self.categories.bind(py);
        let num_categories = categories.len();
        Ok(format!(
            "AbbreviationConfig(categories=[{num_categories} categories])"
        ))
    }

    fn __getitem__(&self, py: Python, key: &str) -> PyResult<Py<PyList>> {
        let categories = self.categories.bind(py);
        categories
            .get_item(key)?
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_string()))?
            .downcast::<PyList>()
            .map(|list| list.clone().unbind())
            .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Category must be a list"))
    }

    fn __setitem__(&self, py: Python, key: String, value: Vec<String>) -> PyResult<()> {
        let categories = self.categories.bind(py);
        let list = PyList::new(py, value)?;
        categories.set_item(key, list)?;
        Ok(())
    }
}

/// Sentence starter configuration
#[pyclass(name = "SentenceStarterConfig")]
pub struct SentenceStarterConfig {
    #[pyo3(get, set)]
    pub categories: Py<PyDict>,
    #[pyo3(get, set)]
    pub require_following_space: bool,
    #[pyo3(get, set)]
    pub min_word_length: usize,
}

// Manual Clone implementation for SentenceStarterConfig
impl Clone for SentenceStarterConfig {
    fn clone(&self) -> Self {
        Python::with_gil(|py| Self {
            categories: self.categories.clone_ref(py),
            require_following_space: self.require_following_space,
            min_word_length: self.min_word_length,
        })
    }
}

#[pymethods]
impl SentenceStarterConfig {
    #[new]
    #[pyo3(signature = (require_following_space=true, min_word_length=1, **kwargs))]
    fn new(
        py: Python,
        require_following_space: bool,
        min_word_length: usize,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        let dict = PyDict::new(py);
        if let Some(kw) = kwargs {
            dict.update(kw.as_mapping())?;
        }
        Ok(Self {
            categories: dict.unbind(),
            require_following_space,
            min_word_length,
        })
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        let categories = self.categories.bind(py);
        let num_categories = categories.len();
        Ok(format!(
            "SentenceStarterConfig(categories=[{} categories], require_following_space={}, min_word_length={})",
            num_categories, self.require_following_space, self.min_word_length
        ))
    }
}

/// Complete language configuration
#[pyclass(name = "LanguageConfig")]
#[derive(Clone)]
pub struct LanguageConfig {
    #[pyo3(get, set)]
    pub metadata: MetadataConfig,
    #[pyo3(get, set)]
    pub terminators: TerminatorConfig,
    #[pyo3(get, set)]
    pub ellipsis: EllipsisConfig,
    #[pyo3(get, set)]
    pub enclosures: EnclosureConfig,
    #[pyo3(get, set)]
    pub suppression: SuppressionConfig,
    #[pyo3(get, set)]
    pub abbreviations: AbbreviationConfig,
    #[pyo3(get, set)]
    pub sentence_starters: Option<SentenceStarterConfig>,
}

#[pymethods]
impl LanguageConfig {
    #[new]
    fn new(
        metadata: MetadataConfig,
        terminators: TerminatorConfig,
        ellipsis: EllipsisConfig,
        enclosures: EnclosureConfig,
        suppression: SuppressionConfig,
        abbreviations: AbbreviationConfig,
        sentence_starters: Option<SentenceStarterConfig>,
    ) -> Self {
        Self {
            metadata,
            terminators,
            ellipsis,
            enclosures,
            suppression,
            abbreviations,
            sentence_starters,
        }
    }

    /// Load configuration from TOML file
    #[classmethod]
    fn from_toml(_cls: &Bound<'_, PyType>, py: Python, path: PathBuf) -> PyResult<Self> {
        // Read TOML file
        let content = std::fs::read_to_string(&path)
            .map_err(|e| InternalError::FileNotFound(format!("Failed to read TOML file: {e}")))?;

        // Parse TOML into Core LanguageConfig
        let core_config: CoreLanguageConfig = toml::from_str(&content)
            .map_err(|e| InternalError::ConfigurationError(format!("Failed to parse TOML: {e}")))?;

        // Validate the configuration
        core_config.validate().map_err(|e| {
            InternalError::ConfigurationError(format!("Invalid configuration: {e}"))
        })?;

        // Convert to Python LanguageConfig
        Self::from_core_config(py, core_config)
    }

    /// Save configuration to TOML file
    fn to_toml(&self, py: Python, path: PathBuf) -> PyResult<()> {
        // Convert to Core LanguageConfig
        let core_config = self.to_core_config(py)?;

        // Validate before saving
        core_config.validate().map_err(|e| {
            InternalError::ConfigurationError(format!("Invalid configuration: {e}"))
        })?;

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(&core_config).map_err(|e| {
            InternalError::ConfigurationError(format!("Failed to serialize to TOML: {e}"))
        })?;

        // Write to file
        std::fs::write(&path, toml_str).map_err(|e| {
            InternalError::ProcessingError(format!("Failed to write TOML file: {e}"))
        })?;

        Ok(())
    }

    fn __repr__(&self) -> String {
        format!(
            "LanguageConfig(code='{}', name='{}')",
            self.metadata.code, self.metadata.name
        )
    }
}

// Conversion implementations
impl LanguageConfig {
    /// Convert from Core LanguageConfig to Python LanguageConfig
    pub fn from_core_config(py: Python, core: CoreLanguageConfig) -> PyResult<Self> {
        // Convert metadata
        let metadata = MetadataConfig {
            code: core.metadata.code,
            name: core.metadata.name,
        };

        // Convert terminators
        let chars = core
            .terminators
            .chars
            .into_iter()
            .map(|c| c.to_string())
            .collect();
        let patterns = core
            .terminators
            .patterns
            .into_iter()
            .map(|p| TerminatorPattern {
                pattern: p.pattern,
                name: p.name,
            })
            .collect();
        let terminators = TerminatorConfig { chars, patterns };

        // Convert ellipsis
        let context_rules = core
            .ellipsis
            .context_rules
            .into_iter()
            .map(|r| ContextRule {
                condition: r.condition,
                boundary: r.boundary,
            })
            .collect();
        let exceptions = core
            .ellipsis
            .exceptions
            .into_iter()
            .map(|e| ExceptionPattern {
                regex: e.regex,
                boundary: e.boundary,
            })
            .collect();
        let ellipsis = EllipsisConfig {
            treat_as_boundary: core.ellipsis.treat_as_boundary,
            patterns: core.ellipsis.patterns,
            context_rules,
            exceptions,
        };

        // Convert enclosures
        let pairs = core
            .enclosures
            .pairs
            .into_iter()
            .map(|p| EnclosurePair {
                open: p.open.to_string(),
                close: p.close.to_string(),
                symmetric: p.symmetric,
            })
            .collect();
        let enclosures = EnclosureConfig { pairs };

        // Convert suppression
        let fast_patterns = core
            .suppression
            .fast_patterns
            .into_iter()
            .map(|p| FastPattern {
                char: p.char.to_string(),
                line_start: p.line_start,
                before: p.before,
                after: p.after,
            })
            .collect();
        let regex_patterns = core
            .suppression
            .regex_patterns
            .into_iter()
            .map(|p| RegexPattern {
                pattern: p.pattern,
                description: p.description,
            })
            .collect();
        let suppression = SuppressionConfig {
            fast_patterns,
            regex_patterns,
        };

        // Convert abbreviations
        let abbrev_dict = PyDict::new(py);
        for (category, words) in core.abbreviations.categories {
            let word_list = PyList::new(py, words)?;
            abbrev_dict.set_item(category, word_list)?;
        }
        let abbreviations = AbbreviationConfig {
            categories: abbrev_dict.unbind(),
        };

        // Convert sentence starters
        let sentence_starters = core.sentence_starters.map(|ss| {
            let categories_dict = PyDict::new(py);
            for (category, words) in ss.categories {
                let word_list = PyList::new(py, words).unwrap();
                categories_dict.set_item(category, word_list).unwrap();
            }
            SentenceStarterConfig {
                categories: categories_dict.unbind(),
                require_following_space: ss.require_following_space,
                min_word_length: ss.min_word_length,
            }
        });

        Ok(Self {
            metadata,
            terminators,
            ellipsis,
            enclosures,
            suppression,
            abbreviations,
            sentence_starters,
        })
    }

    /// Convert from Python LanguageConfig to Core LanguageConfig
    pub fn to_core_config(&self, py: Python) -> PyResult<CoreLanguageConfig> {
        // Convert metadata
        let metadata = CoreMetadataConfig {
            code: self.metadata.code.clone(),
            name: self.metadata.name.clone(),
        };

        // Convert terminators
        let chars = self
            .terminators
            .chars
            .iter()
            .filter_map(|s| s.chars().next())
            .collect();
        let patterns = self
            .terminators
            .patterns
            .iter()
            .map(|p| CoreTerminatorPattern {
                pattern: p.pattern.clone(),
                name: p.name.clone(),
            })
            .collect();
        let terminators = CoreTerminatorConfig { chars, patterns };

        // Convert ellipsis
        let context_rules = self
            .ellipsis
            .context_rules
            .iter()
            .map(|r| CoreContextRule {
                condition: r.condition.clone(),
                boundary: r.boundary,
            })
            .collect();
        let exceptions = self
            .ellipsis
            .exceptions
            .iter()
            .map(|e| CoreExceptionPattern {
                regex: e.regex.clone(),
                boundary: e.boundary,
            })
            .collect();
        let ellipsis = CoreEllipsisConfig {
            treat_as_boundary: self.ellipsis.treat_as_boundary,
            patterns: self.ellipsis.patterns.clone(),
            context_rules,
            exceptions,
        };

        // Convert enclosures
        let pairs = self
            .enclosures
            .pairs
            .iter()
            .filter_map(|p| {
                let open_char = p.open.chars().next()?;
                let close_char = p.close.chars().next()?;
                Some(CoreEnclosurePair {
                    open: open_char,
                    close: close_char,
                    symmetric: p.symmetric,
                })
            })
            .collect();
        let enclosures = CoreEnclosureConfig { pairs };

        // Convert suppression
        let fast_patterns = self
            .suppression
            .fast_patterns
            .iter()
            .filter_map(|p| {
                p.char.chars().next().map(|c| CoreFastPattern {
                    char: c,
                    line_start: p.line_start,
                    before: p.before.clone(),
                    after: p.after.clone(),
                })
            })
            .collect();
        let regex_patterns = self
            .suppression
            .regex_patterns
            .iter()
            .map(|p| CoreRegexPattern {
                pattern: p.pattern.clone(),
                description: p.description.clone(),
            })
            .collect();
        let suppression = CoreSuppressionConfig {
            fast_patterns,
            regex_patterns,
        };

        // Convert abbreviations
        let abbrev_categories = self.abbreviations.categories.bind(py);
        let mut categories = HashMap::new();
        let keys = abbrev_categories.keys().iter();
        for key in keys {
            let key_str: String = key.extract()?;
            if let Some(value) = abbrev_categories.get_item(&key)? {
                let words: Vec<String> = value.extract()?;
                categories.insert(key_str, words);
            }
        }
        let abbreviations = CoreAbbreviationConfig { categories };

        // Convert sentence starters
        let sentence_starters = self.sentence_starters.as_ref().map(|ss| {
            let starter_categories = ss.categories.bind(py);
            let mut categories = HashMap::new();
            let keys = starter_categories.keys().iter();
            for key in keys {
                let key_str: String = key.extract().unwrap();
                if let Some(value) = starter_categories.get_item(&key).unwrap() {
                    let words: Vec<String> = value.extract().unwrap();
                    categories.insert(key_str, words);
                }
            }
            CoreSentenceStarterConfig {
                categories,
                require_following_space: ss.require_following_space,
                min_word_length: ss.min_word_length,
            }
        });

        Ok(CoreLanguageConfig {
            metadata,
            terminators,
            ellipsis,
            enclosures,
            suppression,
            abbreviations,
            sentence_starters,
        })
    }
}
