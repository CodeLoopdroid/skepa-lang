use crate::{RtArray, RtResult, RtString, RtValue};

pub fn len(array: &RtArray) -> i64 {
    array.len() as i64
}

pub fn is_empty(array: &RtArray) -> bool {
    array.is_empty()
}

pub fn first(array: &RtArray) -> RtResult<RtValue> {
    array.get(0)
}

pub fn last(array: &RtArray) -> RtResult<RtValue> {
    let len = array.len();
    array.get(len.saturating_sub(1))
}

pub fn join(array: &RtArray, sep: &RtString) -> RtResult<RtString> {
    let mut out = Vec::with_capacity(array.len());
    for item in array.iter() {
        out.push(item.expect_string()?.as_str().to_owned());
    }
    Ok(RtString::from(out.join(sep.as_str())))
}
