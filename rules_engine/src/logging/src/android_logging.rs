#[cfg(target_os = "android")]
use std::ffi::CString;

#[cfg(target_os = "android")]
use android_log_sys::{__android_log_write, LogPriority};

/// Writes a string to Android logs under the "dreamtides" tag.
pub fn write_to_logcat(s: impl Into<String>) {
    #[cfg(target_os = "android")]
    {
        let msg = s.into();
        if let (Ok(tag_cstr), Ok(msg_cstr)) = (CString::new("dreamtides"), CString::new(msg)) {
            unsafe {
                __android_log_write(LogPriority::INFO as i32, tag_cstr.as_ptr(), msg_cstr.as_ptr());
            }
        }
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = s.into();
    }
}
