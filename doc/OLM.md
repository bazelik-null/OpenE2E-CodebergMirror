# End-to-End Encryption with OLM

**Languages:** [English](/doc/OLM.md) | [Русский](/doc/OLM.ru.md)

# What is OLM?

**OLM is a cryptographic protocol** that lets two people communicate securely over an untrusted network. It combines elliptic curve cryptography (Curve25519) with the Double Ratchet Algorithm to provide encryption that protects past messages even if current keys are stolen, and prevents forged future messages even if you're currently compromised.

# Handshake

### Key Generation

Before messaging, each party generates two keys:

- **Identity Key** - Permanent identifier for the user, alive entire session.
- **One-Time Key** - Single-use key for initial setup, alive only on handshake

Party A generates both keys, combines them as a string, and shares this bundle with Party B out-of-band (via terminal, email, etc.).

### Establishing a Session

**If you initiate (outbound session):**
1. You receive Party B's key bundle
2. You perform key agreement math (ECDH) between your private keys and their public keys
3. This produces a shared secret that only you and they can derive
4. You create a PreKeyMessage - your first encrypted message that includes extra key material so they can sync
5. You send this PreKeyMessage to them

**If they initiate and you receive first (inbound session):**
1. You receive Party A's key bundle
2. You receive their PreKeyMessage
3. You extract their identity and key material from the message
4. You perform the same key agreement math
5. You derive the identical shared secret

Both parties now have **synchronized session state**.


# The Double Ratchet

After the handshake, messages don't use static keys. Instead, a **ratchet mechanism** derives a fresh key for each message.

Each message uses a unique key derived from the current session state. After encryption, the session state is updated so the next message gets a different key. Old keys are deleted. They cannot be recovered even if the current session key leaks.


# Message Types

The protocol uses two message formats:

- **PreKeyMessage**: Sent only on first message by the initiator. Contains identity + key agreement material + encrypted content.
- **Message**: Sent for all subsequent messages. Contains just encrypted content + chain information. Much smaller.


# Base64 Encoding

Keys and encrypted messages are binary data. To send them as text (paste in terminal, copy-paste via chat), they're encoded in Base64—a text-safe representation using letters, numbers, and a few symbols. The receiver decodes Base64 back to binary before using the cryptographic material.


# Session Management

The code manages **multiple independent encrypted channels** with different contacts. Each contact gets their own session with its own ratchet state. You select which contact you're messaging, then encrypt/decrypt messages - the system handles the ratchet advancement automatically.


# Complete Handshake Example

**Party A wants to message Party B:**

1. Party A generates their keys and shares them
2. Party B copies Party A's keys and generates their own
3. Party B shares their keys
4. Party A copies Party B's keys and establishes an outbound session
5. Party A encrypts an initial message (PreKeyMessage) and displays it
6. Party B receives Party A's keys and the encrypted message, establishes an inbound session, and decrypts it
7. Both parties now have synchronized session state and can freely exchange encrypted messages
8. Each new message advances both ratchets independently in the same direction


# Security Guarantees

**Confidentiality**: Messages are encrypted with AES-256-GCM so only the intended recipient reads them.

**Authenticity**: Each message includes a cryptographic proof that it wasn't forged or altered.

**Forward Secrecy**: If your session key leaks today, attackers still can't read past messages because old keys are gone.

**Break-in Recovery**: If you're hacked mid-conversation, attackers can't forge your future messages because keys only move forward.

**Replay Protection**: The same message can't be decrypted twice; the ratchet prevents reuse.
