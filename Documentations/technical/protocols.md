# RealGibber Communication Protocols

## Overview

RealGibber implements multiple secure communication protocols supporting both short-range pairing and long-range directional communication. All protocols feature cryptographic security, environmental adaptation, and comprehensive audit trails.

## Protocol Architecture

### Protocol Engine
**Location**: `rgibberlink-core/src/protocol.rs`
**Purpose**: Central state machine managing communication protocols and security handshakes

### Communication Modes

| Mode | Range | Channels | Handshake Time | Security |
|------|-------|----------|----------------|----------|
| **Short-Range** | 0-5m | Audio + Visual | 100-300ms | ECDH + AES-GCM |
| **Long-Range** | 10-200m | Ultrasound + Laser | 200-500ms | Coupled Channels |
| **Auto** | Adaptive | All available | Variable | Mode-dependent |

## Short-Range Protocol (0-5m)

### Protocol Overview
Traditional pairing protocol using ultrasonic audio synchronization and visual QR codes for key exchange.

### Handshake Sequence

```
Device A (Sender)              Device B (Receiver)
    │                               │
    │ 1. Generate nonce             │
    │    + timestamp                │
    │                               │
    ├───► Send HandshakeMessage ───►│
    │    via ultrasonic (18-22kHz)  │
    │                               │
    │                               │ 2. Generate keypair
    │                               │    + session ID
    │                               │
    │                               │ 3. Create QrPayload
    │                               │    + sign nonce
    │                               │
    │                               ├───► Display QR code
    │                               │    (Reed-Solomon ECC)
    │                               │
    │ 4. Scan QR code               │
    ◄─── (camera input) ◄────────────┤
    │                               │
    │ 5. Verify signature           │
    │    + derive shared secret     │
    │    + establish session key    │
    │                               │
    ├───► Send AckMessage ─────────►│
    │    via ultrasonic             │
    │                               │
    │                               │ 6. Receive ACK
    │                               │
    └───► Secure channel established ◄──┘
```

### Message Formats

#### HandshakeMessage
```rust
struct HandshakeMessage {
    nonce: [u8; 32],           // Random nonce for replay prevention
    timestamp: u64,             // Unix timestamp in seconds
    protocol_version: u16,      // Protocol version (currently 1)
}
```

#### QrPayload
```rust
struct QrPayload {
    public_key: [u8; 65],       // Compressed EC P-256 public key
    session_id: String,         // Unique session identifier
    nonce: [u8; 32],           // Echo of sender's nonce
    signature: [u8; 64],       // ECDSA signature over nonce + session_id
}
```

#### AckMessage
```rust
struct AckMessage {
    session_id: String,         // Session ID for verification
    timestamp: u64,             // Unix timestamp in seconds
}
```

### State Machine

```
Idle ─────────────► WaitingForQr ───────► Connected ───────► SecureChannelEstablished
    │                       │                      │
    │                       │                      │
    └─ invalid transition ──┴─ process_qr_payload ┴─ send_ack
                            │
                            └─ receive_ack (from receiver side)
```

## Long-Range Protocol (10-200m)

### Protocol Overview
Advanced directional communication using coupled ultrasound and laser channels. Requires simultaneous control of both channels within strict timing windows for security.

### Handshake Sequence

```
Device A (Sender)              Device B (Receiver)
    │                               │
    │ 1. Generate sync nonce        │
    │    + ephemeral keypair        │
    │    + session ID                │
    │                               │
    ├───► Send LongRangeHandshakeMessage │
    │    via focused ultrasound      │
    │    (40kHz carrier, parametric) │
    │                               │
    ├───► Send data payload ────────►│
    │    via laser simultaneously    │
    │    (OOK/PWM/QR modulation)     │
    │                               │
    │                               │ 2. Detect ultrasound pulse
    │                               │    within ±100ms window
    │                               │
    │                               │ 3. Receive laser data
    │                               │    + validate correlation
    │                               │
    │                               │ 4. Verify signatures
    │                               │    + derive shared secret
    │                               │
    │                               ├───► Send response
    │                               │    via laser
    │                               │
    │ 5. Receive response           │
    ◄─── via laser ◄─────────────────┤
    │                               │
    └───► Coupled channel secured ◄──┘
```

