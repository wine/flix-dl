use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("missing class: {0}")]
    MissingClass(&'a str),

    #[error("missing attribute: {0}")]
    MissingAttr(&'a str),

    #[error("invalid download")]
    InvalidDownload,
}
