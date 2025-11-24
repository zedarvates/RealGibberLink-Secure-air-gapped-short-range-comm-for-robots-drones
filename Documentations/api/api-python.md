# Python API Documentation

## Overview

The Python bindings for RgibberLink Core provide a high-level interface to the secure communication library, enabling Python applications to leverage the full range of cryptographic, audio, visual, and protocol functionality.

### Installation

```bash
pip install gibberlink-core
```

Or build from source:
```bash
pip install -e ./rgibberlink-core
```

## Core Classes

### `CryptoEngine`

High-level cryptographic operations wrapper.

#### Initialization
```python
from gibberlink_core import CryptoEngine

crypto = CryptoEngine()
```

#### Methods

##### `derive_shared_secret(peer_public_key: bytes) -> bytes`
Derives a shared secret using ECDH key exchange.

**Parameters:**
- `peer_public_key`: 32-byte public key from peer device

**Returns:** 32-byte shared secret

**Example:**
```python
sender_keypair = CryptoEngine()
receiver_keypair = CryptoEngine()

# Get public keys (implementation detail)
sender_public = sender_keypair.public_key()
receiver_public = receiver_keypair.public_key()

# Derive shared secret
shared_secret = receiver_keypair.derive_shared_secret(sender_public)
```

##### `encrypt_data(key: bytes, data: bytes) -> bytes`
Encrypts data using AES-GCM.

**Parameters:**
- `key`: 32-byte encryption key
- `data`: Data to encrypt

**Returns:** Encrypted data with authentication tag

##### `decrypt_data(key: bytes, encrypted_data: bytes) -> bytes`
Decrypts AES-GCM encrypted data.

##### `generate_device_fingerprint(device_info: bytes) -> bytes`
Creates a unique device fingerprint.

##### `compute_hmac(key: bytes, data: bytes) -> bytes`
Computes HMAC-SHA256.

##### `verify_hmac(key: bytes, data: bytes, expected_hmac: bytes) -> None`
Verifies HMAC signature, raises exception on failure.

##### `sign_log_entry(log_data: bytes) -> bytes`
Signs log data with Ed25519.

##### `verify_log_signature(public_key: bytes, log_data: bytes, signature: bytes) -> None`
Verifies Ed25519 signature.

### `RgibberLink`

Main session manager for secure communication.

#### Initialization
```python
from gibberlink_core import RgibberLink

link = RgibberLink()
```

#### Methods

##### `initiate_handshake() -> Coroutine`
Initiates the handshake protocol as the sender.

##### `receive_nonce(nonce: bytes) -> Coroutine`
Receives nonce and prepares QR code as receiver.

##### `process_qr_payload(qr_data: bytes) -> Coroutine`
Processes scanned QR code payload.

##### `receive_ack() -> Coroutine`
Receives acknowledgment from sender.

##### `get_state() -> str`
Returns current protocol state.

##### `encrypt_message(data: bytes) -> bytes`
Encrypts message using session key.

##### `decrypt_message(encrypted_data: bytes) -> bytes`
Decrypts message using session key.

##### Messaging API Methods

##### `send_text_message(content: str) -> str`
Sends a text message.

##### `request_authorization(permissions: List[str]) -> str`
Requests authorization for specific permissions.

##### `respond_to_authorization(request_id: str, granted: bool, reason: Optional[str]) -> str`
Responds to authorization request.

##### `send_status_update(status: str, details: str) -> str`
Sends status update.

##### `send_command(command: str, parameters: Dict[str, str]) -> str`
Sends command with parameters.

##### `send_notification(title: str, body: str) -> str`
Sends notification.

##### `get_pending_messages() -> List[Message]`
Gets pending messages for processing.

##### `process_incoming_message(encrypted_data: bytes) -> None`
Processes incoming encrypted message.

### `WeatherManager`

Manages weather data and mission validation.

#### Initialization
```python
from gibberlink_core import WeatherManager

weather_mgr = WeatherManager(max_history=100)
```

#### Methods

##### `update_weather(weather_data: WeatherData) -> None`
Updates weather database with new data.

##### `assess_weather_impact(mission: MissionPayload, drone_specs: DroneSpecifications) -> WeatherImpact`
Assesses weather impact on mission.