### Coupled Channel Validation

#### Temporal Correlation
- **Validation Window**: ±100ms around ultrasound sync pulse
- **Purpose**: Prevents replay attacks and ensures simultaneity
- **Implementation**: Timestamp validation with configurable tolerance

#### Cross-Channel Authentication
- **Ultrasound Channel**: Carries authentication data and signatures
- **Laser Channel**: Carries encrypted payload data
- **Mutual Verification**: Each channel authenticates the other's data

### Message Formats

#### LongRangeHandshakeMessage
```rust
struct LongRangeHandshakeMessage {
    sync_nonce: [u8; 32],           // Synchronization nonce
    timestamp: u64,                  // Unix timestamp in seconds
    public_key: [u8; 65],            // Sender's public key
    session_id: String,              // Unique session identifier
    laser_data_hash: [u8; 32],       // SHA256 hash of laser payload
    ultrasound_signature: [u8; 64],  // Signature over laser_data_hash
}
```

### Modulation Schemes

#### Laser Modulation Options
- **OOK (On-Off Keying)**: Simple binary modulation, high bandwidth
- **PWM (Pulse Width Modulation)**: Variable pulse width for error correction
- **QR Projection**: Structured data encoding with built-in ECC
- **Frequency Modulation**: Multi-level encoding for dense environments

#### Ultrasound Modulation
- **Parametric Transducers**: 40kHz carrier frequency
- **Focused Beam**: Highly directional, difficult to intercept
- **Range**: 10-30 meters with line-of-sight

## Protocol State Management

### Core States
```rust
enum ProtocolState {
    Idle,                          // Initial state, no active session
    WaitingForQr,                  // Short-range: waiting for QR scan
    QrScanned,                     // Short-range: QR scanned, waiting for ACK
    Connected,                     // Keys exchanged, session established
    SecureChannelEstablished,      // Short-range secure channel active
    LongRangeSecureChannel,        // Long-range coupled channel active
    Error,                         // Error state requiring reset
}
```

### State Transitions

#### Short-Range Transitions
```
Idle → initiate_handshake() → WaitingForQr
WaitingForQr → process_qr_payload() → Connected
Connected → receive_ack() → SecureChannelEstablished
```

#### Long-Range Transitions
```
Idle → initiate_long_range_handshake() → LongRangeSecureChannel
```

### State Validation
- **Invalid Transitions**: Automatically rejected with `ProtocolError::InvalidStateTransition`
- **Timeout Handling**: Automatic state reset after configurable timeout periods
- **Error Recovery**: State machine reset to `Idle` on critical errors

## Data Transmission Protocols

### Encrypted Payload Format
```rust
struct EncryptedPayload {
    iv: [u8; 12],              // AES-GCM initialization vector
    ciphertext: Vec<u8>,       // Encrypted data
    tag: [u8; 16],            // Authentication tag
    sequence_number: u32,     // Anti-replay sequence number
}
```

### Sequence Number Management
- **Per-Session**: Sequence numbers reset for each new session
- **Monotonic**: Strictly increasing to prevent replay attacks
- **Window Validation**: Configurable window size for out-of-order tolerance

## Error Correction and Reliability

### Reed-Solomon ECC
- **Short-Range**: RS(255,223) for QR codes (32 error correction bytes)
- **Long-Range**: Adaptive RS(n,k) based on channel conditions
- **Weather Adaptation**: Increased ECC strength in poor visibility

### Convolutional Coding
- **Rate**: Variable rate coding (1/2, 2/3, 3/4)
- **Constraint Length**: K=7 for optimal performance
- **Interleaving**: Block interleaving for burst error correction

### Automatic Repeat Request (ARQ)
- **Selective Repeat**: Efficient retransmission of corrupted blocks
- **Window Size**: Adaptive based on channel quality
- **Timeout**: Exponential backoff for retransmission

## Environmental Adaptation

### Weather Impact Assessment
The protocol engine continuously monitors environmental conditions:

```rust
// 6-factor weather assessment
struct WeatherImpact {
    wind_speed_ms: f32,
    precipitation_mm_per_hour: f32,
    visibility_meters: f32,
    temperature_celsius: f32,
    microclimate_factors: f32,
    em_interference_level: f32,
    overall_risk_score: f32,  // 0.0-1.0
}
```

