extern crate serde_json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KnockKnockError {
    #[error("could not find recipe file: {0}")]
    RecipesNotFound(#[from] std::io::Error),
    #[error("could not read recipe file: {0}")]
    RecipeMisformat(#[from] serde_json::Error),
}
