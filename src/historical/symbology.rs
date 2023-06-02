pub struct SymbologyClient<'a> {
    pub(crate) inner: &'a mut reqwest::Client,
}
