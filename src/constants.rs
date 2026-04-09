use windows::{Win32::Storage::FileSystem::{FILE_READ_DATA, FILE_WRITE_DATA}, core::{PCSTR, PCWSTR, s, w}};

pub const MAX_PATH: usize = 260;
pub const EVENTLOG_REGISTRY_PATH: PCSTR = s!(r#"System\CurrentControlSet\Services\EventLog\System\WinDivert"#);
pub const WINDIVERT_32_SYS: &str = "WinDivert32.sys";
pub const WINDIVERT_64_SYS: &str = "WinDivert64.sys";
pub const WINDIVERT_DRIVER_NAME: PCWSTR = w!("WinDivert");
pub const WINDIVERT_PIPE_NAME: PCWSTR = w!(r#"\\.\WinDivert"#);
pub const WINDIVERT_MUTEX_NAME: PCWSTR = w!("WinDivertDriverInstallMutex");

pub const WINDIVERT_FILTER_MAXLEN: usize = 256;
pub const FILTER_RESULT_ACCEPT: i16 = 0x7FFE;
pub const FILTER_RESULT_REJECT: i16 = 0x7FFF;
pub const FILTER_MAXLEN: usize = 1024;
pub const WINDIVERT_PRIORITY_MAX: i16 = WINDIVERT_PRIORITY_HIGHEST;
pub const WINDIVERT_PRIORITY_MIN: i16 = WINDIVERT_PRIORITY_LOWEST;
pub const WINDIVERT_PRIORITY_HIGHEST: i16 = 30000;
pub const WINDIVERT_PRIORITY_LOWEST: i16 = -WINDIVERT_PRIORITY_HIGHEST;
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