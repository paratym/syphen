use crate::types::{
    FiniteSignalList, IndexedSignalList, SignalList, SignalReaderList, SignalSeekerList,
    SignalWriterList,
};
use phonic_core::PhonicError;
use phonic_signal::{
    FiniteSignal, IndexedSignal, Signal, SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};

pub struct Concat<T> {
    inner: T,
    spec: SignalSpec,
    current_i: usize,
}

impl<T: SignalList> Concat<T> {
    pub fn new(inner: T) -> Result<Self, PhonicError> {
        Ok(Self {
            current_i: 0,
            spec: inner.spec()?,
            inner,
        })
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: SignalList> Signal for Concat<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: IndexedSignalList> IndexedSignal for Concat<T> {
    fn pos(&self) -> u64 {
        let range = 0..=self.current_i;
        range.map(|i| self.inner.pos(i)).sum()
    }
}

impl<T: FiniteSignalList> FiniteSignal for Concat<T> {
    fn len(&self) -> u64 {
        let range = 0..self.inner.count();
        range.map(|i| self.inner.len(i)).sum()
    }
}

impl<T: SignalReaderList> SignalReader for Concat<T> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        while self.current_i < self.inner.count() {
            let n = self.inner.read(self.current_i, buf)?;
            if n == 0 {
                self.current_i += 1;
                continue;
            }

            return Ok(n);
        }

        Ok(0)
    }
}

impl<T: SignalWriterList> SignalWriter for Concat<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        while self.current_i < self.inner.count() {
            let n = self.inner.write(self.current_i, buf)?;
            if n == 0 {
                self.current_i += 1;
                continue;
            }
        }

        Ok(0)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        todo!()
    }
}

impl<T: IndexedSignalList + SignalSeekerList> SignalSeeker for Concat<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        todo!()
    }
}
