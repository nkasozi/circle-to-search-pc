mod frame_conversion;
#[cfg(test)]
mod tests;

use std::time::Duration;

use anyhow::{Context, Result};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{AllocAnyThread, MainThreadMarker};
use objc2_av_foundation::{AVMediaTypeVideo, AVPlayer, AVPlayerItem, AVPlayerItemVideoOutput};
use objc2_core_media::CMTime;
use objc2_core_video::{
    kCVPixelBufferHeightKey, kCVPixelBufferPixelFormatTypeKey, kCVPixelBufferWidthKey,
    kCVPixelFormatType_32BGRA,
};
use objc2_foundation::{NSDictionary, NSString};

use super::asset_reading::{first_track, open_asset};
use super::foundation_helpers::{
    bridged_key, build_dictionary, file_url, number_object, required_static,
};
use super::system_constant_names::AV_MEDIA_TYPE_VIDEO_NAME;
use crate::core::interfaces::ports::{
    MediaPlayer, PlaybackRequest, PlaybackSession, PlaybackStatus,
};
use crate::core::models::VideoFrame;
use frame_conversion::convert_bgra_buffer;

const LOG_TAG: &str = "[AV_PLAYER]";
const MAIN_THREAD_REQUIRED_ERROR: &str = "the player must be opened on the main thread";
const PLAYER_OPENED_EVENT: &str = "player_opened";
const PLAYING_RATE_THRESHOLD: f32 = 0.0;
const END_TOLERANCE: Duration = Duration::from_millis(40);
const SEEK_TIMESCALE: i32 = 600;
const MINIMUM_EVEN_EDGE: u32 = 2;

pub struct AvFoundationMediaPlayer;

pub struct AvFoundationPlaybackSession {
    player: Retained<AVPlayer>,
    item: Retained<AVPlayerItem>,
    output: Retained<AVPlayerItemVideoOutput>,
    duration: Duration,
}

pub fn cm_time_to_duration(time: CMTime) -> Duration {
    if time.timescale <= 0 || time.value < 0 {
        return Duration::ZERO;
    }
    Duration::from_secs_f64(time.value as f64 / time.timescale as f64)
}

pub fn scaled_output_size(
    natural_width: f64,
    natural_height: f64,
    maximum_width: u32,
) -> (u32, u32) {
    let scale = (maximum_width as f64 / natural_width.max(1.0)).min(1.0);
    let width = (natural_width * scale).round() as u32;
    let height = (natural_height * scale).round() as u32;
    (
        width.max(MINIMUM_EVEN_EDGE) - width.max(MINIMUM_EVEN_EDGE) % 2,
        height.max(MINIMUM_EVEN_EDGE) - height.max(MINIMUM_EVEN_EDGE) % 2,
    )
}

#[allow(deprecated)]
fn video_natural_size(request: &PlaybackRequest) -> Result<(f64, f64, Duration)> {
    let asset = open_asset(&request.file_path);
    let video_type = required_static(unsafe { AVMediaTypeVideo }, AV_MEDIA_TYPE_VIDEO_NAME)?;
    let track = first_track(&asset, video_type)?;
    let natural_size = unsafe { track.naturalSize() };
    Ok((
        natural_size.width,
        natural_size.height,
        cm_time_to_duration(unsafe { asset.duration() }),
    ))
}

fn build_output_attributes(
    output_width: u32,
    output_height: u32,
) -> Retained<NSDictionary<NSString, AnyObject>> {
    build_dictionary(&[
        (
            bridged_key(unsafe { kCVPixelBufferPixelFormatTypeKey }),
            number_object(kCVPixelFormatType_32BGRA as i64),
        ),
        (
            bridged_key(unsafe { kCVPixelBufferWidthKey }),
            number_object(output_width as i64),
        ),
        (
            bridged_key(unsafe { kCVPixelBufferHeightKey }),
            number_object(output_height as i64),
        ),
    ])
}

impl MediaPlayer for AvFoundationMediaPlayer {
    fn open(&self, request: &PlaybackRequest) -> Result<Box<dyn PlaybackSession>> {
        let main_thread = MainThreadMarker::new().context(MAIN_THREAD_REQUIRED_ERROR)?;
        let (natural_width, natural_height, duration) = video_natural_size(request)?;
        let (output_width, output_height) =
            scaled_output_size(natural_width, natural_height, request.maximum_frame_width);
        let attributes = build_output_attributes(output_width, output_height);
        let item =
            unsafe { AVPlayerItem::playerItemWithURL(&file_url(&request.file_path), main_thread) };
        let output = unsafe {
            AVPlayerItemVideoOutput::initWithPixelBufferAttributes(
                AVPlayerItemVideoOutput::alloc(),
                Some(&attributes),
            )
        };
        unsafe { item.addOutput(&output) };
        let player = unsafe { AVPlayer::playerWithPlayerItem(Some(&item), main_thread) };
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": PLAYER_OPENED_EVENT, "width": output_width, "height": output_height, "duration_seconds": duration.as_secs_f64()})
        );
        Ok(Box::new(AvFoundationPlaybackSession {
            player,
            item,
            output,
            duration,
        }))
    }
}

impl AvFoundationPlaybackSession {
    fn position(&self) -> Duration {
        cm_time_to_duration(unsafe { self.player.currentTime() }).min(self.duration)
    }
}

impl PlaybackSession for AvFoundationPlaybackSession {
    fn play(&mut self) -> Result<bool> {
        if self.position() + END_TOLERANCE >= self.duration {
            self.seek(Duration::ZERO)?;
        }
        unsafe { self.player.play() };
        Ok(true)
    }

    fn pause(&mut self) -> Result<bool> {
        unsafe { self.player.pause() };
        Ok(false)
    }

    #[allow(deprecated)]
    fn seek(&mut self, position: Duration) -> Result<Duration> {
        let target = position.min(self.duration);
        let target_time = unsafe {
            CMTime::new(
                (target.as_secs_f64() * SEEK_TIMESCALE as f64) as i64,
                SEEK_TIMESCALE,
            )
        };
        let zero_tolerance = unsafe { CMTime::new(0, SEEK_TIMESCALE) };
        unsafe {
            self.item.seekToTime_toleranceBefore_toleranceAfter(
                target_time,
                zero_tolerance,
                zero_tolerance,
            )
        };
        Ok(target)
    }

    fn status(&self) -> PlaybackStatus {
        let position = self.position();
        let has_ended = position + END_TOLERANCE >= self.duration;
        PlaybackStatus {
            position,
            duration: self.duration,
            is_playing: unsafe { self.player.rate() } > PLAYING_RATE_THRESHOLD && !has_ended,
            has_ended,
        }
    }

    fn take_new_frame(&mut self) -> Option<VideoFrame> {
        let item_time = unsafe { self.item.currentTime() };
        if !unsafe { self.output.hasNewPixelBufferForItemTime(item_time) } {
            return None;
        }
        let pixel_buffer = unsafe {
            self.output
                .copyPixelBufferForItemTime_itemTimeForDisplay(item_time, std::ptr::null_mut())
        }?;
        convert_bgra_buffer(&pixel_buffer)
    }
}