##### `validate_mission_constraints(mission: MissionPayload, drone_specs: DroneSpecifications) -> ConstraintValidationResult`
Validates mission against weather constraints.

### `WeatherData`

Weather observation data structure.

#### Initialization
```python
from gibberlink_core import WeatherData, GeoCoordinate

location = GeoCoordinate(
    latitude=45.5017,
    longitude=-73.5673,
    altitude_msl=100.0
)

weather = WeatherData(
    timestamp=time.time(),
    location=location,
    temperature_celsius=15.2,
    humidity_percent=65.0,
    pressure_hpa=1012.0,
    wind_speed_mps=3.5,
    wind_direction_degrees=270.0,
    visibility_meters=8500.0,
    precipitation_mm_per_hour=0.0,
    precipitation_type="none",
    cloud_cover_percent=35.0,
    uv_index=3.0,
    source="metar"
)
```

### `GeoCoordinate`

Geographic coordinate representation.

#### Initialization
```python
coord = GeoCoordinate(
    latitude=45.5017,      # Degrees
    longitude=-73.5673,    # Degrees
    altitude_msl=100.0     # Meters above mean sea level
)
```

### `DroneSpecifications`

Drone capability specifications for weather validation.

#### Initialization
```python
specs = DroneSpecifications(
    max_wind_speed_mps=10.0,        # Maximum safe wind speed
    max_speed_mps=15.0,             # Maximum flight speed
    abort_gust_threshold_mps=15.0,  # Gust speed that triggers abort
    power_wind_coefficient=5.0,     # Watts per m/s wind
    mass_kg=2.5,                    # Drone mass
    battery_capacity_wh=200.0,      # Battery capacity
    sensor_count=4                  # Number of sensors
)
```

### `MissionPayload`

Mission data structure with header, constraints, and tasks.

#### Initialization
```python
from gibberlink_core import MissionPayload

mission_id = list(range(1, 17))  # 16-byte ID
mission = MissionPayload("Urban Patrol Mission", mission_id)
```

#### Properties
- `header`: MissionHeader with metadata
- `constraints`: MissionConstraints
- `tasks`: List of MissionTask
- `crypto`: MissionCrypto configuration

### `AuditSystem`

Comprehensive audit trail system.

#### Initialization
```python
from gibberlink_core import AuditSystem

audit = AuditSystem(max_entries=1000)
```

#### Methods

##### `record_event(event: AuditEntry) -> str`
Records an audit event.

##### `get_active_alerts() -> List[SecurityAlert]`
Retrieves active security alerts.

### `AuditEntry`

Audit event data structure.

#### Initialization
```python
from gibberlink_core import AuditEntry

event = AuditEntry(
    event_type="MissionTransfer",    # handshake, message, mission, security, system
    severity="High",                 # critical, high, medium, low
    actor="Operator",                # human, drone, station, system
    operation="transfer_mission",    # connect, disconnect, authenticate, etc.
    success=True
)
```

## Example Usage

### Complete Mission Transfer Workflow

```python
import time
from gibberlink_core import (
    WeatherManager, WeatherData, GeoCoordinate, DroneSpecifications,
    MissionPayload, AuditSystem, AuditEntry, RgibberLink
)

def execute_mission_transfer():
    # Initialize systems
    weather_mgr = WeatherManager(100)
    audit_sys = AuditSystem(1000)
    link = RgibberLink()

    # Update weather data
    location = GeoCoordinate(45.5017, -73.5673, 100.0)
    weather = WeatherData(
        timestamp=time.time(),
        location=location,
        temperature_celsius=15.2,
        humidity_percent=65.0,
        wind_speed_mps=3.5,
        wind_direction_degrees=270.0,
        visibility_meters=8500.0,
        precipitation_mm_per_hour=0.0,
        precipitation_type="none",
        cloud_cover_percent=35.0,
        uv_index=3.0,
        source="metar"
    )
    weather_mgr.update_weather(weather)

    # Create drone specifications
    drone_specs = DroneSpecifications(
        max_wind_speed_mps=10.0,
        max_speed_mps=15.0,
        abort_gust_threshold_mps=15.0,
        power_wind_coefficient=5.0,
        mass_kg=2.5,
        battery_capacity_wh=200.0,
        sensor_count=4
    )

    # Create mission
    mission_id = list(range(1, 17))
    mission = MissionPayload("Urban Patrol Mission", mission_id)

    # Assess weather impact
    weather_impact = weather_mgr.assess_weather_impact(mission, drone_specs)
    if weather_impact.overall_risk_score > 0.7:
        raise ValueError("Mission unsafe due to weather")

    # Validate constraints
    validation = weather_mgr.validate_mission_constraints(mission, drone_specs)
    if not validation.is_valid:
        for violation in validation.violations:
            print(f"Violation: {violation.description}")

    # Record audit event
    audit_event = AuditEntry(
        event_type="MissionTransfer",
        severity="High",
        actor="Operator",
        operation="prepare_mission",
        success=True
    )
    audit_sys.record_event(audit_event)

    # Execute secure transfer (simplified)
    # link.initiate_handshake()
    # ... handshake protocol ...

    return True
```

