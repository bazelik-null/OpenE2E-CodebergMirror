<div align="center">
<h1> OpenE2E </h1>
<img src="doc/img/logo.png">
</div>

> [!WARNING]
>
> Under development. Not ready for use yet.
> Check [Roadmap](#roadmap).

# Overview

OpenE2E provides a simple, chat-like interface with **E2E encryption and Perfect Forward Secrecy (PFS)** based on **Matrix's** OLM, allowing you to communicate securely over insecure communication channels.

# Features

- **End-to-End Encryption** - All messages are encrypted with asymmetric cryptography and PFS
- **Easy Key Exchange** - No complex setup. Just generate ephemeral public keys, share via any channel, and establish a secure session
- **Chat-Like Interface** - Intuitive messaging UI built with **Slint**
- **Local Message Storage** - All messages are stored locally in readable chat format
- **Obfuscation Mode** - Optionally save messages as media files to avoid ML analysis and suspicious patterns
- **Works Over Untrusted Channels** - Send encrypted messages through government messengers, email, SMS, or any public platform
- **Built in Rust** - Memory safe and blazingly fast

# How It Works

1. **Create a Session** - Start a new conversation session in the app
2. **Exchange Keys** - Generate and copy your ephemeral public key, share it with your contact
3. **Receive Key** - Paste your contact's public key into the app to establish the secure session
4. **Send Messages** - Type your message, copy the encrypted output, and send it through any messenger
5. **Receive Messages** - Paste received encrypted messages into the app to decrypt and read them
6. **View History** - All decrypted messages are displayed in a readable chat interface

# Installation

### Build from Source

```bash
git clone https://codeberg.org/bazelik-dev/OpenE2E.git
cd OpenE2E
cargo build --release
./target/release/OpenE2E
```

### Pre-built binaries
**Work in progress.**

# Usage

### Basic Workflow

1. Launch the application
2. Click **"New Session"** to create a new encrypted conversation
3. Copy your **Public Key** and send it to your contact
4. Paste your contact's **Public Key** when prompted
5. Start typing messages in the chat input field
6. Copy encrypted messages and send via your preferred channel
7. Paste received encrypted messages to decrypt

### Obfuscation Mode

Enable obfuscation to store messages as media files instead of plaintext:
- Prevents automated analysis of message patterns
- Maintains plausible deniability
- Access messages only through the app

# Security Features

- **Perfect Forward Secrecy (PFS)** - OLM's Double Ratchet algorithm ensures past messages remain secure even if long-term keys are compromised
- **Asymmetric Encryption** - Public-key cryptography prevents eavesdropping
- **Local Storage Only** - No cloud sync, all data stays on your device
- **Untrusted Channel Resistant** - Messages can be sent through any public platform without additional setup

# Architecture

- **Frontend** - Slint UI framework for cross-platform GUI
- **Backend** - Rust cryptographic engine
- **Storage** - Local SQLite database for message history, keys, etc.

# Limitations

- Requires manual key exchange (no automatic key distribution)
- Messages must be manually copied and pasted between channels
- No multi-device support

# Roadmap

- [x] CLI prototype
- [ ] Core encryption and key exchange
- [ ] Encrypted message send/receive
- [ ] Local session storage
- [ ] Message DB storage
- [ ] CLI chat app, demo release
- [ ] GUI chat app with Slint
- [ ] Obfuscation mode
- [ ] Packaging and release builds

# License

This project is licensed under the **GNU Lesser General Public License v3.0**. See the [LICENSE](LICENSE) file for details.

You are free to use, modify, and distribute this software under the terms of the LGPL 3.0. For more information, visit https://www.gnu.org/licenses/lgpl-3.0.html

# Contributing

Contributions are welcome! Please submit issues and pull requests on [Codeberg](https://codeberg.org/bazelik-dev/OpenE2E).

# Disclaimer

This software is provided as-is. While it implements industry-standard encryption, users are responsible for verifying key authenticity and following security best practices.



OpenE2E  Copyright (C) 2026  bazelik-dev
