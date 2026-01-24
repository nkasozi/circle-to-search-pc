use anyhow::{Context, Result};
use async_trait::async_trait;
use image::DynamicImage;
use std::path::PathBuf;
use tesseract_static::parse::ParsedHocr;
use tesseract_static::tesseract::Tesseract;

use crate::core::interfaces::adapters::OcrService;
use crate::core::models::{DetectedText, DetectedWord, OcrResult};

const TRAINING_DATA: &[u8] = include_bytes!("../../tessdata/eng.traineddata");

fn get_ocr_replacements() -> Vec<(&'static str, &'static str)> {
    vec![
        ("¬Æ", ""),
        ("¬©", ""),
        ("(|)", ""),
        ("¬¢", "•"),
        ("¬∑", "•"),
        ("¬∏", "•"),
        ("-¬†", "—"),
        ("¬†", " "),
        ("¬´", "'"),
        ("¬ª", "'"),
        ("¬∫", "'"),
        ("¬º", "'"),
        ("¬´¬´", "\""),
        ("¬ª¬ª", "\""),
        ("¬´¬ª", "\""),
        ("``", "\""),
        ("''", "\""),
        ("~", "−"),
        ("(t/\\)", "(↑/↓)"),
        ("t/\\", "↑/↓"),
        ("<->", "↔"),
        ("->", "→"),
        ("<-", "←"),
        ("<=>", "⇔"),
        ("=>", "⇒"),
        ("<=", "⇐"),
        (">>", "»"),
        ("<<", "«"),
        ("...", "…"),
        ("(c)", "©"),
        ("(C)", "©"),
        ("(r)", "®"),
        ("(R)", "®"),
        ("(tm)", "™"),
        ("(TM)", "™"),
        ("+-", "±"),
        ("+/-", "±"),
        ("!=", "≠"),
        ("/=", "≠"),
        ("<=", "≤"),
        (">=", "≥"),
        ("~=", "≈"),
        ("~~", "≈"),
        ("inf", "∞"),
        ("deg", "°"),
        ("1/2", "½"),
        ("1/4", "¼"),
        ("3/4", "¾"),
        ("1/3", "⅓"),
        ("2/3", "⅔"),
        ("sqrt", "√"),
        ("cbrt", "∛"),
        ("sum", "∑"),
        ("prod", "∏"),
        ("delta", "Δ"),
        ("alpha", "α"),
        ("beta", "β"),
        ("gamma", "γ"),
        ("theta", "θ"),
        ("lambda", "λ"),
        ("mu", "μ"),
        ("pi", "π"),
        ("sigma", "σ"),
        ("omega", "ω"),
        ("x^2", "x²"),
        ("^2", "²"),
        ("^3", "³"),
        ("^n", "ⁿ"),
        ("EUR", "€"),
        ("GBP", "£"),
        ("JPY", "¥"),
        ("CNY", "¥"),
        ("[x]", "☑"),
        ("[ ]", "☐"),
        ("[X]", "☑"),
        ("[v]", "✓"),
        ("[V]", "✓"),
        ("(*)", "★"),
        ("[*]", "★"),
        ("<3", "♥"),
        (":)", "☺"),
        (":(", "☹"),
        ("|>", "▶"),
        ("<|", "◀"),
        ("||", "⏸"),
        ("[>", "▶"),
        ("#S", "⌘S"),
        ("#C", "⌘C"),
        ("#D", "⌘D"),
        ("#V", "⌘V"),
        ("#Z", "⌘Z"),
        ("#X", "⌘X"),
        ("#A", "⌘A"),
        ("#F", "⌘F"),
        ("#W", "⌘W"),
        ("#Q", "⌘Q"),
        ("#N", "⌘N"),
        ("#O", "⌘O"),
        ("#P", "⌘P"),
        ("#T", "⌘T"),
        ("#R", "⌘R"),
        ("#B", "⌘B"),
        ("#I", "⌘I"),
        ("#U", "⌘U"),
        ("#K", "⌘K"),
        ("#L", "⌘L"),
        ("#G", "⌘G"),
        ("#H", "⌘H"),
        ("#M", "⌘M"),
        ("#E", "⌘E"),
        ("#J", "⌘J"),
        ("Ctrl+", "⌃"),
        ("Alt+", "⌥"),
        ("Shift+", "⇧"),
        ("Cmd+", "⌘"),
        ("Tab", "⇥"),
        ("Enter", "↵"),
        ("Return", "↵"),
        ("Esc", "⎋"),
        ("Del", "⌫"),
        ("Backspace", "⌫"),
        ("Caps", "⇪"),
    ]
}

