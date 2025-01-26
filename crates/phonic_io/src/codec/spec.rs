use crate::{CodecTag, TypeLayout};
use phonic_signal::{PhonicError, PhonicResult, Sample, Signal, SignalSpec, SignalSpecBuilder};
use std::{fmt::Debug, time::Duration};

#[derive(Debug, Clone, Copy)]
pub struct StreamSpec<C: CodecTag> {
    pub codec: C,
    pub avg_byte_rate: u32,
    pub block_align: usize,
    pub sample_layout: TypeLayout,
    pub decoded_spec: SignalSpec,
}

#[derive(Debug, Clone, Copy)]
pub struct StreamSpecBuilder<C: CodecTag> {
    pub codec: Option<C>,
    pub avg_byte_rate: Option<u32>,
    pub block_align: Option<usize>,
    pub sample_layout: Option<TypeLayout>,
    pub decoded_spec: SignalSpecBuilder,
}

impl<C: CodecTag> StreamSpec<C> {
    pub fn builder() -> StreamSpecBuilder<C> {
        StreamSpecBuilder::new()
    }

    pub fn into_builder(self) -> StreamSpecBuilder<C> {
        self.into()
    }

    pub fn with_tag_type<T>(self) -> StreamSpec<T>
    where
        T: CodecTag,
        C: Into<T>,
    {
        StreamSpec {
            codec: self.codec.into(),
            avg_byte_rate: self.avg_byte_rate,
            block_align: self.block_align,
            sample_layout: self.sample_layout,
            decoded_spec: self.decoded_spec,
        }
    }

    pub fn avg_byte_rate_duration(&self) -> Duration {
        let seconds = 1.0 / self.avg_byte_rate as f64;
        Duration::from_secs_f64(seconds)
    }

    pub fn avg_block_rate(&self) -> f64 {
        self.avg_byte_rate as f64 / self.block_align as f64
    }

    pub fn avg_bytes_per_sample(&self) -> f64 {
        self.avg_byte_rate as f64 / self.decoded_spec.sample_rate as f64
    }

    pub fn avg_bytes_per_frame(&self) -> f64 {
        self.avg_bytes_per_sample() * self.decoded_spec.channels.count() as f64
    }

    pub fn block_align_duration(&self) -> Duration {
        let seconds = self.block_align as f64 / self.avg_byte_rate as f64;
        Duration::from_secs_f64(seconds)
    }

    pub fn merge<T>(&mut self, other: &StreamSpec<T>) -> PhonicResult<()>
    where
        T: CodecTag + TryInto<C>,
        PhonicError: From<<T as TryInto<C>>::Error>,
    {
        let min_align = self.block_align.min(other.block_align);
        let max_align = self.block_align.max(other.block_align);

        if self.codec != other.codec.try_into()?
            || self.avg_byte_rate != other.avg_byte_rate
            || self.sample_layout != other.sample_layout
            || max_align % min_align != 0
        {
            return Err(PhonicError::param_mismatch());
        }

        self.block_align = max_align;
        self.decoded_spec.merge(&other.decoded_spec)
    }

    pub fn merged<T>(mut self, other: &StreamSpec<T>) -> PhonicResult<Self>
    where
        T: CodecTag + TryInto<C>,
        PhonicError: From<<T as TryInto<C>>::Error>,
    {
        self.merge(other)?;
        Ok(self)
    }
}

