use crate::error::ExtractorError;
use crate::traits::Extractor;
use async_trait::async_trait;
use hwp::HWP;
use hwp::hwp::paragraph::{
    Paragraph,
    control::{
        Control,
        common_properties::Caption,
        draw_text::DrawText,
        paragraph_list::ParagraphList,
        shape_object::{container::ContainerContent, content::ShapeObjectContent},
    },
};
use std::path::Path;
use tokio::fs;

/// An extractor for Hwp files.
///
/// Hwp is a document format used primarily in South Korea.
/// This extractor parses the binary format and extracts text content.
#[derive(Debug, Default)]
pub struct HwpExtractor;

impl HwpExtractor {
    /// Creates a new HWP extractor
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Extractor for HwpExtractor {
    /// Extracts text from a Hwp file.
    async fn extract(&self, path: &Path) -> Result<String, ExtractorError> {
        // Read the HWP file into bytes
        let bytes = fs::read(path).await.map_err(|e| ExtractorError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        // Parse the HWP file from bytes
        let hwp = HWP::from_bytes(&bytes);

        // Determine the body text based on the header flags
        let body = if hwp.header.flags.distributed {
            hwp.view_texts.as_ref().unwrap()
        } else {
            &hwp.body_texts
        };

        // Extract text from paragraphs and controls
        let mut text_parts = Vec::new();

        // Iterate through sections and paragraphs
        for section in &body.sections {
            for paragraph in &section.paragraphs {
                self.extract_paragraph_content(paragraph, &mut text_parts);
            }
        }

        Ok(text_parts.join("\n"))
    }
}

impl HwpExtractor {
    /// Extracts content from a paragraph including nested controls
    fn extract_paragraph_content(&self, paragraph: &Paragraph, result: &mut Vec<String>) {
        add_text_if_not_empty(result, extract_paragraph_text(paragraph));
        result.extend(find_paragraph(paragraph, true));
    }
}

/// Adds text to the result vector if it's not empty
fn add_text_if_not_empty(result: &mut Vec<String>, text: String) {
    if !text.is_empty() {
        result.push(text);
    }
}

/// Extracts text content from a paragraph's character list
fn extract_paragraph_text(paragraph: &Paragraph) -> String {
    paragraph.to_string()
}

/// Finds and extracts text from paragraph controls
fn find_paragraph(paragraph: &Paragraph, recursive: bool) -> Vec<String> {
    let mut result = Vec::new();

    for control in &paragraph.controls {
        match control {
            Control::Table(control) => {
                for cell in &control.cells {
                    add_text_if_not_empty(
                        &mut result,
                        concat_paragraph_in_list(&cell.paragraph_list, recursive),
                    );
                }
                extract_caption_text(&mut result, &control.common_properties.caption, recursive);
            }
            Control::GenShapeObject(control) => {
                extract_caption_text(&mut result, &control.common_properties.caption, recursive);

                if let ShapeObjectContent::Container(content) = &control.content {
                    result.extend(search_paragraph_in_shape_object(content, recursive));
                }
            }
            Control::ShapeLine(control) => {
                extract_draw_text_and_caption(
                    &mut result,
                    control.draw_text.as_ref(),
                    &control.common_properties.caption,
                    recursive,
                );
            }
            Control::ShapeRectangle(control) => {
                extract_draw_text_and_caption(
                    &mut result,
                    control.draw_text.as_ref(),
                    &control.common_properties.caption,
                    recursive,
                );
            }
            Control::ShapeEllipse(control) => {
                extract_draw_text_and_caption(
                    &mut result,
                    control.draw_text.as_ref(),
                    &control.common_properties.caption,
                    recursive,
                );
            }
            Control::ShapeArc(control) => {
                extract_draw_text_and_caption(
                    &mut result,
                    control.draw_text.as_ref(),
                    &control.common_properties.caption,
                    recursive,
                );
            }
            Control::ShapePolygon(control) => {
                extract_draw_text_and_caption(
                    &mut result,
                    control.draw_text.as_ref(),
                    &control.common_properties.caption,
                    recursive,
                );
            }
            Control::ShapeCurve(control) => {
                extract_draw_text_and_caption(
                    &mut result,
                    control.draw_text.as_ref(),
                    &control.common_properties.caption,
                    recursive,
                );
            }
            Control::Equation(control) => {
                extract_caption_text(&mut result, &control.common_properties.caption, recursive);
            }
            Control::Picture(control) => {
                extract_caption_text(&mut result, &control.common_properties.caption, recursive);
            }
            Control::Ole(control) => {
                extract_caption_text(&mut result, &control.common_properties.caption, recursive);
            }
            Control::Container(control) => {
                extract_caption_text(&mut result, &control.common_properties.caption, recursive);
                result.extend(search_paragraph_in_shape_object(
                    &control.content,
                    recursive,
                ));
            }
            Control::Header(control) | Control::Footer(control) => {
                add_text_if_not_empty(
                    &mut result,
                    concat_paragraph_in_list(&control.paragraph_list, recursive),
                );
            }
            Control::Footnote(control) | Control::Endnote(control) => {
                add_text_if_not_empty(
                    &mut result,
                    concat_paragraph_in_list(&control.paragraph_list, recursive),
                );
            }
            _ => {}
        }
    }

    result
}

/// Extracts text from a caption if it exists
fn extract_caption_text(result: &mut Vec<String>, caption: &Option<Caption>, recursive: bool) {
    if let Some(caption) = caption {
        add_text_if_not_empty(
            result,
            concat_paragraph_in_list(&caption.paragraph_list, recursive),
        );
    }
}

/// Extracts text from draw_text if it exists
fn extract_draw_text(result: &mut Vec<String>, draw_text: Option<&DrawText>, recursive: bool) {
    if let Some(draw_text) = draw_text {
        add_text_if_not_empty(
            result,
            concat_paragraph_in_list(&draw_text.paragraph_list, recursive),
        );
    }
}

/// Extracts text from draw_text and caption
fn extract_draw_text_and_caption(
    result: &mut Vec<String>,
    draw_text: Option<&DrawText>,
    caption: &Option<Caption>,
    recursive: bool,
) {
    extract_draw_text(result, draw_text, recursive);
    extract_caption_text(result, caption, recursive);
}

/// Concatenates paragraphs in a list recursively
fn concat_paragraph_in_list(list: &ParagraphList, recursive: bool) -> String {
    let mut result = Vec::new();

    for paragraph in &list.paragraphs {
        add_text_if_not_empty(&mut result, extract_paragraph_text(paragraph));

        if recursive {
            result.extend(find_paragraph(paragraph, recursive));
        }
    }

    result.join("\n")
}

/// Searches for paragraph text in shape object containers
fn search_paragraph_in_shape_object(content: &ContainerContent, recursive: bool) -> Vec<String> {
    let mut result = Vec::new();

    for child in &content.children {
        if let Some(draw_text) = &child.draw_text {
            let text = concat_paragraph_in_list(&draw_text.paragraph_list, recursive);
            if !text.is_empty() {
                result.push(text);
            }
        }

        if let ShapeObjectContent::Container(container) = &child.content {
            let nested_text = search_paragraph_in_shape_object(container, recursive);
            result.extend(nested_text);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn instantiate_hwp() {
        let extractor = HwpExtractor;
        let path = PathBuf::from("tests/test_files/sample.hwp");

        // Ensure the extractor can be instantiated and used
        let result = extractor.extract(&path).await;

        assert!(result.is_ok());
        let text = result.unwrap();
        println!("Extracted text: {}", text);
        assert!(!text.is_empty(), "Extracted text should not be empty");
    }
}
