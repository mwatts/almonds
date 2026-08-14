use markdown2pdf::{config::ConfigSource, parse_into_file};

use crate::error::LunarError;

// Default styling — saves the PDF to the user's Downloads directory
pub fn parse_markdown_to_pdf(markdown: &str, file_name: &str) -> Result<(), LunarError> {
    let _output_path = dirs::download_dir()
        .ok_or(LunarError::DownloadDirNotFound)?
        .join(file_name)
        .join(".pdf");

    parse_into_file(
        markdown.to_string(),
        "output.pdf",
        // &output_path.to_string_lossy(),
        ConfigSource::Default,
        None,
    )
    .map_err(|e| LunarError::Markdown2Pdf(e.to_string()))
}
