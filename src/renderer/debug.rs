use std::{ffi::c_void, panic, ptr};

use gl::types::{GLchar, GLenum, GLsizei};

#[derive(Clone, Debug)]
pub struct DebugMessage {
    id: GLenum,
    source: DebugSource,
    type_: DebugType,
    severity: DebugSeverity,
    message: String,
}

#[derive(Clone, Copy, Debug)]
pub enum DebugSource {
    Api,
    WindowSystem,
    ShaderCompiler,
    ThirdParty,
    Application,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub enum DebugType {
    Error,
    DeprecatedBehavior,
    UndefinedBehavior,
    Portability,
    Performance,
    Marker,
    Other,
    PopGroup,
    PushGroup,
}

#[derive(Clone, Copy, Debug)]
pub enum DebugSeverity {
    Notification,
    Low,
    Medium,
    High,
}

pub struct DebugCallback {
    _user_callback: Box<Box<dyn Fn(DebugMessage)>>,
}

impl DebugCallback {
    pub unsafe fn new<F>(user_callback: F) -> Self
    where
        F: Fn(DebugMessage) + 'static + Send + panic::RefUnwindSafe,
    {
        // let mut debug_already_enabled = 0;
        // gl::GetBooleanv(gl::DEBUG_OUTPUT, &mut debug_already_enabled);
        // if debug_already_enabled != 0 {
        //     panic!("Debug is already enabled");
        // }

        let user_callback = Box::new(Box::new(user_callback) as Box<_>);

        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(
            Some(debug_callback),
            &*user_callback as &Box<_> as *const Box<_> as *const c_void,
        );

        Self {
            _user_callback: user_callback,
        }
    }
}

impl Drop for DebugCallback {
    fn drop(&mut self) {
        unsafe {
            gl::DebugMessageCallback(None, ptr::null());
            gl::Disable(gl::DEBUG_OUTPUT);
        }
    }
}

extern "system" fn debug_callback(
    source: GLenum,
    type_: GLenum,
    id: GLenum,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    user_param: *mut c_void,
) {
    let user_callback = user_param as *mut Box<dyn Fn()> as *const _;
    let user_callback: &Box<dyn Fn(DebugMessage)> = unsafe { &*user_callback };

    let msg_src = match source {
        gl::DEBUG_SOURCE_API => DebugSource::Api,
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => DebugSource::WindowSystem,
        gl::DEBUG_SOURCE_SHADER_COMPILER => DebugSource::ShaderCompiler,
        gl::DEBUG_SOURCE_THIRD_PARTY => DebugSource::ThirdParty,
        gl::DEBUG_SOURCE_APPLICATION => DebugSource::Application,
        gl::DEBUG_SOURCE_OTHER => DebugSource::Other,
        _ => unreachable!(),
    };

    let msg_type = match type_ {
        gl::DEBUG_TYPE_ERROR => DebugType::Error,
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => DebugType::DeprecatedBehavior,
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => DebugType::UndefinedBehavior,
        gl::DEBUG_TYPE_PORTABILITY => DebugType::Portability,
        gl::DEBUG_TYPE_PERFORMANCE => DebugType::Performance,
        gl::DEBUG_TYPE_MARKER => DebugType::Marker,
        gl::DEBUG_TYPE_OTHER => DebugType::Other,
        gl::DEBUG_TYPE_POP_GROUP => DebugType::PopGroup,
        gl::DEBUG_TYPE_PUSH_GROUP => DebugType::PushGroup,
        _ => unreachable!(),
    };

    let msg_severity = match severity {
        gl::DEBUG_SEVERITY_NOTIFICATION => DebugSeverity::Notification,
        gl::DEBUG_SEVERITY_LOW => DebugSeverity::Low,
        gl::DEBUG_SEVERITY_MEDIUM => DebugSeverity::Medium,
        gl::DEBUG_SEVERITY_HIGH => DebugSeverity::High,
        _ => unreachable!(),
    };

    let msg =
        unsafe { String::from_raw_parts(message as *mut u8, length as usize, length as usize) };

    let message = DebugMessage {
        id,
        source: msg_src,
        type_: msg_type,
        severity: msg_severity,
        message: msg,
    };

    user_callback(message);
}
