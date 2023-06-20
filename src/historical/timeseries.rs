// TODO(cg): remove
#![allow(dead_code, unused)]

use dbn::record_ref::RecordRef;

pub struct TimeseriesClient<'a> {
    pub(crate) inner: &'a mut reqwest::Client,
}

impl TimeseriesClient<'_> {
    pub async fn get_range<F>(&mut self, callback: F)
    where
        F: FnMut(RecordRef) -> KeepGoing,
    {
        // TODO: or async_stream
        todo!()
    }

    pub async fn get_range_persist(&mut self) {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeepGoing {
    Continue,
    Stope,
}
