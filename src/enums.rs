use std::fmt;

use crate::WinDivertAddress;

pub struct WindivertOptions {
    pub install_service_on_file_not_found: bool
}

impl Default for WindivertOptions {
    fn default() -> Self {
        Self {
            install_service_on_file_not_found: Default::default()
        }
    }
}

pub struct WinDivertPacket {
    pub received: u32,
    pub address: WinDivertAddress,
    pub data: Box<[u8]>,
}

pub struct WinDivertPacketBatch {
    pub received: u32,
    pub packets: Vec<WinDivertPacket>,
}

#[derive(Debug)]
pub enum WinDivertError {
    InvalidParameter,
    CouldNotLockForInstall,
    Cancelled,
    BadObject,
    NoMemory,
    UnexpectedToken(usize),
    TooLong,
    TokenizeError(usize),
    ParseError(usize),
    BadToken(usize),
    TooDeep(usize),
    CouldNotInitialize(u32),
    CouldNotSend(u32),
    CouldNotSetParam(u32),
    CouldNotGetParam(u32),
    CouldNotReceive(u32),
    FileNotFound,
    AccessDenied,
    ServiceExists,
    ServiceAlreadyRunning,
    CorruptedService,
    CouldNotInstallService(u32),
    CouldNotMarkServiceForDeletion(u32),
    Handle(u32)
}

impl std::error::Error for WinDivertError {}
impl std::fmt::Display for WinDivertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WinDivertError::InvalidParameter =>
                write!(f, "Invalid parameter"),

            WinDivertError::BadObject =>
                write!(f, "Invalid or corrupted object"),

            WinDivertError::NoMemory =>
                write!(f, "Out of memory"),

            WinDivertError::Cancelled =>
                write!(f, "Operation was cancelled"),

            WinDivertError::AccessDenied =>
                write!(f, "Access denied (administrator privileges required)"),

            WinDivertError::FileNotFound =>
                write!(f, "WinDivert driver device not found (driver not installed?)"),

            WinDivertError::CouldNotLockForInstall =>
                write!(f, "Could not acquire installation mutex"),

            // --- Parsing / filter errors ---
            WinDivertError::UnexpectedToken(pos) =>
                write!(f, "Unexpected token at position {}", pos),

            WinDivertError::TooLong =>
                write!(f, "Input too long"),

            WinDivertError::TokenizeError(pos) =>
                write!(f, "Tokenization error at position {}", pos),

            WinDivertError::ParseError(pos) =>
                write!(f, "Parse error at position {}", pos),

            WinDivertError::BadToken(pos) =>
                write!(f, "Invalid token at position {}", pos),

            WinDivertError::TooDeep(pos) =>
                write!(f, "Expression too deeply nested at position {}", pos),

            // --- Driver / IOCTL errors ---
            WinDivertError::CouldNotInitialize(code) =>
                write!(f, "Failed to initialize WinDivert driver (code {})", code),

            WinDivertError::CouldNotSend(code) =>
                write!(f, "Failed to send packet via WinDivert (code {})", code),

            WinDivertError::CouldNotReceive(code) =>
                write!(f, "Failed to receive packet from WinDivert (code {})", code),

            WinDivertError::CouldNotSetParam(code) =>
                write!(f, "Failed to set WinDivert parameter (code {})", code),

            WinDivertError::CouldNotGetParam(code) =>
                write!(f, "Failed to get WinDivert parameter (code {})", code),

            // --- Service / driver install errors ---
            WinDivertError::ServiceExists =>
                write!(f, "WinDivert service already exists"),

            WinDivertError::ServiceAlreadyRunning =>
                write!(f, "WinDivert service is already running"),

            WinDivertError::CorruptedService =>
                write!(f, "WinDivert service is corrupted or misconfigured"),

            WinDivertError::CouldNotInstallService(code) =>
                write!(f, "Failed to install/start WinDivert service (code {})", code),

            WinDivertError::CouldNotMarkServiceForDeletion(code) =>
                write!(f, "Failed to mark WinDivert service for deletion (code {})", code),

            // --- Generic handle error ---
            WinDivertError::Handle(code) =>
                write!(f, "Win32 handle operation failed (code {})", code),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WinDivertParam {
    QueueLength,
    QueueTime,
    QueueSize,
    VersionMajor,
    VersionMinor
}

impl fmt::Display for WinDivertParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            WinDivertParam::QueueLength => "Queue Length (packets)",
            WinDivertParam::QueueTime   => "Queue Time (ms)",
            WinDivertParam::QueueSize   => "Queue Size (bytes)",
            WinDivertParam::VersionMajor => "Version Major",
            WinDivertParam::VersionMinor => "Version Minor",
        };

        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WinDivertShutdown {
    Recv = 1,
    Send = 2,
    Both = 3 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum WinDivertLayer {
    Network = 0,
    NetworkForward = 1,
    Flow = 2,
    Socket = 3,
    Reflect = 4,
}

#[repr(u32)]
pub enum WinDivertEvent {
    FlowDeleted = 0,
    SocketBind = 1,
    SocketConnect = 2,
    SocketClose = 3,
    SocketListen = 4,
    SocketAccept = 5,
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WinDivertFilterFlag {
    Inbound = 0x0000000000000010,   // 16
    Outbound = 0x0000000000000020,  // 32
    Ip = 0x0000000000000040,        // 64
    Ipv6 = 0x0000000000000080,      // 128
    EventFlowDeleted = 0x0000000000000100,   // 256
    EventSocketBind = 0x0000000000000200,    // 512
    EventSocketConnect = 0x0000000000000400, // 1024
    EventSocketListen = 0x0000000000000800,  // 2048
    EventSocketAccept = 0x0000000000001000,  // 4096
    EventSocketClose = 0x0000000000002000,   // 8192
}