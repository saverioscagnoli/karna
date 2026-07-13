#[derive(Debug)]
pub enum ImageError {
    NotAPng,
    PngHeaderInvalid,
    PngTruncated,
    PngUnknownCriticalChunk([u8; 4]),
    PngNoImageData,
    PngMissingPalette,
    PngBadFilter(u8),
    PngSizeMismatch { expected: usize, got: usize },
    PngUnsupported(String),
    Deflate(&'static str),
    ChecksumMismatch,
    Inflate(std::io::Error),
}

impl From<std::io::Error> for ImageError {
    fn from(e: std::io::Error) -> Self {
        ImageError::Inflate(e)
    }
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ImageError {}
