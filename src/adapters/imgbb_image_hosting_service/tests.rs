use super::*;

fn build_test_config(auth_mode: ImageHostingAuthMode) -> ImgbbImageHostingConfig {
    ImgbbImageHostingConfig {
        provider_url: "https://api.imgbb.com/1/upload".to_string(),
        auth_mode,
        public_key_name: "key".to_string(),
        public_key_value: "test-key".to_string(),
        expiration_seconds: "900".to_string(),
        http_method: ImageUploadHttpMethod::Post,
        image_field_name: "image".to_string(),
    }
}

fn create_test_buffer() -> CaptureBuffer {
    let raw_data = vec![255u8; 10 * 10 * 4];
    CaptureBuffer::build_from_raw_data(1.0, 10, 10, raw_data)
}

#[test]
fn test_new_creates_service() {
    let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));

    assert_eq!(service.config.public_key_name, "key");
}

#[tokio::test]
async fn test_save_buffer_to_temp_file_creates_file_at_temp_location() {
    let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));
    let buffer = create_test_buffer();

    let result = service.save_buffer_to_temp_file(&buffer).await;

    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path
        .to_string_lossy()
        .contains("circle_to_search_image.png"));

    if path.exists() {
        std::fs::remove_file(path).ok();
    }
}

#[tokio::test]
async fn test_save_buffer_to_temp_file_returns_error_for_invalid_buffer() {
    let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));
    let invalid_raw_data = vec![255u8; 50];
    let invalid_buffer = CaptureBuffer::build_from_raw_data(1.0, 10, 10, invalid_raw_data);

    let result = service.save_buffer_to_temp_file(&invalid_buffer).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_save_buffer_to_temp_file_can_be_called_multiple_times() {
    let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));
    let buffer1 = create_test_buffer();
    let buffer2 = create_test_buffer();

    let result1 = service.save_buffer_to_temp_file(&buffer1).await;
    assert!(result1.is_ok());

    let result2 = service.save_buffer_to_temp_file(&buffer2).await;
    assert!(result2.is_ok());

    assert_eq!(result1.unwrap(), result2.unwrap());
}

#[test]
fn test_build_upload_url_adds_query_key_when_query_mode() {
    let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));
    let upload_url = service.build_upload_url().unwrap();
    assert!(upload_url.as_str().contains("key=test-key"));
}

#[test]
fn test_build_upload_request_adds_header_key_when_header_mode() {
    let mut config = build_test_config(ImageHostingAuthMode::Header);
    config.public_key_name = "X-API-Key".to_string();
    let service = ImgbbImageHostingService::new(config);
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new().text("image", "test-image");
    let request = service
        .build_upload_request(
            &client,
            Url::parse("https://api.imgbb.com/1/upload").unwrap(),
            form,
        )
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(request.headers().get("X-API-Key").unwrap(), "test-key");
}
