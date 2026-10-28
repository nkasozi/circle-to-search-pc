use anyhow::Result;
use objc2::rc::Retained;
use objc2::AllocAnyThread;
use objc2_av_foundation::{
    AVAsset, AVAssetReader, AVAssetReaderAudioMixOutput, AVAssetReaderTrackOutput, AVAssetTrack,
    AVMediaType, AVMutableComposition,
};
use objc2_foundation::NSArray;

use super::super::asset_reading::create_reader;
use super::settings::{audio_settings, LINEAR_PCM_FORMAT_IDENTIFIER};

pub struct MuxReaders {
    pub video_reader: Retained<AVAssetReader>,
    pub video_output: Retained<AVAssetReaderTrackOutput>,
    pub audio_reader: Retained<AVAssetReader>,
    pub audio_output: Retained<AVAssetReaderAudioMixOutput>,
}

fn create_video_reader(
    video_asset: &AVAsset,
    video_track: &AVAssetTrack,
) -> Result<(Retained<AVAssetReader>, Retained<AVAssetReaderTrackOutput>)> {
    let video_reader = create_reader(video_asset)?;
    let video_output = unsafe {
        AVAssetReaderTrackOutput::initWithTrack_outputSettings(
            AVAssetReaderTrackOutput::alloc(),
            video_track,
            None,
        )
    };
    unsafe { video_reader.addOutput(&video_output) };
    Ok((video_reader, video_output))
}

#[allow(deprecated)]
fn create_audio_reader(
    composition: &AVMutableComposition,
    audio_type: &AVMediaType,
) -> Result<(
    Retained<AVAssetReader>,
    Retained<AVAssetReaderAudioMixOutput>,
)> {
    let audio_reader = create_reader(composition)?;
    let composition_as_asset: &AVAsset = composition;
    let composition_audio_tracks: Retained<NSArray<AVAssetTrack>> =
        unsafe { composition_as_asset.tracksWithMediaType(audio_type) };
    let pcm_settings = audio_settings(LINEAR_PCM_FORMAT_IDENTIFIER)?;
    let audio_output = unsafe {
        AVAssetReaderAudioMixOutput::initWithAudioTracks_audioSettings(
            AVAssetReaderAudioMixOutput::alloc(),
            &composition_audio_tracks,
            Some(&pcm_settings),
        )
    };
    unsafe { audio_reader.addOutput(&audio_output) };
    Ok((audio_reader, audio_output))
}

pub fn create_mux_readers(
    video_asset: &AVAsset,
    video_track: &AVAssetTrack,
    composition: &AVMutableComposition,
    audio_type: &AVMediaType,
) -> Result<MuxReaders> {
    let (video_reader, video_output) = create_video_reader(video_asset, video_track)?;
    let (audio_reader, audio_output) = create_audio_reader(composition, audio_type)?;
    Ok(MuxReaders {
        video_reader,
        video_output,
        audio_reader,
        audio_output,
    })
}
