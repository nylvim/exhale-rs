use std::ffi::{c_uchar, c_uint};

#[repr(C)]
pub struct ExhaleEncAPI {
    _unused: (),
}

unsafe extern "C" {
    pub fn exhaleCreate(
        inputPcmData: *mut i32,
        outputAuData: *mut c_uchar,
        sampleRate: c_uint,
        numChannels: c_uint,
        frameLength: c_uint,
        indepPeriod: c_uint,
        varBitRateMode: c_uint,
        useNoiseFilling: bool,
        useEcodisExt: bool,
    ) -> *mut ExhaleEncAPI;

    pub fn exhaleInitEncoder(
        exhaleEnc: *mut ExhaleEncAPI,
        audioConfigBuffer: *mut c_uchar,
        audioConfigBytes: *mut u32,
    ) -> c_uint;

    pub fn exhaleEncodeLookahead(exhaleEnc: *mut ExhaleEncAPI) -> c_uint;

    pub fn exhaleEncodeFrame(exhaleEnc: *mut ExhaleEncAPI) -> c_uint;

    pub fn exhaleDelete(exhaleEnc: *mut ExhaleEncAPI) -> c_uint;
}
