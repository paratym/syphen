use std::{fs::File, path::Path};
use syphon::{
    dsp::adapters::SampleTypeAdapter,
    io::{
        syphon_codec_registry, syphon_format_registry,
        utils::{pipe_buffered, Track},
        EncodedStreamSpecBuilder, FormatData, FormatIdentifier, SampleReaderRef, SampleWriterRef,
        SyphonCodec, SyphonFormat,
    },
    Sample, SampleStream, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/samples/sine.wav");
    let src_file = Box::new(File::open(src_path)?);
    let format_identifier = src_path
        .extension()
        .map(|ext| ext.to_str())
        .flatten()
        .map(|ext| FormatIdentifier::FileExtension(ext));

    let format_registry = syphon_format_registry();
    let codec_registry = syphon_codec_registry();

    let format_reader = format_registry.resolve_reader(src_file, format_identifier)?;
    let track_reader = Box::new(Track::default(format_reader)?);
    let decoder = match codec_registry.construct_decoder(track_reader)? {
        SampleReaderRef::I16(decoder) => decoder,
        _ => return Err(SyphonError::Unsupported),
    };

    let mut converter = SampleTypeAdapter::<_, _, f32, 1024>::reader(decoder);
    let stream_spec = *converter.spec();
    let bytes_per_frame = stream_spec.sample_format.byte_size() * stream_spec.n_channels as usize;

    let out_data = FormatData {
        tracks: vec![EncodedStreamSpecBuilder {
            codec_key: Some(SyphonCodec::Pcm),
            block_size: Some(bytes_per_frame),
            byte_len: stream_spec.n_frames.map(|n| n * bytes_per_frame as u64),
            decoded_spec: stream_spec.into(),
        }],
    };

    let out_file = Box::new(File::create("./examples/samples/sine_converted.wav")?);
    let format_writer = format_registry.construct_writer(&SyphonFormat::Wav, out_file, out_data)?;
    let track_writer = Box::new(Track::default(format_writer)?);
    let mut encoder = match codec_registry.construct_encoder(track_writer)? {
        SampleWriterRef::F32(encoder) => encoder,
        _ => return Err(SyphonError::StreamMismatch),
    };

    let mut buf = [f32::MID; 1024];
    pipe_buffered(&mut converter, &mut encoder, &mut buf)
}
