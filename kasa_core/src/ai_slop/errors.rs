use thiserror::Error;

#[derive(Error, Debug)]
pub enum SlopTagParseError {
    #[error("This file format is not supported for AI tag parser, that format was: {0}")]
    UnsupportedFile(String),
    #[error(
        "There was multiple null characters in the tEXt field of the PNG, file might be corrupt?"
    )]
    PngTextParseSplitError,
    #[error("parameters png exif doesn't exist")]
    A1111ParametersNotFound,
    #[error("The image doesn't have UserComment exif data, where the AI metadata is stored")]
    ImageDoesntHaveUserCommentExif,
    #[error("The user comment exif field of the image doesn't have the correct value type")]
    InvalidUserComment,
    #[error("Nom parser error: {0}")]
    NomParserError(String),
    #[error("A1111 The positive prompts don't have a valid ending")]
    A1111NoPromptEndFound,
}
