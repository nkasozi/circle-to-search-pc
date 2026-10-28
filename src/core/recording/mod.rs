pub mod file_naming;
mod frame_ticker;
pub mod output_geometry;
pub mod preview_slot;
pub mod recording_clock;
pub mod session;
mod session_state;
pub mod wav_reader;
pub mod wav_writer;

#[cfg(all(test, target_os = "macos"))]
mod session_tests;
#[cfg(test)]
mod wav_reader_tests;
