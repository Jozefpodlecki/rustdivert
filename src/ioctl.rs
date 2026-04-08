use crate::{constants::{WINDIVERT_MAGIC_DLL, WINDIVERT_VERSION_MAJOR, WINDIVERT_VERSION_MINOR}, *};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WinDivertVersion {
    pub magic: u64,
    pub major: u32,
    pub minor: u32,
    pub bits: u32,
    pub reserved32: [u32; 3],
    pub reserved64: [u64; 4],
}

impl WinDivertVersion {
    pub fn new() -> Self {

        Self {
            magic: WINDIVERT_MAGIC_DLL,
            major: WINDIVERT_VERSION_MAJOR,
            minor: WINDIVERT_VERSION_MINOR,
            bits: (std::mem::size_of::<usize>() * 8) as u32,
            reserved32: [0; 3],
            reserved64: [0; 4],
        }
    }

    pub fn size_of() -> u32 {
        std::mem::size_of::<Self>() as u32
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union WinDivertIoctl {
    pub recv: Recv,
    pub send: Send,
    pub initialize: Initialize,
    pub startup: Startup,
    pub shutdown: Shutdown,
    pub get_param: GetParam,
    pub set_param: SetParam,
}

impl WinDivertIoctl {
    pub fn initialize(layer: u32, priority: u32, flags: u64) -> Self {
        let initialize = Initialize { layer, priority, flags };

        Self {
            initialize
        }
    }

    pub fn get_param(param: WinDivertParam) -> Self {
        Self {
            get_param: GetParam { param }
        }
    }

    pub fn set_param(param: WinDivertParam, value: u64) -> Self {
        Self {
            set_param: SetParam {
                param,
                value
            }
        }
    }

    pub fn recv(addr: *const ioctl::WinDivertAddress) -> Self {
        Self {
            recv: Recv { addr, addr_len_ptr: 0 }
        }
    }

    pub fn send(addr: *const ioctl::WinDivertAddress) -> Self {
        Self {
            send: Send {
                addr,
                addr_len: std::mem::size_of::<WinDivertAddress>() as u64
            }
        }
    }

    pub fn send_ex(addr: *const ioctl::WinDivertAddress, addr_len: u64) -> Self {
        Self {
            send: Send {
                addr,
                addr_len: addr_len * std::mem::size_of::<WinDivertAddress>() as u64
            }
        }
    }

    pub fn startup(flags: u64) -> Self {
        Self {
            startup: Startup { flags }
        }
    }

    pub fn shutdown(how: WinDivertShutdown) -> Self {
        Self {
            shutdown: Shutdown { how: how as u32 }
        }
    }

    pub const fn size_of() -> u32 {
        std::mem::size_of::<Self>() as u32
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Recv {
    pub addr: *const ioctl::WinDivertAddress,
    pub addr_len_ptr: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Send {
    pub addr: *const ioctl::WinDivertAddress,
    pub addr_len: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Initialize {
    pub layer: u32,
    pub priority: u32,
    pub flags: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Startup {
    pub flags: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Shutdown {
    pub how: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GetParam {
    pub param: WinDivertParam,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SetParam {
    pub value: u64,
    pub param: WinDivertParam,
}

use std::mem::ManuallyDrop;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct WinDivertAddress {
    pub timestamp: i64,
    pub bits: u32,
    pub reserved2: u32,
    pub data: WinDivertAddressData,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union WinDivertAddressData {
    pub network: ManuallyDrop<WinDivertDataNetwork>,
    pub flow: ManuallyDrop<WinDivertDataFlow>,
    pub socket: ManuallyDrop<WinDivertDataSocket>,
    pub reflect: ManuallyDrop<WinDivertDataReflect>,
    pub reserved3: ManuallyDrop<[u8; 64]>,
}

impl WinDivertAddress {
    pub fn layer(&self) -> u8 {
        (self.bits & 0xFF) as u8
    }
    
    pub fn event(&self) -> u8 {
        ((self.bits >> 8) & 0xFF) as u8
    }
    
    pub fn sniffed(&self) -> bool {
        ((self.bits >> 16) & 1) != 0
    }
    
    pub fn outbound(&self) -> bool {
        ((self.bits >> 17) & 1) != 0
    }
    
    pub fn loopback(&self) -> bool {
        ((self.bits >> 18) & 1) != 0
    }
    
    pub fn impostor(&self) -> bool {
        ((self.bits >> 19) & 1) != 0
    }
    
    pub fn ipv6(&self) -> bool {
        ((self.bits >> 20) & 1) != 0
    }
    
    pub fn ip_checksum(&self) -> bool {
        ((self.bits >> 21) & 1) != 0
    }
    
    pub fn tcp_checksum(&self) -> bool {
        ((self.bits >> 22) & 1) != 0
    }
    
    pub fn udp_checksum(&self) -> bool {
        ((self.bits >> 23) & 1) != 0
    }
    
    pub fn reserved1(&self) -> u8 {
        ((self.bits >> 24) & 0xFF) as u8
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WinDivertDataNetwork {
    pub if_idx: u32,
    pub sub_if_idx: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WinDivertDataFlow {
    pub endpoint_id: u64,
    pub parent_endpoint_id: u64,
    pub process_id: u32,
    pub local_addr: [u32; 4],
    pub remote_addr: [u32; 4],
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WinDivertDataSocket {
    pub endpoint_id: u64,
    pub parent_endpoint_id: u64,
    pub process_id: u32,
    pub local_addr: [u32; 4],
    pub remote_addr: [u32; 4],
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct WinDivertDataReflect {
    pub timestamp: i64,
    pub process_id: u32,
    pub layer: u32,
    pub flags: u64,
    pub priority: i16,
}