fn cleanup_ocr_artifacts(text: &str) -> String {
    let mut result = text.to_string();
    for (pattern, replacement) in get_ocr_replacements() {
        result = result.replace(pattern, replacement);
    }
    result.trim().to_string()
}

fn parse_bounds_from_debug(debug_str: &str) -> (f32, f32, f32, f32) {
    let numbers: Vec<f32> = debug_str
        .split(|c: char| !c.is_numeric() && c != '.' && c != '-')
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();

    if numbers.len() >= 4 {
        let min_x = numbers[0];
        let min_y = numbers[1];
        let max_x = numbers[2];
        let max_y = numbers[3];
        (min_x, min_y, max_x - min_x, max_y - min_y)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

pub struct TesseractOcrService {
    tessdata_dir: PathBuf,
}

impl TesseractOcrService {
    pub fn build() -> Result<Self> {
        log::info!("[TESSERACT_OCR] Initializing Tesseract OCR service");

        let tessdata_dir = std::env::temp_dir().join("circle-to-search-tessdata");
        std::fs::create_dir_all(&tessdata_dir)
            .context("Failed to create tessdata directory in temp folder")?;

        let eng_traineddata_path = tessdata_dir.join("eng.traineddata");
        if !eng_traineddata_path.exists() {
            log::info!(
                "[TESSERACT_OCR] Extracting training data to {:?}",
                eng_traineddata_path
            );
            std::fs::write(&eng_traineddata_path, TRAINING_DATA)
                .context("Failed to write eng.traineddata to temp directory")?;
        }

        log::info!("[TESSERACT_OCR] Using tessdata from: {:?}", tessdata_dir);

        Ok(Self { tessdata_dir })
    }
}

#[async_trait]
impl OcrService for TesseractOcrService {
    async fn extract_text_from_image(&self, image: &DynamicImage) -> Result<OcrResult> {
        log::info!("[TESSERACT_OCR] Starting text extraction");
        log::debug!(
            "[TESSERACT_OCR] Image dimensions: {}x{}",
            image.width(),
            image.height()
        );

        let rgb_image = image.to_rgb8();
        let width = rgb_image.width() as i32;
        let height = rgb_image.height() as i32;
        let bytes_per_pixel = 3;
        let bytes_per_line = width * bytes_per_pixel;
        let frame_data = rgb_image.as_raw();

        log::debug!(
            "[TESSERACT_OCR] Using raw frame data: width={}, height={}, bpp={}, bpl={}",
            width,
            height,
            bytes_per_pixel,
            bytes_per_line
        );

        let tesseract = Tesseract::new(Some(&self.tessdata_dir.display().to_string()), Some("eng"))
            .map_err(|e| {
                log::error!(
                    "[TESSERACT_OCR] Failed to initialize Tesseract with tessdata: {:?}, error: {:?}",
                    self.tessdata_dir,
                    e
                );
                anyhow::anyhow!("Failed to initialize Tesseract instance: {:?}", e)
            })?;

        let mut tesseract = tesseract
            .set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line)
            .map_err(|e| {
                log::error!(
                    "[TESSERACT_OCR] Failed to set image from frame data, error: {:?}",
                    e
                );
                anyhow::anyhow!("Failed to set image in Tesseract: {:?}", e)
            })?;

        log::debug!("[TESSERACT_OCR] Getting hOCR output for word bounding boxes");

        let hocr_xml = tesseract.get_hocr_text(1).map_err(|e| {
            log::error!("[TESSERACT_OCR] Failed to get hOCR text, error: {:?}", e);
            anyhow::anyhow!("Failed to extract hOCR data from image: {:?}", e)
        })?;

        log::debug!("[TESSERACT_OCR] Parsing hOCR output");

        let hocr = ParsedHocr::new(&hocr_xml)
            .map_err(|e| anyhow::anyhow!("Failed to parse hOCR XML: {:?}", e))?;

        let mut detected_texts = Vec::new();
        let mut full_text = String::new();

        for carea in &hocr.careas {
            for paragraph in &carea.paragraphs {
                for line in &paragraph.lines {
                    for word in &line.words {
                        let raw_word_text = word.text.trim();

                        if raw_word_text.is_empty() {
                            continue;
                        }

                        let conf = word.confidence;
                        if conf < 0.30 {
                            log::debug!(
                                "[TESSERACT_OCR] Skipping low confidence word '{}' (conf: {:.2})",
                                raw_word_text,
                                conf * 100.0
                            );
                            continue;
                        }

                        let word_text = cleanup_ocr_artifacts(raw_word_text);
                        if word_text.is_empty() {
                            continue;
                        }

                        let word_bounds_str = format!("{:?}", word.bounds);
                        let (x, y, width, height) = parse_bounds_from_debug(&word_bounds_str);

                        log::debug!(
                            "[TESSERACT_OCR] Word: '{}' at ({},{}) {}x{} conf: {:.2}",
                            word_text,
                            x,
                            y,
                            width,
                            height,
                            conf * 100.0
                        );

                        full_text.push_str(&word_text);
                        full_text.push(' ');

                        detected_texts.push(DetectedText::new(
                            word_text.to_string(),
                            x,
                            y,
                            width,
                            height,
                            conf,
                            vec![DetectedWord::new(
                                word_text.to_string(),
                                x,
                                y,
                                width,
                                height,
                            )],
                        ));
                    }
                }
            }
        }

        log::info!(
            "[TESSERACT_OCR] Text extraction complete. Found {} words",
            detected_texts.len()
        );
        log::debug!("[TESSERACT_OCR] Full text: {}", full_text.trim());

        Ok(OcrResult {
            text_blocks: detected_texts,
            full_text: full_text.trim().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_creates_service_successfully() {
        let result = TesseractOcrService::build();

        assert!(result.is_ok());
    }

    #[test]
    fn test_cleanup_ocr_artifacts_removes_garbage_characters() {
        assert_eq!(cleanup_ocr_artifacts("¬Æ test"), "test");
        assert_eq!(cleanup_ocr_artifacts("¬© text"), "text");
        assert_eq!(cleanup_ocr_artifacts("hello (|) world"), "hello  world");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_bullets() {
        assert_eq!(cleanup_ocr_artifacts("¬¢ item"), "• item");
        assert_eq!(cleanup_ocr_artifacts("¬∑ bullet"), "• bullet");
        assert_eq!(cleanup_ocr_artifacts("¬∏ point"), "• point");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_quotes() {
        assert_eq!(cleanup_ocr_artifacts("¬´hello¬ª"), "'hello'");
        assert_eq!(cleanup_ocr_artifacts("``quoted''"), "\"quoted\"");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_arrows() {
        assert_eq!(cleanup_ocr_artifacts("go ->"), "go →");
        assert_eq!(cleanup_ocr_artifacts("<- back"), "← back");
        assert_eq!(cleanup_ocr_artifacts("<->"), "↔");
        assert_eq!(cleanup_ocr_artifacts("=>"), "⇒");
        assert_eq!(cleanup_ocr_artifacts("t/\\"), "↑/↓");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_mac_shortcuts() {
        assert_eq!(cleanup_ocr_artifacts("#C"), "⌘C");
        assert_eq!(cleanup_ocr_artifacts("#V"), "⌘V");
        assert_eq!(cleanup_ocr_artifacts("#S"), "⌘S");
        assert_eq!(cleanup_ocr_artifacts("#Z"), "⌘Z");
        assert_eq!(
            cleanup_ocr_artifacts("Press #C to copy"),
            "Press ⌘C to copy"
        );
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_modifier_keys() {
        assert_eq!(cleanup_ocr_artifacts("Ctrl+C"), "⌃C");
        assert_eq!(cleanup_ocr_artifacts("Alt+Tab"), "⌥⇥");
        assert_eq!(cleanup_ocr_artifacts("Shift+Enter"), "⇧↵");
        assert_eq!(cleanup_ocr_artifacts("Cmd+Q"), "⌘Q");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_special_keys() {
        assert_eq!(cleanup_ocr_artifacts("Tab"), "⇥");
        assert_eq!(cleanup_ocr_artifacts("Enter"), "↵");
        assert_eq!(cleanup_ocr_artifacts("Esc"), "⎋");
        assert_eq!(cleanup_ocr_artifacts("Del"), "⌫");
        assert_eq!(cleanup_ocr_artifacts("Backspace"), "⌫");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_math_symbols() {
        assert_eq!(cleanup_ocr_artifacts("+-"), "±");
        assert_eq!(cleanup_ocr_artifacts("!="), "≠");
        assert_eq!(cleanup_ocr_artifacts("..."), "…");
        assert_eq!(cleanup_ocr_artifacts("1/2"), "½");
        assert_eq!(cleanup_ocr_artifacts("pi"), "π");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_copyright_symbols() {
        assert_eq!(cleanup_ocr_artifacts("(c)"), "©");
        assert_eq!(cleanup_ocr_artifacts("(C)"), "©");
        assert_eq!(cleanup_ocr_artifacts("(r)"), "®");
        assert_eq!(cleanup_ocr_artifacts("(tm)"), "™");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_checkboxes() {
        assert_eq!(cleanup_ocr_artifacts("[x]"), "☑");
        assert_eq!(cleanup_ocr_artifacts("[ ]"), "☐");
        assert_eq!(cleanup_ocr_artifacts("[v]"), "✓");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_converts_currency() {
        assert_eq!(cleanup_ocr_artifacts("EUR 100"), "€ 100");
        assert_eq!(cleanup_ocr_artifacts("GBP 50"), "£ 50");
        assert_eq!(cleanup_ocr_artifacts("JPY 1000"), "¥ 1000");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_trims_whitespace() {
        assert_eq!(cleanup_ocr_artifacts("  hello  "), "hello");
        assert_eq!(cleanup_ocr_artifacts("\ttest\n"), "test");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_handles_empty_string() {
        assert_eq!(cleanup_ocr_artifacts(""), "");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_preserves_normal_text() {
        assert_eq!(cleanup_ocr_artifacts("Hello World"), "Hello World");
        assert_eq!(cleanup_ocr_artifacts("Test 123"), "Test 123");
    }

    #[test]
    fn test_cleanup_ocr_artifacts_handles_complex_text() {
        let input = "Press #C to copy ¬¢ item -> next";
        let expected = "Press ⌘C to copy • item → next";
        assert_eq!(cleanup_ocr_artifacts(input), expected);
    }

    #[test]
    fn test_parse_bounds_from_debug_extracts_coordinates() {
        let result = parse_bounds_from_debug("Rect { min: (10, 20), max: (100, 50) }");
        assert_eq!(result, (10.0, 20.0, 90.0, 30.0));
    }

    #[test]
    fn test_parse_bounds_from_debug_handles_invalid_input() {
        let result = parse_bounds_from_debug("invalid");
        assert_eq!(result, (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_get_ocr_replacements_returns_non_empty_list() {
        let replacements = get_ocr_replacements();
        assert!(!replacements.is_empty());
        assert!(replacements.len() > 100);
    }

    #[test]
    fn test_get_ocr_replacements_contains_essential_patterns() {
        let replacements = get_ocr_replacements();
        let patterns: Vec<&str> = replacements.iter().map(|(p, _)| *p).collect();

        assert!(patterns.contains(&"#C"));
        assert!(patterns.contains(&"#V"));
        assert!(patterns.contains(&"->"));
        assert!(patterns.contains(&"..."));
        assert!(patterns.contains(&"(c)"));
    }
}
