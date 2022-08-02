# Logitech G600 for Linux with Rust

Experimental program for using Logitech 600 mouse and its macro keys on Linux. Adapted from https://github.com/mafik/logitech-g600-linux

If you see this error when trying to run compiled program without sudo `Error: Os { code: 13, kind: PermissionDenied, message: "Permission denied" }`, run these commands:
```
sudo chown .input target/debug/g600-rust
sudo chmod g+s target/debug/g600-rust
```