### Adaptive Parameters

| Condition | ECC Strength | Validation Window | Key Rotation |
|-----------|--------------|-------------------|--------------|
| Low Risk  | RS(255,223) | ±500ms | 24 hours |
| Medium Risk| RS(255,191) | ±200ms | 12 hours |
| High Risk | RS(255,127) | ±100ms | 6 hours |
| Critical  | RS(255,63)  | ±50ms  | 1 hour |

## Fallback Mechanisms

### Automatic Mode Switching
- **Trigger Conditions**: Channel quality drops below threshold
- **Fallback Priority**: Long-range → Short-range → Error
- **Recovery Monitoring**: Automatic restoration when conditions improve

### Channel Health Assessment
```rust
struct ChannelHealth {
    signal_strength: f32,          // 0.0-1.0
    error_rate: f32,              // Bit error rate
    latency_ms: u32,              // Round-trip time
    correlation_coefficient: f32, // Channel coupling quality
    overall_health: f32,          // Composite health score
}
```

## Security Protocols

### Key Exchange Protocol
1. **Ephemeral Key Generation**: ECDH keypair generation
2. **Public Key Transmission**: Via QR code or laser modulation
3. **Shared Secret Derivation**: HKDF with salt and info parameters
4. **Session Key Generation**: HKDF-Expand for AES-GCM keys

### Authentication Protocol
1. **Nonce Generation**: Cryptographically secure random nonces
2. **Signature Creation**: ECDSA signatures over critical data
3. **Verification**: Signature validation before key derivation
4. **Session Binding**: Nonce binding prevents replay attacks

### Encryption Protocol
- **Algorithm**: AES-GCM-256
- **Key Derivation**: HKDF-SHA256
- **IV Generation**: Unique IV per message
- **Authentication**: Integrated HMAC-SHA256

## Audit and Compliance

### Protocol Audit Events
```rust
enum AuditEventType {
    ProtocolInitiated,
    HandshakeCompleted,
    KeyExchangeSuccessful,
    ChannelEstablished,
    MessageTransmitted,
    ErrorOccurred,
    SecurityViolation,
}
```

### Audit Trail Structure
- **Timestamp**: High-precision system time
- **Actor**: Device fingerprint and session ID
- **Operation**: Specific protocol operation
- **Result**: Success/failure with error details
- **Evidence**: Cryptographic proofs and signatures

## Performance Optimization

### Latency Optimization
- **Handshake Pipelining**: Parallel key generation and transmission
- **Hardware Acceleration**: NEON SIMD for cryptographic operations
- **Caching**: Session parameter caching for repeated connections

### Bandwidth Optimization
- **Compression**: CBOR encoding for structured data
- **Adaptive Modulation**: Rate adaptation based on channel quality
- **Packet Aggregation**: Multiple messages in single transmission

### Power Optimization
- **Duty Cycling**: Adaptive transmission power based on range
- **Beam Steering**: Minimal power for target alignment
- **Environmental Adaptation**: Power scaling based on conditions

## Protocol Extensions

### Future Enhancements
- **Multi-Device Coordination**: Mesh networking protocols
- **Quantum-Resistant Crypto**: Post-quantum key exchange algorithms
- **Satellite Backup**: Long-range fallback via satellite links
- **AI-Driven Adaptation**: Machine learning optimization

### Backward Compatibility
- **Version Negotiation**: Protocol version exchange in handshake
- **Graceful Degradation**: Support for older protocol versions
- **Migration Paths**: Seamless upgrades with compatibility layers

## Implementation Details

### Rust Implementation
- **Async/Await**: Tokio-based asynchronous operations
- **Error Handling**: Comprehensive error types with `thiserror`
- **Serialization**: Serde for message encoding/decoding
- **Cryptography**: `ring` crate for cryptographic primitives

### Hardware Integration
- **JNI Interface**: Android NDK integration for mobile devices
- **WebAssembly**: Browser-based protocol execution
- **Python Bindings**: PyO3 for scripting integration

### Testing and Validation
- **Unit Tests**: Individual protocol component validation
- **Integration Tests**: End-to-end handshake verification
- **Security Tests**: Penetration testing and cryptographic validation
- **Performance Benchmarks**: Latency and throughput measurements