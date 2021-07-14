use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Author {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) guest: bool,
}
