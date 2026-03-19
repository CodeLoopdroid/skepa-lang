#[path = "../../common.rs"]
mod common;

mod cfg;
mod const_fold;
mod copy_prop;
mod dce;
mod inline;
mod r#loop;
mod pipeline;
mod runtime;
mod strength;