impl<C: CodecTag> StreamSpecBuilder<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tag_type<T>(self) -> PhonicResult<StreamSpecBuilder<T>>
    where
        T: CodecTag,
        C: TryInto<T>,
        PhonicError: From<<C as TryInto<T>>::Error>,
    {
        Ok(StreamSpecBuilder {
            codec: self.codec.map(TryInto::try_into).transpose()?,
            avg_byte_rate: self.avg_byte_rate,
            block_align: self.block_align,
            sample_layout: self.sample_layout,
            decoded_spec: self.decoded_spec,
        })
    }

    pub fn with_codec(mut self, codec: C) -> Self {
        self.codec = Some(codec);
        self
    }

    pub fn with_avg_byte_rate(mut self, byte_rate: u32) -> Self {
        self.avg_byte_rate = Some(byte_rate);
        self
    }

    pub fn with_block_align(mut self, block_align: usize) -> Self {
        self.block_align = Some(block_align);
        self
    }

    pub fn with_sample_layout(mut self, layout: TypeLayout) -> Self {
        self.sample_layout = Some(layout);
        self
    }

    pub fn with_sample_type<T: Sample + 'static>(self) -> Self {
        self.with_sample_layout(TypeLayout::of::<T>())
    }

    pub fn with_decoded_spec(mut self, decoded_spec: impl Into<SignalSpecBuilder>) -> Self {
        self.decoded_spec = decoded_spec.into();
        self
    }

    pub fn is_full(&self) -> bool {
        self.avg_byte_rate.is_some()
            && self.block_align.is_some()
            && self.sample_layout.is_some()
            && self.decoded_spec.is_full()
    }

    pub fn is_empty(&self) -> bool {
        self.avg_byte_rate.is_none()
            && self.block_align.is_none()
            && self.sample_layout.is_none()
            && self.decoded_spec.is_empty()
    }

    pub fn merge(&mut self, other: &Self) -> PhonicResult<()> {
        if other
            .codec
            .is_some_and(|codec| *self.codec.get_or_insert(codec) != codec)
        {
            return Err(PhonicError::param_mismatch());
        }

        if other
            .avg_byte_rate
            .is_some_and(|rate| *self.avg_byte_rate.get_or_insert(rate) != rate)
        {
            return Err(PhonicError::param_mismatch());
        }

        if let Some(align) = other.block_align {
            let self_align = self.block_align.unwrap_or(align);
            let min = align.min(self_align);
            let max = align.max(self_align);

            if max % min != 0 {
                return Err(PhonicError::param_mismatch());
            }

            self.block_align = Some(max);
        }

        self.decoded_spec.merge(&other.decoded_spec)
    }

    pub fn merged(mut self, other: &Self) -> PhonicResult<Self> {
        self.merge(other)?;
        Ok(self)
    }

    pub fn inferred(self) -> PhonicResult<StreamSpec<C>> {
        C::infer_spec(self)
    }

    pub fn build(self) -> PhonicResult<StreamSpec<C>> {
        self.try_into()
    }
}

impl<C: CodecTag> Default for StreamSpecBuilder<C> {
    fn default() -> Self {
        Self {
            codec: Default::default(),
            avg_byte_rate: Default::default(),
            block_align: Default::default(),
            sample_layout: Default::default(),
            decoded_spec: Default::default(),
        }
    }
}

impl<C: CodecTag> TryFrom<StreamSpecBuilder<C>> for StreamSpec<C> {
    type Error = PhonicError;

    fn try_from(spec: StreamSpecBuilder<C>) -> Result<StreamSpec<C>, Self::Error> {
        Ok(StreamSpec {
            codec: spec.codec.ok_or(PhonicError::missing_data())?,
            avg_byte_rate: spec.avg_byte_rate.ok_or(PhonicError::missing_data())?,
            block_align: spec.block_align.ok_or(PhonicError::missing_data())?,
            sample_layout: spec.sample_layout.ok_or(PhonicError::missing_data())?,
            decoded_spec: spec.decoded_spec.build()?,
        })
    }
}

impl<C: CodecTag> From<StreamSpec<C>> for StreamSpecBuilder<C> {
    fn from(spec: StreamSpec<C>) -> Self {
        Self {
            codec: spec.codec.into(),
            avg_byte_rate: spec.avg_byte_rate.into(),
            block_align: spec.block_align.into(),
            sample_layout: spec.sample_layout.into(),
            decoded_spec: spec.decoded_spec.into(),
        }
    }
}

impl<T: Signal, C: CodecTag> From<&T> for StreamSpecBuilder<C> {
    fn from(value: &T) -> Self {
        Self {
            sample_layout: Some(TypeLayout::of::<T::Sample>()),
            decoded_spec: value.spec().into_builder(),
            ..Self::default()
        }
    }
}
