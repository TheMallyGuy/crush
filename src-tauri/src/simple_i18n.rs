// simple i18n
// the rust crate i18n-rust doesnt support my project structure, so i made one myself

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

type Translations = HashMap<String, HashMap<String, String>>;

#[derive(Clone)]
pub struct I18n(Arc<Inner>);

struct Inner {
    translations: RwLock<Translations>,
    locale: RwLock<String>,
    fallback: String,
    dir: PathBuf,
}

impl I18n {
    pub fn new(lang_dir: impl Into<PathBuf>, default_locale: &str) -> Result<Self, String> {
        let dir = lang_dir.into();
        let mut translations = HashMap::new();
        translations.insert(
            default_locale.to_string(),
            Self::load_file(&dir, default_locale)?,
        );
        Ok(Self(Arc::new(Inner {
            translations: RwLock::new(translations),
            locale: RwLock::new(default_locale.to_string()),
            fallback: default_locale.to_string(),
            dir,
        })))
    }

    pub fn empty(default_locale: &str) -> Self {
        let mut translations = HashMap::new();
        translations.insert(default_locale.to_string(), HashMap::new());
        Self(Arc::new(Inner {
            translations: RwLock::new(translations),
            locale: RwLock::new(default_locale.to_string()),
            fallback: default_locale.to_string(),
            dir: PathBuf::new(),
        }))
    }

    pub fn set_locale(&self, locale: &str) -> Result<(), String> {
        if !self.0.translations.read().unwrap().contains_key(locale) {
            let flat = Self::load_file(&self.0.dir, locale)?;
            self.0
                .translations
                .write()
                .unwrap()
                .insert(locale.to_string(), flat);
        }
        *self.0.locale.write().unwrap() = locale.to_string();
        Ok(())
    }

    pub fn locale(&self) -> String {
        self.0.locale.read().unwrap().clone()
    }

    pub fn t(&self, key: &str) -> String {
        let locale = self.0.locale.read().unwrap().clone();
        self.resolve(&locale, key)
    }

    pub fn t_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut result = self.t(key);
        for (placeholder, value) in args {
            result = result.replace(&format!("{{{placeholder}}}"), value);
        }
        result
    }

    pub fn t_in(&self, locale: &str, key: &str) -> String {
        self.resolve(locale, key)
    }

    pub fn t_in_args(&self, locale: &str, key: &str, args: &[(&str, &str)]) -> String {
        let mut result = self.t_in(locale, key);
        for (placeholder, value) in args {
            result = result.replace(&format!("{{{placeholder}}}"), value);
        }
        result
    }

    pub fn has(&self, key: &str) -> bool {
        let locale = self.0.locale.read().unwrap().clone();
        let translations = self.0.translations.read().unwrap();
        translations
            .get(&locale)
            .map(|m| m.contains_key(key))
            .unwrap_or(false)
            || translations
                .get(&self.0.fallback)
                .map(|m| m.contains_key(key))
                .unwrap_or(false)
    }

    fn resolve(&self, locale: &str, key: &str) -> String {
        let translations = self.0.translations.read().unwrap();
        if let Some(v) = translations.get(locale).and_then(|m| m.get(key)) {
            return v.clone();
        }
        if locale != self.0.fallback {
            if let Some(v) = translations.get(&self.0.fallback).and_then(|m| m.get(key)) {
                return v.clone();
            }
        }
        key.to_string()
    }

    fn load_file(dir: &Path, locale: &str) -> Result<HashMap<String, String>, String> {
        let path = dir.join(locale).join("default.json");
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        // Many of the translation files are saved with a UTF-8 BOM, which
        // serde_json rejects ("expected value at line 1 column 1"). Strip it.
        let raw = raw.strip_prefix('\u{feff}').unwrap_or(&raw);
        let json: serde_json::Value = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid JSON in {}: {e}", path.display()))?;
        let mut flat = HashMap::new();
        Self::flatten_json(&json, "", &mut flat);
        Ok(flat)
    }

    fn flatten_json(value: &serde_json::Value, prefix: &str, flat: &mut HashMap<String, String>) {
        match value {
            serde_json::Value::Object(map) => {
                for (k, v) in map {
                    let key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{prefix}.{k}")
                    };
                    Self::flatten_json(v, &key, flat);
                }
            }
            serde_json::Value::String(s) => {
                flat.insert(prefix.to_string(), s.clone());
            }
            other => {
                flat.insert(prefix.to_string(), other.to_string());
            }
        }
    }
}

#[macro_export]
macro_rules! t {
    ($i18n:expr, $key:expr) => { $i18n.t($key) };
    ($i18n:expr, $key:expr, $val:literal) => {{
        let raw = $i18n.t($key);
        let val_str = $val.to_string();
        let replaced = if let Some(open) = raw.find('{') {
            if let Some(close) = raw[open..].find('}') {
                format!("{}{}{}", &raw[..open], val_str, &raw[open + close + 1..])
            } else { raw }
        } else { raw };
        replaced
    }};
    ($i18n:expr, $key:expr, $($name:ident = $val:expr),+ $(,)?) => {{
        let args: &[(&str, &str)] = &[$((stringify!($name), $val)),+];
        $i18n.t_args($key, args)
    }};
}
