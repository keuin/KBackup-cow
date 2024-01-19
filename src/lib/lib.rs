use crate::CopyError::{FileOpenError, IoctlError};
use jni::descriptors::Desc;
use jni::objects::{JClass, JString};
use jni::strings::JNIString;
use jni::sys::jstring;
use jni::JNIEnv;
use log::debug;
use nix::libc::{ioctl, FICLONE};
use std::fmt::{Formatter, Write};
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::{fmt, io};

fn throw<S: Into<JNIString>>(env: &mut JNIEnv, msg: S) {
    throw_new(env, "java/lang/RuntimeException", msg);
}

fn throw_new<'a, 'b, S: Into<JNIString>, T: Desc<'a, JClass<'b>>>(
    env: &mut JNIEnv<'a>,
    ex: T,
    msg: S,
) {
    if let Err(why) = env.throw_new(ex, msg) {
        env.fatal_error(format!("failed to throw new exception: {why}"));
    }
}

#[no_mangle]
pub extern "system" fn Java_com_keuin_kbackupfabric_util_cow_FileCowCopier_init<'local>(
    _env: JNIEnv<'local>, _class: JClass<'local>) {
    env_logger::init();
    debug!("kbackup-cow init");
}

#[no_mangle]
pub extern "system" fn Java_com_keuin_kbackupfabric_util_cow_FileCowCopier_getVersion<'local>(
    env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jstring {
    match env.new_string(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")) {
        Ok(s) => s.into_raw(),
        Err(why) => env.fatal_error(format!("failed to create String: {why}")),
    }
}

#[no_mangle]
pub extern "system" fn Java_com_keuin_kbackupfabric_util_cow_FileCowCopier_copy<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    dst: JString<'local>,
    src: JString<'local>,
) {
    unsafe {
        let src = match env.get_string_unchecked(&src) {
            Ok(s) => s,
            Err(why) => {
                throw_new(
                    &mut env,
                    "java/lang/IllegalArgumentException",
                    format!("invalid src: {why}"),
                );
                return;
            }
        };
        let dst = match env.get_string_unchecked(&dst) {
            Ok(s) => s,
            Err(why) => {
                throw_new(
                    &mut env,
                    "java/lang/IllegalArgumentException",
                    format!("invalid dst: {why}"),
                );
                return;
            }
        };
        match copy(dst.into(), src.into()) {
            Ok(_) => {}
            Err(why) => {
                throw_new(&mut env, "java/io/IOException", format!("{}", why));
            }
        };
    }
}

enum CopyError {
    IoctlError(io::Error),
    FileOpenError(String, io::Error),
}

impl fmt::Display for CopyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            IoctlError(why) => {
                write!(f, "ioctl error: {why}")
            }
            FileOpenError(path, why) => {
                write!(f, "error opening file `{path}`: {why}")
            }
        }
    }
}

fn copy(dst: String, src: String) -> Result<(), CopyError> {
    debug!("copy src: {src}, dst: {dst}");
    let f_src = match OpenOptions::new().read(true).open(&src) {
        Ok(f) => f,
        Err(why) => {
            return Err(FileOpenError(src.clone(), why));
        }
    };
    let f_dst = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&dst)
    {
        Ok(f) => f,
        Err(why) => {
            return Err(FileOpenError(dst.clone(), why));
        }
    };
    unsafe {
        let fd_dst = f_dst.as_raw_fd();
        let fd_src = f_src.as_raw_fd();
        debug!("fd_src: {fd_src}, fd_dst: {fd_dst}");
        if ioctl(fd_dst, FICLONE, fd_src) != 0 {
            let err = io::Error::last_os_error();
            debug!("ioctl failed: {:?}", err);
            return Err(IoctlError(err));
        }
    }
    Ok(())
}
