# RgibberLink Core Library

A comprehensive Rust library implementing both short-range and long-range secure directional communication protocols for autonomous drone operations.

## Overview

RgibberLink Core provides the foundational cryptographic and communication primitives for secure directional communication systems. The library supports multiple communication modes with weather-adaptive protocols, comprehensive audit trails, and mission-critical reliability.

## Architecture

### Core Components

- **Crypto Engine**: ECDH key exchange, AES-GCM encryption, HMAC verification
- **Audio Engine**: GGWave ultrasonic transmission for short-range pairing
- **Ultrasonic Beam Engine**: Focused ultrasound communication (10-30m range)
- **Visual Engine**: QR code generation with Reed-Solomon ECC
- **Laser Engine**: High-speed optical data transmission with adaptive ECC
- **Range Detector**: Ultrasonic time-of-flight ranging
- **Optical ECC**: Advanced error correction for laser transmission
- **Protocol Engine**: Handshake state machine with coupled validation
- **Security Manager**: Permission-based access control
- **Weather Integration**: Environmental constraint validation
- **Audit System**: Comprehensive compliance logging

### Communication Modes

#### Short-Range Mode (0-5m)
- Ultrasonic audio synchronization (18-22kHz FSK)
- QR code visual payload with CBOR compression
- ECDH key exchange + AES-GCM encryption
- Handshake time: 100-300ms

#### Long-Range Mode (10-200m)
- Coupled laser and ultrasound channels
- Temporal correlation validation (±100ms)
- Adaptive modulation (OOK/PWM/QR projection)
- Weather compensation and ECC adaptation

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rgibberlink-core = "0.3.0"
```

### Features

Enable optional features as needed:

```toml
[dependencies]
rgibberlink-core = { version = "0.3.0", features = ["python", "short-range"] }
```

Available features:
- `short-range`: QR code and ultrasonic support
- `python`: Python bindings via PyO3

## Quick Start

### Basic Usage

```rust
use rgibberlink_core::RgibberLink;

let mut link = RgibberLink::new();

// Initiate short-range handshake
link.initiate_handshake().await?;

// Send encrypted message
let encrypted = link.encrypt_message(b"Hello, World!").await?;
```

### Weather-Aware Mission Transfer

```rust
use rgibberlink_core::{mission::*, weather::*};

// Create mission with weather constraints
let mission = MissionPayload::default();

// Assess weather impact
let weather_manager = WeatherManager::new(100);
let impact = weather_manager.assess_weather_impact(&mission, &drone_specs)?;

// Validate constraints
let validation = weather_manager.validate_mission_constraints(&mission, &drone_specs)?;
```

## API Reference

### Core Types

- [`RgibberLink`](struct.RgibberLink.html): Main session manager
- [`MissionPayload`](mission/struct.MissionPayload.html): Complete mission structure
- [`WeatherManager`](weather/struct.WeatherManager.html): Environmental assessment
- [`AuditSystem`](audit/struct.AuditSystem.html): Compliance logging

### Communication Engines

- [`CryptoEngine`](crypto/struct.CryptoEngine.html): Cryptographic operations
- [`AudioEngine`](audio/struct.AudioEngine.html): Ultrasonic transmission
- [`LaserEngine`](laser/struct.LaserEngine.html): Optical communication
- [`UltrasonicBeamEngine`](ultrasonic_beam/struct.UltrasonicBeamEngine.html): Focused ultrasound

## Security Features

- **Perfect Forward Secrecy**: Ephemeral ECDH keys
- **Authenticated Encryption**: AES-GCM with integrity
- **Anti-Replay Protection**: Timestamp and nonce validation
- **Coupled Channel Security**: Requires simultaneous presence in both beams
- **Weather-Adaptive Security**: Environmental condition compensation

## Performance Characteristics

- **Handshake Time**: 100-300ms (short-range), 200-500ms (long-range)
- **Encryption**: <20ms for key exchange and message encryption
- **Weather Assessment**: <50ms for full environmental impact evaluation
- **Memory Usage**: ~50MB baseline + 2MB per active mission
- **Battery Optimized**: Minimal power consumption for mobile devices

## Testing

Run the test suite:

```bash
cargo test
```

Run with specific features:

```bash
cargo test --features python
```

## Examples

See the `examples/` directory for comprehensive usage examples:

- `drone_mission_transfer_demo.rs`: Complete mission transfer workflow
- `weather_integration_example.rs`: Environmental constraint validation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Safety & Ethics

This library implements secure communication protocols for autonomous systems. All cryptographic operations follow industry best practices and are designed for mission-critical applications.

---

**Built with Rust for maximum safety, performance, and reliability in critical autonomous operations.**