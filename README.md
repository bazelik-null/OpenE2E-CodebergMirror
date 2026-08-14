<div align="center">
<h1> OpenE2E </h1>
<img src="doc/img/logo.png">
</div>

> [!WARNING]
> Under active development. See the [Roadmap](#roadmap).

**Languages:** [English](README.md) | [Русский](doc/README.ru.md)

# Table of Contents

- [Overview](#overview)
- [What is This For?](#what-is-this-for)
- [Supported Platforms](#supported-platforms)
- [Installation](#installation)
- [How It Works](#how-it-works)
- [Architecture](#architecture)
- [Data Protection](#data-protection)
- [Features](#features)
- [FAQ](#faq)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Disclaimer and Terms of Use](#disclaimer-and-terms-of-use)

# Overview

**OpenE2E** is a manual secure chat app for exchanging encrypted messages over any channel - SMS, email, messengers, social platforms, or other untrusted channels. You encrypt messages locally on your device, copy the encrypted text, and send it anywhere. Only you and the recipient can decrypt it.

The app uses **Matrix OLM** (via `voldozemac`) with **AES-256-GCM encryption** and **Perfect Forward Secrecy (PFS)**, ensuring that past messages remain protected even if later keys are compromised. All data stays on your device - there is no cloud sync or server-side storage.

**Why this matters:** Your private thoughts, plans, and opinions shouldn't be scanned by algorithms, analyzed by companies, or monitored by surveillance systems. OpenE2E keeps your communication visible only to you and the recipient.

For technical details on the encryption model, see [OLM.md](doc/OLM.md).

# What is This For?

**OpenE2E** lets you send messages through _any_ channel (text, email, social media, and so on) so that only you and the recipient can read them. You encrypt messages on your device, copy the encrypted text, and paste it anywhere.

## Why Confidentiality Matters

**Data collection and analysis.** Companies and government agencies collect information about your location, contacts, browsing history, and online behavior. This data is used to build profiles, predict preferences, and target you with tailored content or messaging. In some cases, it's also used to identify and track "problematic" populations.

**Automated content analysis.** With advances in machine learning, tools have emerged that can automatically analyze, categorize, and filter vast volumes of communications. Your messages can be scanned for specific content, flagged by algorithms, or reported to authorities. While such systems are still imperfect and produce false positives, they are becoming increasingly accurate and widespread.

**What this means in practice.** Even if you're doing nothing illegal, your private thoughts, plans, and opinions can be misinterpreted by an algorithm, taken out of context, or used against you later, for example, when applying for a job, seeking a loan, or in legal proceedings.

**Why end-to-end encryption matters:** When your messages are encrypted, neither companies nor government systems can scan or analyze them. Your communication remains visible only to you and the recipient - beyond the reach of mass monitoring and algorithmic analysis.

# Supported Platforms

| Platform           | Status                   | Notes                                                            |
| ------------------ | ------------------------ | ---------------------------------------------------------------- |
| **Linux (64-bit)** | ✅ Fully supported       | Generic glibc-based systems (Debian, Ubuntu, Fedora, Arch, etc.) |
| **NixOS**          | ✅ Fully supported       | Packaged builds in releases                                      |
| **Windows 10/11**  | ✅ Fully supported       | 64-bit only                                                      |
| **Linux ARM**      | ⚠️ Requires manual build | Manual Rust build required                                       |
| **Termux**         | ⚠️ CLI only              | No GUI support. Manual Rust build required                       |
| **Android**        | 🔄 Planned               | Not yet available                                                |
| **MacOS**          | ❌ Untested              | May work, but unsupported and requires manual building.          |

# Installation

### Option 1: Pre-built Binaries

1. Go to [Releases](https://codeberg.org/bazelik-dev/OpenE2E/releases) and download the latest binary for your OS (choose GUI for graphical interface or CLI for console interface)
2. Verify the signature:
   ```bash
   gpg --auto-key-locate keyserver --keyserver-options auto-key-retrieve --verify OpenE2E*.asc $(find . -maxdepth 1 -name 'OpenE2E*' ! -name '*.asc')
   ```
   The key should match: `C4C5BDC6C5E4C96CF12B3E85B7BBEB3BC5439F72` from `bazelik-dev@proton.me`

### Option 2: Build with Cargo

```bash
git clone https://codeberg.org/bazelik-dev/OpenE2E.git
cd OpenE2E
cargo build --release
```

Add `--features ui` to include the Slint GUI:

```bash
cargo build --release --features ui
```

### Option 3: Parallel Build (All Cores)

Uses the build script to compile both CLI and GUI in parallel:

```bash
git clone https://codeberg.org/bazelik-dev/OpenE2E.git
cd OpenE2E
./build.sh
```

**Outputs:**

- CLI: `./OpenE2E-CLI_{linux,windows}/bin/OpenE2E`
- GUI: `./OpenE2E-GUI_{linux,windows}/bin/OpenE2E`

### Option 4: Nix Build

```bash
git clone https://codeberg.org/bazelik-dev/OpenE2E.git
cd OpenE2E
./build-nix.sh
```

**Outputs:**

- CLI: `./OpenE2E-CLI_NixOS/bin/OpenE2E`
- GUI: `./OpenE2E-GUI_NixOS/bin/OpenE2E`

Both binaries include all runtime dependencies in their `lib/` directories.

# How It Works

### Step-by-Step Process

1. **Create a Session** - Start a new conversation in the app
2. **Exchange Public Keys** - Generate your ephemeral public key and share it with your contact via any channel
3. **Receive Their Key** - Paste your contact's public key into the app to establish the session
4. **Encrypt & Copy** - Write your message, and the app encrypts it locally and gives you the ciphertext
5. **Send Ciphertext** - Copy and paste the encrypted message through email, SMS, social media, or any other channel
6. **Receive & Decrypt** - Paste the encrypted message your contact sends back into the app
7. **Read Locally** - The app decrypts it on your device and displays it in the chat view
8. **Continue** - Repeat for ongoing secure communication

**Key innovation:** you control the transport. OpenE2E handles encryption, you move the encrypted payloads.

# Architecture

OpenE2E follows a separation-of-concerns design with a clear object/service split inside the backend, and a frontend/backend boundary.

### Project Structure

```
src/
├── backend/                    # Core logic
│   ├── objects/                # Data models
│   │   ├── user.rs             # User account data
│   │   ├── session.rs          # Session state, keys, crypto
│   │   └── message.rs          # Message data structures
│   └── services/               # Services handle operations on objects
│       ├── repository.rs       # Container for storage service
│       ├── storage_service.rs  # Database operations
│       ├── user_service.rs     # User workflows (create/auth/etc.)
│       ├── session_service.rs  # Session workflows (init/handshake/etc.)
│       └── message_service.rs  # Message workflows (encrypt/save/etc.)
│
├── frontend/                  # UI and user interaction
│   ├── cli/                   # Command-line interface
│   ├── gui/                   # Graphical interface
│   │   └── slint/             # Slint UI layout
│   ├── fluent_service.rs      # Localization system
│   └── logger.rs              # Logging
│
├── error_mapper.rs            # Error conversion to string
├── main.rs                    # Entry point
└── tests/                     # Unit and integration tests
```

### Design Pattern

**Backend (Objects + Services):**

- **Objects** (`objects/user.rs`, `objects/session.rs`, `objects/message.rs`) represent the core data types and state used across the system.
- **Services** (`services/*_service.rs`) implement the backend workflows that operate on those objects.
  - `repository.rs` owns and manages the storage worker lifecycle (start/autosave/shutdown) and exposes a WorkerHandle.

**Frontend (CLI + GUI):**

- **CLI** provides a command-line interface
- **GUI** uses Slint for a native, lightweight interface
- Both frontends call the same backend managers, ensuring consistent behavior
- `fluent_manager.rs` handles localization

### Key Design Benefits

- **Modularity**: Backend workflow logic lives in `services/`, and frontend can change without touching core operations.
- **Maintainability**: Object types are centralized in `objects/`, while operations on them are centralized in `services/`.
- **Localization**: Text strings are centralized in `locales/` directory
- **Reuse across frontends**: CLI and GUI can use the same backend services for consistent behavior.

# Data Protection

- **Sessions & Accounts:** Encrypted in fjall DB with AES-256-GCM
- **Messages:** Encrypted at rest with AES-256-GCM and random nonce
- **Passwords:** Used only to derive encryption keys. They're never stored
- **Local Only:** No cloud sync, no server-side storage, no third-party access

# Features

- **Manual Secure Chat** - Exchange encrypted messages via copy-paste through any channel
- **Perfect Forward Secrecy** - Past messages stay protected even if keys are later compromised
- **End-to-End Encryption** - Messages encrypted locally. Only sender and recipient can read them
- **Works Over Any Channel** - SMS, email, messengers, social platforms, government services, etc.
- **Local Storage Only** - All data stays on your device
- **AES-256-GCM Encryption** - Industrial-grade encryption for messages and session storage
- **Dual Interface** - CLI for power users and Termux. GUI for general use
- **Multi-Language** - English and Russian localization
- **Rust-Based** - Memory-safe, blazingly fast, and secure

# FAQ

**Q: Why would I use this instead of Signal, WhatsApp, or Telegram?**

A: Those apps require dedicated apps on both ends and control the transport. OpenE2E lets you send encrypted messages through _any_ channel you already use - your existing email, SMS, social media, etc. You don't need both people to install anything special. It's useful when:

- Direct secure transport is unavailable
- You want to add encryption to existing communication channels
- You need a "just copy-paste" workflow
- You prefer to control the transport layer

**Q: Does OpenE2E support file attachments?**

A: Not yet. File support is on the [Roadmap](#roadmap).

**Q: Can I import/export my messages, sessions and accounts?**

A: Currently, messages are stored in the local encrypted database only. Storage import/export support is on the [Roadmap](#roadmap).

**Q: What happens if I forget my password?**

A: Your password is used to derive the encryption keys for your database. If you forget it, your messages and sessions cannot be recovered. There is no password reset or recovery mechanism.

**Q: Is my data safe if my device is compromised?**

A: Your messages are encrypted with AES-256-GCM, and the encryption keys are derived from your password using Argon2. If an attacker accesses your device stored messages remain encrypted and cannot be read without your password

However, if the attacker can access your device while you're logged in, they can read message contents in memory. This is a general limitation of any local-only system.

**Q: Can developers see my messages?**

A: No. OpenE2E runs entirely on your device. There are no servers, no accounts on external services, and no way for developers to access your data. The source code is open for you to verify this.

**Q: How do I verify the public key identity of my contact?**

A: OpenE2E does not automatically verify keys (it's pretty much impossible without a trusted channel). You should verify your contact's public key identity through a separate, trusted channel.

**Q: What if I lose my encrypted database file?**

A: Your encrypted database file (stored locally) contains all your messages and session keys. If you delete or lose it, those messages cannot be recovered. Back up your encrypted database file to a safe location if you want to preserve your message history.

# Roadmap

- [x] CLI prototype
- [x] Core encryption and key exchange
- [x] Encrypted message send/receive
- [x] Local session and database storage
- [x] Russian UI localization
- [x] CLI chat app demo
- [x] GUI chat app (Slint-based)
- [x] Packaging and release builds
- [] File Sending
- [] Data import/export
- [] Obfuscation mode (steganography for message hiding)
- [] Android GUI

# Acknowledgments

Some of the open source packages we use:

- **[voldozemac](https://github.com/poljar/voldozemac)** - OLM protocol implementation
- **[Slint](https://slint.rs/)** - Lightweight GUI framework
- **[fjall](https://github.com/fjall-rs/fjall)** - Embedded key-value database
- **[Rust](https://www.rust-lang.org/)** - Memory-safe systems programming language
- **Matrix Protocol** - Open standard for decentralized communication

# Contributing

Contributions are welcome! Please submit issues and pull requests on [Codeberg](https://codeberg.org/bazelik-dev/OpenE2E).

# License

This project is licensed under the **GNU General Public License v3.0**. See the [LICENSE](LICENSE) file for details.

You are free to use, modify, and distribute this software under the terms of the GPL 3.0. For more information, visit https://www.gnu.org/licenses/gpl-3.0.html

# Disclaimer and Terms of Use

OpenE2E is provided as-is for educational and personal use. While the encryption is sound, the application is still in active development. Always test thoroughly before relying on it for sensitive communications.

### Your Responsibility

Users are fully responsible for:

- Verifying that their use complies with the laws of their jurisdiction
- All legal and ethical consequences of using this tool
- Respecting the rights of others

### Limitations of Our Liability

The developer(s) of OpenE2E:

- Do not bear legal responsibility for the use of this software
- Do not control or monitor its application (the application runs locally on your device)
- Do not provide legal or technical support for unlawful activities
- Explicitly do not endorse use for criminal purposes

Before using this tool, ensure that your use complies with the laws of your jurisdiction.

<div align="center">

**[back to top](#table-of-contents)**

**Copyright (C) 2026 bazelik-dev**

</div>
```
