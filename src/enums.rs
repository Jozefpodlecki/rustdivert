use std::fmt;


#[derive(Debug)]
pub enum WinDivertError {
    InvalidParameter,
    BadObject,
    NoMemory,
    UnexpectedToken(usize),
    TooLong,
    TokenizeError(usize),
    ParseError(usize),
    BadToken(usize),
    TooDeep(usize),
    DeviceIoControl(u32),
    File(u32),
    CouldNotInstallService(u32),
    Handle(u32)
}

impl std::error::Error for WinDivertError {}
impl std::fmt::Display for WinDivertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WinDivertError::InvalidParameter => write!(f, "Invalid parameter"),
            WinDivertError::BadObject => write!(f, "Bad object"),
            WinDivertError::NoMemory => write!(f, "Out of memory"),
            WinDivertError::CouldNotInstallService(code) => write!(f, "Could not install service {}", code),
            WinDivertError::UnexpectedToken(pos) => write!(f, "Unexpected token at position {}", pos),
            WinDivertError::TooLong => write!(f, "Data too long"),
            WinDivertError::TokenizeError(pos) => write!(f, "Tokenization error at position {}", pos),
            WinDivertError::ParseError(pos) => write!(f, "Parse error at position {}", pos),
            WinDivertError::BadToken(pos) => write!(f, "Bad token at position {}", pos),
            WinDivertError::TooDeep(pos) => write!(f, "Too deeply nested at position {}", pos),
            WinDivertError::DeviceIoControl(code) => write!(f, "DeviceIoControl failed with code {}", code),
            WinDivertError::File(code) => write!(f, "File operation failed with code {}", code),
            WinDivertError::Handle(code) => write!(f, "Handle operation failed with code {}", code),
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