### Weather-Aware Mission Planning

```python
class DroneMissionController:
    def __init__(self):
        self.weather_mgr = WeatherManager(100)
        self.audit_sys = AuditSystem(1000)

    def prepare_mission(self, mission_data, drone_specs):
        # Assess weather impact
        mission = MissionPayload(mission_data['name'], mission_data['id'])
        impact = self.weather_mgr.assess_weather_impact(mission, drone_specs)

        if impact.overall_risk_score > 0.7:
            raise ValueError("Mission unsafe due to weather conditions")

        # Log preparation
        event = AuditEntry(
            event_type="MissionTransfer",
            severity="Medium",
            actor="Operator",
            operation="weather_validation",
            success=True
        )
        self.audit_sys.record_event(event)

        return mission, impact
```

## Error Handling

All methods that can fail raise Python exceptions with descriptive messages:

```python
try:
    encrypted = crypto.encrypt_data(key, data)
except ValueError as e:
    print(f"Encryption failed: {e}")
```

## Platform-Specific Considerations

### Async/Await Support

All protocol operations are async and return coroutines:

```python
import asyncio

async def handshake():
    link = RgibberLink()
    await link.initiate_handshake()
    # ... continue protocol

asyncio.run(handshake())
```

### Memory Management

Python bindings handle memory management automatically. Large data structures are reference-counted and cleaned up when no longer needed.

### Thread Safety

Most classes are not thread-safe. Use separate instances per thread or implement external synchronization.

## Integration Patterns

### Drone Control Software Integration

```python
import gibberlink_core as gl

class DroneController:
    def __init__(self):
        self.weather = gl.WeatherManager(100)
        self.audit = gl.AuditSystem(1000)
        self.link = gl.RgibberLink()

    async def transfer_mission(self, mission_data, drone_id):
        # Weather validation
        weather_data = self._fetch_weather()
        self.weather.update_weather(weather_data)

        # Create mission object
        mission = gl.MissionPayload(mission_data['name'], mission_data['id'])

        # Validate safety
        specs = self._get_drone_specs(drone_id)
        validation = self.weather.validate_mission_constraints(mission, specs)

        if not validation.is_valid:
            raise RuntimeError("Mission constraints violated")

        # Execute transfer
        await self.link.initiate_handshake()
        # ... complete handshake ...

        # Audit success
        audit_event = gl.AuditEntry(
            event_type="MissionTransfer",
            severity="High",
            actor="Controller",
            operation="mission_transfer_complete",
            success=True
        )
        await self.audit.record_event(audit_event)

        return True
```

### Web API Integration

```python
from flask import Flask, request, jsonify
import gibberlink_core as gl

app = Flask(__name__)
weather_mgr = gl.WeatherManager(100)

@app.route('/weather/impact', methods=['POST'])
def assess_impact():
    data = request.json

    # Parse mission and drone data
    mission = gl.MissionPayload(data['mission']['name'], data['mission']['id'])
    drone_specs = gl.DroneSpecifications(**data['drone_specs'])

    # Assess impact
    impact = weather_mgr.assess_weather_impact(mission, drone_specs)

    return jsonify({
        'risk_score': impact.overall_risk_score,
        'recommendations': impact.recommended_actions
    })
```

This Python API provides a complete interface to the RgibberLink Core functionality, enabling secure, weather-aware drone mission transfers with comprehensive audit trails.