use windows::{core::{PCSTR, PCWSTR}, Win32::Storage::FileSystem::{FILE_READ_DATA, FILE_WRITE_DATA}};

pub const MAX_PATH: usize = 260;

macro_rules! wide_string {
    ($s:literal) => {{
        const BYTES: &[u8] = concat!($s, "\0").as_bytes();
        const WIDE: [u16; BYTES.len()] = {
            let mut wide = [0u16; BYTES.len()];
            let mut i = 0;
            while i < BYTES.len() {
                wide[i] = BYTES[i] as u16;
                i += 1;
            }
            wide
        };
        PCWSTR::from_raw(WIDE.as_ptr())
    }};
}

macro_rules! ansi_string {
    ($s:literal) => {{
        const BYTES: &[u8] = concat!($s, "\0").as_bytes();
        PCSTR::from_raw(BYTES.as_ptr())
    }};
}

pub const EVENTLOG_REGISTRY_PATH: PCSTR = ansi_string!(r#"System\CurrentControlSet\Services\EventLog\System\WinDivert"#);

pub const WINDIVERT_32_SYS: &str = "windivert32.sys";
pub const WINDIVERT_64_SYS: &str = "windivert64.sys";

pub const WINDIVERT_DRIVER_NAME: PCWSTR = PCWSTR::from_raw(&[
    'W' as u16, 'i' as u16, 'n' as u16, 'D' as u16, 'i' as u16,
    'v' as u16, 'e' as u16, 'r' as u16, 't' as u16, 0
] as *const u16);

pub const WINDIVERT_PIPE_NAME: PCWSTR = PCWSTR::from_raw(&[
    '\\' as u16, '\\' as u16, '.' as u16, '\\' as u16,
    'W' as u16, 'i' as u16, 'n' as u16, 'D' as u16,
    'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16,
    't' as u16, 0
] as *const u16);

pub const  WINDIVERT_MUTEX_NAME: PCWSTR =  PCWSTR::from_raw(&[
    'W' as u16, 'i' as u16, 'n' as u16, 'D' as u16,
    'i' as u16, 'v' as u16, 'e' as u16, 'r' as u16,
    't' as u16, 'D' as u16, 'r' as u16, 'i' as u16,
    'v' as u16, 'e' as u16, 'r' as u16, 'I' as u16,
    'n' as u16, 's' as u16, 't' as u16, 'a' as u16,
    'l' as u16, 'l' as u16, 'M' as u16, 'u' as u16,
    't' as u16, 'e' as u16, 'x' as u16, 0
] as *const u16);

pub const WINDIVERT_FILTER_MAXLEN: usize = 256;
pub const FILTER_RESULT_ACCEPT: i16 = 0x7FFE;
pub const FILTER_RESULT_REJECT: i16 = 0x7FFF;
pub const FILTER_MAXLEN: usize = 1024;
pub const WINDIVERT_PRIORITY_MAX: i32 = WINDIVERT_PRIORITY_HIGHEST;
pub const WINDIVERT_PRIORITY_MIN: i32 = WINDIVERT_PRIORITY_LOWEST;
pub const WINDIVERT_PRIORITY_HIGHEST: i32 = 30000;
pub const WINDIVERT_PRIORITY_LOWEST: i32 = -WINDIVERT_PRIORITY_HIGHEST;
pub const WINDIVERT_MAGIC_DLL: u64 = 0x4C4C447669645724;
pub const WINDIVERT_MAGIC_SYS: u64 = 0x5359537669645723;
pub const WINDIVERT_VERSION_MAJOR: u32 = 2;
pub const WINDIVERT_VERSION_MINOR: u32 = 2;
pub const FILE_DEVICE_NETWORK: u32 = 0x00000012;
pub const METHOD_IN_DIRECT: u32 = 1;
pub const METHOD_OUT_DIRECT: u32 = 2;

const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

pub const IOCTL_WINDIVERT_INITIALIZE: u32 = ctl_code(
    FILE_DEVICE_NETWORK,
    0x921,
    METHOD_OUT_DIRECT,
    FILE_READ_DATA.0 | FILE_WRITE_DATA.0,
);

pub const IOCTL_WINDIVERT_STARTUP: u32 = ctl_code(
    FILE_DEVICE_NETWORK,
    0x922,
    METHOD_IN_DIRECT,
    FILE_READ_DATA.0 | FILE_WRITE_DATA.0,
);

pub const IOCTL_WINDIVERT_RECV: u32 = ctl_code(
    FILE_DEVICE_NETWORK,
    0x923,
    METHOD_OUT_DIRECT,
    FILE_READ_DATA.0,
);

pub const IOCTL_WINDIVERT_SEND: u32 = ctl_code(
    FILE_DEVICE_NETWORK,
    0x924,
    METHOD_IN_DIRECT,
    FILE_READ_DATA.0 | FILE_WRITE_DATA.0,
);

pub const IOCTL_WINDIVERT_SET_PARAM: u32 = ctl_code(
    FILE_DEVICE_NETWORK,
    0x925,
    METHOD_IN_DIRECT,
    FILE_READ_DATA.0 | FILE_WRITE_DATA.0,
);

pub const IOCTL_WINDIVERT_GET_PARAM: u32 = ctl_code(
    FILE_DEVICE_NETWORK,
    0x926,
    METHOD_OUT_DIRECT,
    FILE_READ_DATA.0,
);

pub const IOCTL_WINDIVERT_SHUTDOWN: u32 = ctl_code(
    FILE_DEVICE_NETWORK,
    0x927,
    METHOD_IN_DIRECT,
    FILE_READ_DATA.0 | FILE_WRITE_DATA.0,
);