#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Generic error")]
    Generic(#[from] Box<dyn std::error::Error>),
    #[error("Serialization")]
    Serde(#[from] serde_json::error::Error),
    #[error("Unknown error")]
    Unknown
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}