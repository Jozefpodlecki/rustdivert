# RustDivert Examples

This repository contains examples demonstrating how to use the rustdivert crate.
The examples cover:

- Setting up packet filters and parameters  
- Sending and receiving packets  
- Using TCP client/server examples with packet interception  

---

## ⚠️ Requirements

- **Windows only** (WinDivert is Windows-specific)  
- **Administrator privileges are required** for all examples, even for loopback traffic (`127.0.0.1`).  
- Rust toolchain installed (`cargo` & `rustc`) 

---

## 🔧 Running with Administrator Rights

You can launch PowerShell as admin and run the examples like this:

```powershell
Start-Process powershell -Verb RunAs -ArgumentList "-NoExit", "-Command", "Set-Location '$pwd'; `$env:RUST_BACKTRACE='1'; cargo run"