#![recursion_limit = "512"]

use crate::prelude::*;

mod agents;
pub mod app;
mod components;
mod inputs;
mod prelude;
mod scenario;
mod template_engine;
mod views;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Path(String);

impl Path {
    fn from_segments(segments: &[&str]) -> Self {
        Path(segments.join("."))
    }

    fn get_segments(&self) -> impl Iterator<Item = &str> {
        self.0.split('.').filter(|s| !s.is_empty())
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl core::ops::Add for &Path {
    type Output = Path;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Path::default();

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

impl From<&str> for Path {
    fn from(path: &str) -> Self {
        Path(path.to_string())
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

impl From<JsonValue> for InputsData {
    fn from(value: JsonValue) -> Self {
        InputsData(value)
    }
}

impl InputsData {
    /// TODO
    ///
    /// Returns an error if the key in ill-formed in the case of an array access.
    fn insert_at(&mut self, path: &Path, value: JsonValue) -> Result<()> {
        *path
            .get_segments()
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
                        arr.resize_with(len.max(index + 1), || {
                            JsonValue::Object(Default::default())
                        });

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
        path.get_segments()
            .try_fold(&self.0, |obj, segment: &str| match obj {
                JsonValue::Object(obj) => obj.get(segment),
                JsonValue::Array(arr) => {
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
        path.get_segments()
            .try_fold(&mut self.0, |obj, segment: &str| match obj {
                JsonValue::Object(obj) => obj.get_mut(segment),
                JsonValue::Array(arr) => {
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
        // Make sure that `path` points to an array.
        // Returns an error if the key is ill-formed or invalid because of array access
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

    fn remove_at(&mut self, path: &Path) -> Result<Option<JsonValue>> {
        let (base, last) = {
            let mut segments: Vec<_> = path.get_segments().collect();

            if segments.is_empty() {
                let mut previous = JsonValue::Object(Default::default());
                std::mem::swap(&mut previous, &mut self.0);
                return Ok(Some(previous));
            }

            let last = segments
                .pop()
                .ok_or_else(|| anyhow!("remove_at called with an empty path."))?;
            (Path::from_segments(&segments), last)
        };

        match self.get_at_mut(&base) {
            Some(JsonValue::Object(obj)) => Ok(obj.remove(last)),
            Some(JsonValue::Array(arr)) => match last.parse::<usize>() {
                Ok(index) if index < arr.len() => Ok(Some(arr.remove(index))),
                Ok(_) => Ok(None),
                Err(e) => bail!(
                    "Invalid key: '{}' not found in array '{}': {}",
                    last,
                    base,
                    e
                ),
            },
            Some(JsonValue::String(_)) => bail!("Cannot remove from String at '{}'", base),
            Some(JsonValue::Number(_)) => bail!("Cannot remove from Number at '{}'", base),
            Some(JsonValue::Bool(_)) => bail!("Cannot remove from Bool at '{}'", base),
            Some(JsonValue::Null) => bail!("Cannot remove from Null at '{}'", base),
            None => bail!("Invalid key: nothing at '{}'", base),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use serde_json::json;

    #[test]
    fn Path_display() {
        ["", "a.path", "with.0.index"]
            .iter()
            .for_each(|s| assert_eq!(&Path::from(*s).to_string(), s))
    }

    #[test]
    fn Path_add_refs() {
        [
            (&Path::from("left") + &Path::from("right"), "left.right"),
            (&Path::default() + &Path::from("right"), "right"),
            (&Path::from("left") + &Path::default(), "left"),
        ]
        .iter()
        .for_each(|(actual, expected)| assert_eq!(&actual.0, expected))
    }

    #[test]
    fn InputsData_insert_in_empty_data() {
        let mut data = InputsData(json!({}));
        data.insert_at(&Path::from("some.path"), json!("data"))
            .unwrap();
        assert_eq!(data.0, json!({"some": {"path": "data"}}));
    }

    #[test]
    fn InputsData_insert_at_empty_object() {
        let mut data = InputsData(json!({"some": {}}));
        data.insert_at(&Path::from("some.path"), json!("data"))
            .unwrap();
        assert_eq!(data.0, json!({"some": {"path": "data"}}));
    }

    #[test]
    fn InputsData_insert_in_number_fails() {
        let json = json!({"some": 42});
        let mut data = InputsData(json.clone());
        assert!(data
            .insert_at(&Path::from("some.path"), json!("data"))
            .is_err());
        assert_eq!(data.0, json);
    }

    #[test]
    fn InputsData_insert_at_array_fails_if_not_an_index() {
        let json = json!({"some": []});
        let mut data = InputsData(json.clone());
        assert!(data
            .insert_at(&Path::from("some.path"), json!("data"))
            .is_err());
        assert_eq!(data.0, json);
    }

    #[test]
    fn InputsData_insert_in_array_at_first_position() {
        let mut data = InputsData(json!({"some": []}));
        data.insert_at(&Path::from("some.0"), json!("data"))
            .unwrap();
        assert_eq!(data.0, json!({"some": ["data"]}));
    }

    #[test]
    fn InputsData_insert_in_array_at_middle() {
        let mut data = InputsData(json!({"some": []}));
        data.insert_at(&Path::from("some.3"), json!("data"))
            .unwrap();
        assert_eq!(data.0, json!({"some": [{}, {}, {}, "data"]}));
    }

    #[test]
    fn InputsData_insert_deeply() {
        let mut data: InputsData = json!({"some": {"complex": [{}, null, {"json": {}}]}}).into();
        data.insert_at(&Path::from("some.complex.2.json"), json!("data"))
            .unwrap();
        assert_eq!(
            data.0,
            json!({"some": {"complex": [{}, null, {"json": "data"}]}})
        );
    }

    #[test]
    fn InputsData_get_at_empty_path() {
        let data: InputsData = json!({"a": "b"}).into();
        assert_eq!(data.get_at(&Path::from("")), Some(&data.0));
        assert_eq!(data.get_at(&Path::from(".")), Some(&data.0));
        assert_eq!(data.get_at(&Path::from("..")), Some(&data.0));
        assert_eq!(data.get_at(&Path::from("...")), Some(&data.0));
    }

    #[test]
    fn InputsData_get_at() {
        let json = json!({"a": "b", "c": ["d", {"e": 1}, 2], "f": {"g": "h"}});
        let data = InputsData::from(json.clone());
        [
            ("", Some(&json)),
            ("a", Some(&json["a"])),
            ("foo", None),
            ("c", Some(&json["c"])),
            ("c.0", Some(&json["c"][0])),
            ("c.1", Some(&json["c"][1])),
            ("c.2", Some(&json["c"][2])),
            ("c.3", None),
            ("f", Some(&json["f"])),
            ("f.g", Some(&json["f"]["g"])),
            ("f.foo", None),
        ]
        .iter()
        .map(|(s, exp)| (Path::from(*s), exp))
        .for_each(|(p, expected)| assert_eq!(data.get_at(&p), *expected))
    }

    #[test]
    fn InputsData_get_at_same_as_get_at_mut() {
        let mut data: InputsData =
            json!({"a": "b", "c": ["d", {"e": 1}, 2], "f": {"g": "h"}}).into();
        [
            "", "a", "foo", "c", "c.0", "c.1", "c.2", "c.3", "f", "f.g", "f.foo",
        ]
        .iter()
        .map(|s| Path::from(*s))
        .for_each(|p| assert_eq!(data.get_at(&p).cloned(), data.get_at_mut(&p).cloned()))
    }

    #[test]
    fn InputsData_resize_array_from_zero() {
        let mut data: InputsData = json!({"a": []}).into();
        data.resize_array_at(&Path::from("a"), 5).unwrap();
        assert_eq!(data.0, json!({"a": [{}, {}, {}, {}, {}]}))
    }

    #[test]
    fn InputsData_resize_array_that_doesnt_exist_yet() {
        let mut data: InputsData = json!({"a": {}}).into();
        data.resize_array_at(&Path::from("a.b"), 2).unwrap();
        assert_eq!(data.0, json!({"a": {"b": [{}, {}]}}))
    }

    #[test]
    fn InputsData_resize_at_number_will_change_it_into_an_array() {
        let mut data: InputsData = json!({"a": 42}).into();
        data.resize_array_at(&Path::from("a"), 1).unwrap();
        assert_eq!(data.0, json!({"a": [{}]}))
    }

    #[test]
    fn InputsData_remove_at_empty_path() {
        let mut data: InputsData = json!({"a": "b"}).into();
        assert_eq!(
            data.remove_at(&Path::from("")).unwrap().unwrap(),
            json!({"a": "b"})
        );
        assert_eq!(data.0, json!({}))
    }

    #[test]
    fn InputsData_remove_at_from_beginning() {
        let mut data: InputsData = json!({"a": [1, 2, 3]}).into();
        assert_eq!(data.remove_at(&Path::from("a.0")).unwrap(), Some(json!(1)));
        assert_eq!(data.0, json!({"a": [2, 3]}));
        assert_eq!(data.remove_at(&Path::from("a.0")).unwrap(), Some(json!(2)));
        assert_eq!(data.0, json!({"a": [3]}));
        assert_eq!(data.remove_at(&Path::from("a.0")).unwrap(), Some(json!(3)));
        assert_eq!(data.0, json!({"a": []}));
        assert_eq!(data.remove_at(&Path::from("a.0")).unwrap(), None);
        assert_eq!(data.0, json!({"a": []}));
    }
}
