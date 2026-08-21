#[derive(Debug, Clone)]
pub(crate) struct Workspace {
    pub(crate) name: String,
    pub(crate) monitor: String,
}

impl Workspace {
    pub(crate) fn is_special(&self) -> bool {
        self.name.starts_with("special")
    }
}
