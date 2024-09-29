use std::sync::Arc;

pub trait ResultExt<T, E> {
    fn arc(self) -> Result<Arc<T>, E>;
    fn some(self) -> Result<Option<T>, E>;
    fn err_str(self) -> Result<T, String>
    where
        E: ToString;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn arc(self) -> Result<Arc<T>, E> {
        self.map(Arc::new)
    }

    fn some(self) -> Result<Option<T>, E> {
        self.map(Some)
    }

    fn err_str(self) -> Result<T, String>
    where
        E: ToString,
    {
        self.map_err(|e| e.to_string())
    }
}
