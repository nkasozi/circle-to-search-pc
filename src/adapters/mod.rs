pub mod auto_launch;
#[allow(dead_code)]
mod ffmpeg_video_encoder;
mod google_lens_search_provider;
mod imgbb_image_hosting_service;
pub mod macos_app_behavior;
pub mod macos_permissions;
#[allow(dead_code)]
mod rgba_frame_compositor;
mod tesseract_ocr_service;

#[allow(unused_imports)]
pub use ffmpeg_video_encoder::FfmpegVideoEncoder;
pub use google_lens_search_provider::GoogleLensSearchProvider;
pub use imgbb_image_hosting_service::ImgbbImageHostingService;
#[allow(unused_imports)]
pub use rgba_frame_compositor::RgbaFrameCompositor;
pub use tesseract_ocr_service::TesseractOcrService;
