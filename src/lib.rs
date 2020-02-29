use crate::prelude::*;

pub mod app;
mod inputs;
mod prelude;
mod scenario;
mod template_engine;
mod views;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Path(String);

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::ops::Add for &Path {
    type Output = Path;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Path(String::default());

        if !self.0.is_empty() {
            result.0.push_str(&self.0);
        }

        if !self.0.is_empty() && !rhs.0.is_empty() {
            result.0.push('.');
        }

        if !rhs.0.is_empty() {
            result.0.push_str(&rhs.0);
        }
        result
    }
}

/// Represents the data entered in the inputs on the page.
///
/// Backed by a JSON object.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct InputsData(JsonValue);

impl Default for InputsData {
    fn default() -> Self {
        Self(JsonValue::Object(Default::default()))
    }
}

impl fmt::Display for InputsData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#}", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl InputsData {
    /// TODO
    ///
    /// Returns an error if the key in ill-formed in the case of an array access.
    fn insert_at(&mut self, path: &Path, value: JsonValue) -> Result<()> {
        *path
            .0
            .split('.')
            .fold(Ok(&mut self.0), |obj, segment: &str| {
                Ok(match obj? {
                    JsonValue::Object(obj) => obj
                        .entry(segment)
                        .or_insert(JsonValue::Object(Default::default())),
                    JsonValue::Array(arr) => {
                        let index = segment
                            .parse::<usize>()
                            .context("Key is invalid, the index is ill-formed for array access.")?;

                        // Extend the array if needed
                        let len = arr.len();
                        arr.resize_with(index.max(len), || JsonValue::Object(Default::default()));

                        arr.get_mut(index).unwrap()
                    }
                    _ => bail!(
                        "Failed to insert data, the data is ill-formed at segment: {}.",
                        segment
                    ),
                })
            })? = value;
        Ok(())
    }

    fn get_at(&self, path: &Path) -> Option<&JsonValue> {
        path.0
            .split('.')
            .fold(Some(&self.0), |obj, segment: &str| match obj {
                Some(JsonValue::Object(obj)) => obj.get(segment),
                Some(JsonValue::Array(arr)) => {
                    if let Ok(index) = segment.parse::<usize>() {
                        arr.get(index)
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }

    fn get_at_mut(&mut self, path: &Path) -> Option<&mut JsonValue> {
        path.0
            .split('.')
            .fold(Some(&mut self.0), |obj, segment: &str| match obj {
                Some(JsonValue::Object(obj)) => obj.get_mut(segment),
                Some(JsonValue::Array(arr)) => {
                    if let Ok(index) = segment.parse::<usize>() {
                        arr.get_mut(index)
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }

    fn resize_array_at(&mut self, path: &Path, new_size: usize) -> Result<()> {
        // Make sure that `path` points to an array
        if !self.get_at(&path).map(JsonValue::is_array).unwrap_or(false) {
            self.insert_at(&path, serde_json::json!([]))?;
        }

        self
            .get_at_mut(&path)
            .map(JsonValue::as_array_mut)
            .flatten()
            .unwrap() // guaranteed by the if above
            .resize_with(new_size, || JsonValue::Object(Default::default()));
        Ok(())
    }
}
