use crate::{
    BlockingFormat, BlockingStream, FiniteFormat, FiniteStream, Format, FormatReader, FormatSeeker,
    FormatTag, FormatWriter, IndexedFormat, IndexedStream, Stream, StreamReader, StreamSeeker,
    StreamSpec, StreamWriter,
};
use phonic_signal::PhonicResult;
use std::mem::MaybeUninit;

pub struct StreamSelector<F: Format> {
    inner: F,
    spec: StreamSpec<<F::Tag as FormatTag>::Codec>,
    stream: usize,
}

impl<F: Format> StreamSelector<F> {
    pub fn new(inner: F, stream: usize) -> Self {
        let spec = inner.streams()[stream];

        Self {
            inner,
            spec,
            stream,
        }
    }
}

impl<F: Format> Stream for StreamSelector<F> {
    type Tag = <F::Tag as FormatTag>::Codec;

    fn stream_spec(&self) -> &StreamSpec<Self::Tag> {
        &self.spec
    }
}

impl<F: BlockingFormat> BlockingStream for StreamSelector<F> {
    fn block(&self) {
        self.inner.block()
    }
}

impl<F: IndexedFormat> IndexedStream for StreamSelector<F> {
    fn pos(&self) -> u64 {
        self.inner.stream_pos(self.stream)
    }
}

impl<F: FiniteFormat> FiniteStream for StreamSelector<F> {
    fn len(&self) -> u64 {
        self.inner.stream_len(self.stream)
    }
}

impl<T: FormatReader> StreamReader for StreamSelector<T> {
    fn read(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        loop {
            match self.inner.read(buf) {
                Ok((i, n)) if i == self.stream => return Ok(n),
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }
    }
}

impl<T: FormatWriter> StreamWriter for StreamSelector<T> {
    fn write(&mut self, buf: &[u8]) -> PhonicResult<usize> {
        self.inner.write(self.stream, buf)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T: FormatSeeker> StreamSeeker for StreamSelector<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.inner.seek(self.stream, offset)
    }
}
