use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtString(Rc<str>);

impl RtString {
    pub fn new(value: impl Into<String>) -> Self {
        Self(Rc::<str>::from(value.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn len_chars(&self) -> usize {
        self.0.chars().count()
    }
}

impl From<&str> for RtString {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RtString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
