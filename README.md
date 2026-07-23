<div align="center">
<h1> OpenE2E </h1>
<img src="doc/img/logo.png">
</div>

> [!WARNING]
>
> Under active development. Not ready for production use yet.
> See the [Roadmap](#roadmap).

**Languages:** [English](README.md) | [Русский](doc/README.ru.md)

# Overview

OpenE2E is a manual secure chat app for exchanging encrypted messages over any channel, from SMS and email to messengers, social platforms, or other untrusted channels.

The app handles encryption and decryption locally, while you move encrypted payloads between people using whatever channel is available. This makes it useful anywhere direct secure transport is unavailable or inconvenient.

OpenE2E uses **Matrix OLM** via the `voldozemac` library, with **AES-256-GCM** for message encryption and **end-to-end encryption with Perfect Forward Secrecy (PFS)**.

More information about the encryption model and internal design: [OLM.md](doc/OLM.md)

# Features

- **Manual Secure Chat** - Exchange encrypted messages by copy-pasting them through any channel
- **End-to-End Encryption** - Messages are encrypted locally and can only be decrypted by the intended recipient
- **Perfect Forward Secrecy** - Past messages stay protected even if later keys are compromised
- **Works Over Any Channel** - SMS, email, messengers, government platforms, and other public channels
- **Local Storage** - All data stays on your device
- **Encrypted Storage** - Messages and sessions are stored locally in an encrypted with AES-256-GCM database
- **Chat-Like Interface** - A clean UI built with **Slint**  [Work in progress]
- **Rust-Based** - Memory safe and blazingly fast

# How It Works

1. **Create a Session** - Start a new conversation in the app
2. **Exchange Public Keys** - Generate your ephemeral public key and share it with your contact by any channel
3. **Receive Their Key** - Paste your contact's public key into the app to establish the session
4. **Write a Message** - Enter your message in the app
5. **Encrypt and Copy** - The app encrypts the message locally and gives you the ciphertext to send anywhere
6. **Receive Ciphertext** - Paste the encrypted message from your contact into the app
7. **Decrypt Locally** - The app decrypts it on your device and shows it in a readable chat view
8. **Continue the Conversation** - Repeat the same process for ongoing secure communication

# Installation

## Installation from source

### Build from Source

**Requirements:**
- Rust

```bash
git clone https://codeberg.org/bazelik-dev/OpenE2E.git
cd OpenE2E
cargo build --release
./target/release/OpenE2E
```

## Installation from pre-built binaries

### Installation

- Go to [Releases](https://codeberg.org/bazelik-dev/OpenE2E/releases) and download latest binary

### Verification

- Download `.asc` signature file from releases tab.
- Verify: `gpg --auto-key-locate keyserver --keyserver-options auto-key-retrieve --verify OpenE2E*.asc OpenE2E*`
- Key should match key published at: https://keys.openpgp.org/vks/v1/by-fingerprint/C4C5BDC6C5E4C96CF12B3E85B7BBEB3BC5439F72


# Security Features

- **Perfect Forward Secrecy (PFS)** - OLM's ratchet-based design limits the impact of key compromise
- **End-to-End Encryption** - Only the two endpoints can read message contents
- **Local Storage Only** - No cloud sync, no server-side message storage
- **Manual Key Exchange** - No automatic trust assumptions
- **Channel Agnostic** - Encrypted data can travel through almost any medium

### Data Protection

Messages and sessions are stored locally in fjall DB, AES-256-GCM encrypts all message data at rest in fjall DB and AES-CBC-HMAC encrypts all sessions and accounts data. Each message uses a randomly generated 12-byte nonce to ensure ciphertext uniqueness.

All encryption keys are derived from your user password and stored in memory during the session. Keys are never written to disk or persisted after logout.

# Limitations

- Requires manual key exchange
- Messages must be copied and pasted between channels
- No multi-device support (DB must be manually shared)
- Not yet ready for production use

# Roadmap

- [x] CLI prototype
- [x] Core encryption and key exchange
- [x] Encrypted message send/receive
- [x] Local session storage
- [x] Message DB storage
- [x] Rus localisation
- [x] CLI chat app, demo release
- [ ] GUI chat app with Slint
- [ ] Obfuscation mode
- [ ] Packaging and release builds

# License

This project is licensed under the **GNU General Public License v3.0**. See the [LICENSE](LICENSE) file for details.

You are free to use, modify, and distribute this software under the terms of the GPL 3.0. For more information, visit https://www.gnu.org/licenses/gpl-3.0.html

# Contributing

Contributions are welcome! Please submit issues and pull requests on [Codeberg](https://codeberg.org/bazelik-dev/OpenE2E).

# Disclaimer

This software is provided as-is. While it implements industry-standard encryption, users are responsible for verifying key authenticity and following security best practices.


Copyright (C) 2026 bazelik-dev
