use std::fmt::Display;

pub enum Error {
    K8sError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::K8sError(message) => write!(f, "K8sError: {}", message),
        }
    }
}
