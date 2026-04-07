
# Rustdivert 

rust-native interface to WinDivert

Instead of relying on the provided user-mode library, this crate performs raw communication with the driver via CreateFile, DeviceIoControl, and related Win32 APIs. This approach removes the DLL dependency while retaining full compatibility with the WinDivert driver.

Key Features
- Direct interaction with WinDivert.sys (no DLL layer)
- Packet capture, filtering, and injection
- Minimal abstraction over native Win32/IOCTL interface
- Suitable for low-level networking and systems programming