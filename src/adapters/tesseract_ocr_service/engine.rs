use anyhow::Result;
use image::DynamicImage;
use std::path::Path;
use tesseract_static::parse::ParsedHocr;
use tesseract_static::tesseract::Tesseract;

const TRAINING_LANGUAGE: &str = "eng";
const BYTES_PER_RGB_PIXEL: i32 = 3;
const FIRST_PAGE_NUMBER: i32 = 1;
const LOG_TAG: &str = "[TESSERACT_OCR]";
const RAW_FRAME_DATA_PREPARED_EVENT: &str = "raw_frame_data_prepared";
const FRAME_DATA_SET_FAILED_EVENT: &str = "frame_data_set_failed";
const TESSERACT_INITIALIZATION_FAILED_EVENT: &str = "tesseract_initialization_failed";
const HOCR_EXTRACTION_STARTED_EVENT: &str = "hocr_extraction_started";
const HOCR_EXTRACTION_FAILED_EVENT: &str = "hocr_extraction_failed";
const HOCR_PARSING_STARTED_EVENT: &str = "hocr_parsing_started";
const HOCR_PARSING_FAILED_EVENT: &str = "hocr_parsing_failed";

pub(super) fn recognize_hocr_from_image(
    tessdata_directory: &Path,
    image: &DynamicImage,
) -> Result<ParsedHocr> {
    let rgb_image = image.to_rgb8();
    let width = rgb_image.width() as i32;
    let height = rgb_image.height() as i32;
    let bytes_per_line = width * BYTES_PER_RGB_PIXEL;

    log::debug!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": RAW_FRAME_DATA_PREPARED_EVENT, "width": width, "height": height, "bytes_per_pixel": BYTES_PER_RGB_PIXEL, "bytes_per_line": bytes_per_line})
    );

    let tesseract = initialize_tesseract(tessdata_directory)?;
    let mut tesseract = tesseract
        .set_frame(
            rgb_image.as_raw(),
            width,
            height,
            BYTES_PER_RGB_PIXEL,
            bytes_per_line,
        )
        .map_err(|error| {
            log::error!("{} {}", LOG_TAG, serde_json::json!({"event": FRAME_DATA_SET_FAILED_EVENT, "error": format!("{:?}", error)}));
            anyhow::anyhow!("Failed to set image in Tesseract: {:?}", error)
        })?;

    let hocr_xml = extract_hocr_xml(&mut tesseract)?;
    parse_hocr_xml(&hocr_xml)
}

fn initialize_tesseract(tessdata_directory: &Path) -> Result<Tesseract> {
    let tessdata_directory_text = tessdata_directory.display().to_string();
    Tesseract::new(Some(&tessdata_directory_text), Some(TRAINING_LANGUAGE)).map_err(|error| {
        log::error!("{} {}", LOG_TAG, serde_json::json!({"event": TESSERACT_INITIALIZATION_FAILED_EVENT, "tessdata_directory": tessdata_directory_text, "error": format!("{:?}", error)}));
        anyhow::anyhow!("Failed to initialize Tesseract instance: {:?}", error)
    })
}

fn extract_hocr_xml(tesseract: &mut Tesseract) -> Result<String> {
    log::debug!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": HOCR_EXTRACTION_STARTED_EVENT})
    );

    tesseract.get_hocr_text(FIRST_PAGE_NUMBER).map_err(|error| {
        log::error!("{} {}", LOG_TAG, serde_json::json!({"event": HOCR_EXTRACTION_FAILED_EVENT, "error": format!("{:?}", error)}));
        anyhow::anyhow!("Failed to extract hOCR data from image: {:?}", error)
    })
}

fn parse_hocr_xml(hocr_xml: &str) -> Result<ParsedHocr> {
    log::debug!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": HOCR_PARSING_STARTED_EVENT})
    );

    ParsedHocr::new(hocr_xml).map_err(|error| {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": HOCR_PARSING_FAILED_EVENT, "error": format!("{:?}", error)})
        );
        anyhow::anyhow!("Failed to parse hOCR XML: {:?}", error)
    })
}
