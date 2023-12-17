use std::{fs::File, path::Path};
use syphon::{
    io::{
        formats::FormatIdentifier, utils::copy, Format, FormatData, StreamReader, StreamSpec,
        StreamWriter, SyphonFormat,
    },
    Sample, Signal, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/generated/sine.wav");
    let src_file = Box::new(File::open(src_path)?);
    let format_identifier = src_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FormatIdentifier::FileExtension(ext));

    let mut decoder = SyphonFormat::resolve_reader(src_file, format_identifier)?
        .into_default_track()?
        .into_decoder()?
        .adapt_sample_type::<i16>();

    let track_spec = StreamSpec::builder().with_decoded_spec(*decoder.spec());
    let data = FormatData::builder()
        .with_format(SyphonFormat::Wave)
        .with_track(track_spec)
        .filled()?
        .build()?;

    let dst_file = Box::new(File::create("./examples/generated/sine_converted.wav")?);
    let mut encoder = data
        .writer(dst_file)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_i16_signal()?;

    let mut buf = [i16::ORIGIN; 1024];
    copy(&mut decoder, &mut encoder, &mut buf)
}
