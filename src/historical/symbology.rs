// TODO(cg): remove
#![allow(dead_code, unused)]

pub struct SymbologyClient<'a> {
    pub(crate) inner: &'a mut reqwest::Client,
}
