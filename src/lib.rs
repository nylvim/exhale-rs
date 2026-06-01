//! High level bindings to exhale, the open-source Extended HE-AAC encoder.
//!
//! See upstream docs for details: <https://gitlab.com/ecodis/exhale/-/wikis/api>

/// Low level FFI bindings.
pub mod sys;

use core::ffi::{c_uchar, c_uint};
use core::ptr::NonNull;

/// Channel configuration of the input.
#[derive(Debug, Clone, Copy, Default)]
pub enum Channels {
    /// 2.0, dual-mono: channel1, channel2
    DualMono = 0,
    /// 1.0: front-center
    Mono = 1,
    /// 2.0: front-left, front-right
    #[default]
    Stereo = 2,
    /// 3.0: front-center, front-left, front-right
    ThreePointZero = 3,
    /// 4.0: front-center, front-left, front-right, back-center
    FourPointZero = 4,
    /// 5.0: front-center, front-left, front-right, back-left, back-right
    FivePointZero = 5,
    /// 5.1: front-center, front-left, front-right, back-left, back-right, LFE
    FivePointOne = 6,
}

impl Channels {
    /// Number of channels.
    #[inline]
    pub const fn count(self) -> usize {
        match self {
            Self::DualMono => 2,
            _ => self as usize,
        }
    }
}

/// Configuration used to create the encoder.
#[derive(Debug, Clone, Copy)]
pub struct EncoderConfig {
    /// Sample rate of the input.
    pub sample_rate: u32,
    /// Channel configuration of the input.
    pub channels: Channels,
    /// Independent frame interval.
    pub indep_period: u32,
    /// Variable bitrate level, must be in range `0..=12`.
    pub vbr_level: u8,
    /// Enable 2:1 spectral band replication.
    pub enable_sbr: bool,
    /// Use USAC noise filling functionality.
    pub use_noise_filling: bool,
}

impl Default for EncoderConfig {
    #[inline]
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: Channels::Stereo,
            indep_period: 45,
            vbr_level: 3,
            enable_sbr: false,
            use_noise_filling: true,
        }
    }
}

/// An `exhale` xHE-AAC encoder.
///
/// # Example
///
/// ```no_run
/// use exhale::{Encoder, EncoderConfig};
///
/// let mut encoder = Encoder::new(EncoderConfig::default()).unwrap();
/// let mut input = vec![0i32; encoder.frame_size() * 2];
/// // fill input...
/// let output = encoder.encode_frame_with(&input).unwrap();
/// ```
#[derive(Debug)]
pub struct Encoder {
    ptr: NonNull<sys::ExhaleEncAPI>,
    input: Box<[i32]>,
    output: Box<[u8]>,
    asc: [u8; 16],
    asc_size: u8,
    frame_size: u16,
    is_first_frame: bool,
}

impl Drop for Encoder {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            sys::exhaleDelete(self.ptr.as_ptr());
        }
    }
}

unsafe impl Send for Encoder {}
unsafe impl Sync for Encoder {}

impl Encoder {
    /// Create and initialize an encoder.
    ///
    /// # Errors
    ///
    /// Returns an error if config is incorrect.
    pub fn new(config: EncoderConfig) -> Result<Self, Error> {
        if config.vbr_level > 12 {
            return Err(Error);
        }

        let frame_size = if config.enable_sbr { 2048 } else { 1024 };

        let mut input = vec![0; frame_size * config.channels.count()].into_boxed_slice();
        let mut output = vec![0; (6144 / 8) * config.channels.count()].into_boxed_slice();

        let ptr = unsafe {
            sys::exhaleCreate(
                input.as_mut_ptr(),
                output.as_mut_ptr() as *mut c_uchar,
                config.sample_rate as c_uint,
                config.channels as c_uint,
                frame_size as c_uint,
                config.indep_period as c_uint,
                config.vbr_level as c_uint,
                config.use_noise_filling,
                false,
            )
        };

        let mut asc = [0; 16];
        let mut asc_size = 0;
        let result =
            unsafe { sys::exhaleInitEncoder(ptr, asc.as_mut_ptr() as *mut c_uchar, &mut asc_size) };
        if result != 0 {
            return Err(Error);
        }

        Ok(Self {
            ptr: NonNull::new(ptr).unwrap(),
            input,
            output,
            asc,
            asc_size: asc_size as u8,
            frame_size: frame_size as u16,
            is_first_frame: true,
        })
    }

    /// Encode a frame.
    ///
    /// `input_mut` should be called and the input buffer be filled before calling this method.
    ///
    /// Returns a reference to the output, whose data is valid till next call of this method.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding failed.
    #[inline]
    pub fn encode_frame(&mut self) -> Result<&[u8], Error> {
        let output_len = if self.is_first_frame {
            self.is_first_frame = false;
            unsafe { sys::exhaleEncodeLookahead(self.ptr.as_ptr()) }
        } else {
            unsafe { sys::exhaleEncodeFrame(self.ptr.as_ptr()) }
        };

        if output_len < 2 {
            return Err(Error);
        }
        Ok(&self.output[..output_len as usize])
    }

    /// Encode a frame with given frame data.
    ///
    /// The input should be interleaved, unpacked signed 24-bit PCM data, that is,
    /// in range of `-8388608..=8388607`.
    ///
    /// Returns a reference to the output, whose data is valid till next call of this method.
    ///
    /// # Errors
    ///
    /// Returns an error if input is not long enough or encoding failed.
    #[inline]
    pub fn encode_frame_with(&mut self, input: &[i32]) -> Result<&[u8], Error> {
        if input.len() < self.input.len() {
            return Err(Error);
        }
        self.input.copy_from_slice(&input[..self.input.len()]);
        self.encode_frame()
    }

    /// Get a mutable reference to the input buffer.
    ///
    /// The input should be interleaved, unpacked signed 24-bit PCM data, that is,
    /// in range of `-8388608..=8388607`.
    #[inline]
    pub fn input_mut(&mut self) -> &mut [i32] {
        &mut self.input
    }

    /// Get the required input frame size per channel.
    #[inline]
    pub fn frame_size(&self) -> usize {
        self.frame_size as usize
    }

    /// Get the AudioSpecificConfig (ASC) data.
    #[inline]
    pub fn asc_data(&self) -> &[u8] {
        &self.asc[..self.asc_size as usize]
    }
}

/// Crate error type.
#[derive(Debug, Clone, Copy)]
pub struct Error;

impl std::fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exhale error")
    }
}

impl std::error::Error for Error {}
