use crate::{
    io::{EncodedStreamReader, SampleReader, SampleReaderRef, StreamSpec},
    Sample, SampleFormat, SyphonError,
};
use byte_slice_cast::{AsMutByteSlice, ToMutByteSlice};
use std::{io::Read, mem::size_of};

pub struct PcmDecoder {
    reader: Box<dyn EncodedStreamReader>,
}

impl PcmDecoder {
    pub fn new(reader: Box<dyn EncodedStreamReader>) -> Result<Self, SyphonError> {
        let spec = reader.stream_spec();
        let bytes_per_sample_block =
            spec.decoded_spec.block_size * spec.decoded_spec.sample_format.size();

        if bytes_per_sample_block % spec.block_size != 0 {
            return Err(SyphonError::BadRequest);
        }

        Ok(Self { reader })
    }

    pub fn into_sample_reader_ref(self) -> SampleReaderRef {
        let decoder_ref = Box::new(self);
        match decoder_ref.reader.stream_spec().decoded_spec.sample_format {
            SampleFormat::U8 => SampleReaderRef::U8(decoder_ref),
            SampleFormat::U16 => SampleReaderRef::U16(decoder_ref),
            SampleFormat::U32 => SampleReaderRef::U32(decoder_ref),
            SampleFormat::U64 => SampleReaderRef::U64(decoder_ref),

            SampleFormat::I8 => SampleReaderRef::I8(decoder_ref),
            SampleFormat::I16 => SampleReaderRef::I16(decoder_ref),
            SampleFormat::I32 => SampleReaderRef::I32(decoder_ref),
            SampleFormat::I64 => SampleReaderRef::I64(decoder_ref),

            SampleFormat::F32 => SampleReaderRef::F32(decoder_ref),
            SampleFormat::F64 => SampleReaderRef::F64(decoder_ref),
        }
    }
}

impl<S: Sample + ToMutByteSlice> SampleReader<S> for PcmDecoder {
    fn stream_spec(&self) -> &StreamSpec {
        &self.reader.stream_spec().decoded_spec
    }

    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let spec = SampleReader::<S>::stream_spec(self);
        let n_samples = buffer.len() - (buffer.len() % spec.block_size);
        let sample_block_size = spec.block_size * size_of::<S>();

        match self.reader.read(buffer[..n_samples].as_mut_byte_slice()) {
            Ok(n) if n % sample_block_size == 0 => Ok(n / size_of::<S>()),
            Ok(_) => Err(SyphonError::StreamMismatch),
            Err(e) => Err(e.into()),
        }
    }
}

// pub struct PcmEncoder {
//     writer: Box<dyn Write>,
//     stream_spec: StreamSpec,
// }

// impl PcmEncoder {
//     pub fn new(writer: Box<dyn Write>, stream_spec: StreamSpec) -> Self {
//         Self {
//             writer,
//             stream_spec,
//         }
//     }
// }

// impl<S: Sample + ToByteSlice> SampleWriter<S> for PcmEncoder {
//     fn stream_spec(&self) -> &StreamSpec {
//         &self.stream_spec
//     }

//     fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
//         let spec = self.;

//         match self.writer.write(buffer[..n_samples].as_byte_slice()) {
//             Ok(n) if n % bytes_per_block == 0 => Ok(n / <S>::N_BYTES),
//             Ok(_) => Err(SyphonError::StreamMismatch),
//             Err(e) => Err(e.into()),
//         }
//     }
// }
