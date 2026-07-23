# End-to-End Encryption with OLM

**Languages:** [English](OLM.md) | [Русский](OLM.ru.md)

# What is OLM?

OLM is a cryptographic protocol for secure communication over an untrusted network. It combines Curve25519 key agreement with the Double Ratchet algorithm to provide end-to-end encryption and perfect forward secrecy.

In practice, this means messages are encrypted locally, old message keys are discarded, and future messages stay protected even if part of the session is compromised.

# Handshake

### Key Material

Before messaging begins, each side prepares key material for establishing a session:

- **Identity Key** - A long-lived key that identifies the user during a session.
- **One-Time Key** - A single-use key used during initial session setup.

### Establishing a Session

**If you initiate the session:**
1. You receive the other party's key bundle
2. You perform ECDH key agreement using your private keys and their public keys
3. This produces a shared secret that only both sides can derive
4. You create the first encrypted message, which carries the extra key material needed to synchronize the session
5. You send that message to the other party

**If you receive the first message:**
1. You receive the other party's key bundle
2. You receive their initial encrypted message
3. You extract the key material from the message
4. You perform the same key agreement steps
5. You derive the same shared secret and synchronize the session state

After this, both sides have a shared encrypted session.

# The Double Ratchet

After the handshake, messages do not use static keys. Instead, the session advances with a ratchet that derives fresh keys for every message.

Each message uses a unique key derived from the current session state. After encryption or decryption, the session advances and old message keys are discarded, reducing the impact of key compromise.

# Message Types

The protocol uses two main message formats:

- **PreKeyMessage** - Used for the first message in a session and includes initial key material plus encrypted content.
- **Message** - Used for all later messages and contains only encrypted content and ratchet data.

# Base64 Encoding

Keys and encrypted messages are binary data. To make them easy to copy and paste through text channels, they are encoded in Base64. The receiver decodes them back into binary before use.

# Session Management

The app manages multiple independent encrypted sessions, one per contact. Each contact has its own ratchet state, so conversations stay isolated from each other.

You select the contact, then encrypt or decrypt messages for that session. The app handles ratchet advancement automatically.

# Complete Handshake Example

1. Party A generates key material and shares it
2. Party B receives it and prepares their own keys
3. Party B shares their key bundle back
4. Party A establishes the session and sends the first encrypted message
5. Party B receives the bundle and the message, synchronizes the session, and decrypts it
6. Both sides now share the same session state
7. Each new message advances the ratchet independently on both sides

# Security Guarantees

- **Confidentiality** - Messages are encrypted so only the intended recipient can read them
- **Authenticity** - Tampering with messages is detected
- **Forward Secrecy** - Past messages remain protected even if current keys are compromised
- **Break-in Recovery** - If a session is compromised, future messages remain protected once the ratchet advances
- **Replay Resistance** - Ratchet state limits reuse of old message material


Copyright (C) 2026 bazelik-dev
