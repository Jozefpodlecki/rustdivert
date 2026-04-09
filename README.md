
# Rustdivert 

rust-native interface to [WinDivert](https://github.com/basil00/WinDivert/)

Instead of relying on the provided user-mode library, this crate performs raw communication with the driver via CreateFile, DeviceIoControl, and related Win32 APIs. This approach removes the DLL dependency while retaining full compatibility with the WinDivert driver.

Key Features
- Direct interaction with WinDivert.sys (no DLL layer)
- Packet capture, filtering, and injection
- Minimal abstraction over native Win32/IOCTL interface
- Suitable for low-level networking and systems programming

## 🧰 Useful Commands

### ▶️ Start WinDivert manually

```sh
sc.exe start WinDivert
```

### 🔍 Check WinDivert driver status

```ps1
Get-Service -Name "WinDivert"
sc.exe qc WinDivert
Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Services\WinDivert"
Get-CimInstance -Class Win32_SystemDriver  -Filter "Name LIKE 'WinDivert'"
```

### ⚙️ Set startup mode

```sh
sc.exe config WinDivert start= demand
```

### 🛑 Stop driver

```sh
sc.exe stop WinDivert
```

### 🧨 Remove driver

```ps1
sc.exe delete WinDivert
# Generally not needed as delete removes registry entry
Remove-Item -Path "HKLM:\SYSTEM\CurrentControlSet\Services\WinDivert" -Recurse -Force
```