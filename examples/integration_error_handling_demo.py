#!/usr/bin/env python3
"""
RealGibber Integration and Error Handling Demo

This example demonstrates robust integration patterns and comprehensive error handling
for RealGibber systems in production environments.

Features demonstrated:
- Graceful error handling and recovery
- Integration with existing systems
- Fault tolerance and redundancy
- Monitoring and alerting
- Configuration management
"""

import sys
import time
import json
import logging
from typing import Dict, List, Optional, Any
from dataclasses import dataclass
from enum import Enum
import traceback

# RealGibber imports
from gibberlink_core import (
    RgibberLink, CryptoEngine, AuditSystem, AuditEntry,
    WeatherManager, WeatherData, GeoCoordinate,
    DroneSpecifications, MissionPayload
)


class ErrorSeverity(Enum):
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class IntegrationError(Exception):
    """Custom exception for integration errors."""
    def __init__(self, message: str, severity: ErrorSeverity = ErrorSeverity.MEDIUM, component: str = "unknown"):
        super().__init__(message)
        self.severity = severity
        self.component = component
        self.timestamp = time.time()


@dataclass
class SystemHealth:
    """System health status."""
    component: str
    status: str
    last_check: float
    error_count: int
    last_error: Optional[str] = None


class ErrorHandler:
    """Centralized error handling and recovery system."""

    def __init__(self):
        self.errors: List[IntegrationError] = []
        self.recovery_actions: Dict[str, callable] = {}
        self.logger = logging.getLogger("RealGibber.ErrorHandler")

    def register_recovery_action(self, error_type: str, action: callable):
        """Register a recovery action for a specific error type."""
        self.recovery_actions[error_type] = action

    def handle_error(self, error: Exception, component: str) -> bool:
        """Handle an error with appropriate severity and recovery."""
        integration_error = IntegrationError(
            str(error),
            severity=self._classify_severity(error),
            component=component
        )

        self.errors.append(integration_error)
        self.logger.error(f"[{component}] {error.__class__.__name__}: {error}")

        # Attempt recovery
        error_type = error.__class__.__name__
        if error_type in self.recovery_actions:
            try:
                self.logger.info(f"Attempting recovery for {error_type}")
                return self.recovery_actions[error_type](error)
            except Exception as recovery_error:
                self.logger.error(f"Recovery failed: {recovery_error}")
                return False

        return False

    def _classify_severity(self, error: Exception) -> ErrorSeverity:
        """Classify error severity based on type."""
        if isinstance(error, (ConnectionError, TimeoutError)):
            return ErrorSeverity.HIGH
        elif isinstance(error, ValueError):
            return ErrorSeverity.MEDIUM
        elif isinstance(error, RuntimeError):
            return ErrorSeverity.CRITICAL
        else:
            return ErrorSeverity.LOW


class SystemMonitor:
    """System health monitoring and alerting."""

    def __init__(self):
        self.components: Dict[str, SystemHealth] = {}
        self.alert_thresholds = {
            "error_count": 5,
            "check_interval": 300  # 5 minutes
        }
        self.logger = logging.getLogger("RealGibber.Monitor")

    def register_component(self, component: str):
        """Register a component for monitoring."""
        self.components[component] = SystemHealth(
            component=component,
            status="unknown",
            last_check=0,
            error_count=0
        )

    def update_health(self, component: str, status: str, error: Optional[str] = None):
        """Update component health status."""
        if component not in self.components:
            self.register_component(component)

        health = self.components[component]
        health.status = status
        health.last_check = time.time()

        if error:
            health.error_count += 1
            health.last_error = error

        # Check for alerts
        if health.error_count >= self.alert_thresholds["error_count"]:
            self._trigger_alert(component, health)

    def _trigger_alert(self, component: str, health: SystemHealth):
        """Trigger an alert for unhealthy component."""
        self.logger.warning(f"ALERT: Component '{component}' has {health.error_count} errors")
        self.logger.warning(f"Last error: {health.last_error}")

    def get_health_report(self) -> Dict[str, Any]:
        """Generate comprehensive health report."""
        return {
            "timestamp": time.time(),
            "components": {
                name: {
                    "status": health.status,
                    "error_count": health.error_count,
                    "last_check": health.last_check,
                    "last_error": health.last_error
                }
                for name, health in self.components.items()
            },
            "overall_status": self._calculate_overall_status()
        }

    def _calculate_overall_status(self) -> str:
        """Calculate overall system status."""
        statuses = [health.status for health in self.components.values()]
        if "critical" in statuses:
            return "critical"
        elif "error" in statuses:
            return "degraded"
        elif all(s == "healthy" for s in statuses):
            return "healthy"
        else:
            return "warning"


class RealGibberIntegrator:
    """Main integration class with comprehensive error handling."""

    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.error_handler = ErrorHandler()
        self.monitor = SystemMonitor()
        self.logger = logging.getLogger("RealGibber.Integrator")

        # Initialize components
        self._initialize_components()

    def _initialize_components(self):
        """Initialize all system components with error handling."""
        components = [
            ("gibberlink", self._init_gibberlink),
            ("crypto", self._init_crypto),
            ("audit", self._init_audit),
            ("weather", self._init_weather)
        ]

        for name, init_func in components:
            self.monitor.register_component(name)
            try:
                init_func()
                self.monitor.update_health(name, "healthy")
                self.logger.info(f"Component '{name}' initialized successfully")
            except Exception as e:
                self.error_handler.handle_error(e, name)
                self.monitor.update_health(name, "error", str(e))

    def _init_gibberlink(self):
        """Initialize gibberlink with retry logic."""
        max_retries = 3
        for attempt in range(max_retries):
            try:
                self.gibberlink = RgibberLink()
                return
            except Exception as e:
                if attempt == max_retries - 1:
                    raise e
                time.sleep(1)
                self.logger.warning(f"Gibberlink init attempt {attempt + 1} failed, retrying...")

    def _init_crypto(self):
        """Initialize crypto engine."""
        self.crypto_engine = CryptoEngine()

    def _init_audit(self):
        """Initialize audit system."""
        self.audit_system = AuditSystem(self.config.get("audit_capacity", 1000))

    def _init_weather(self):
        """Initialize weather manager."""
        self.weather_manager = WeatherManager(self.config.get("weather_stations", 50))

    def execute_mission_transfer(self, mission_data: Dict[str, Any], drone_id: str) -> bool:
        """Execute mission transfer with comprehensive error handling."""
        self.monitor.register_component(f"mission_{drone_id}")

        try:
            # Step 1: Validate mission data
            self._validate_mission_data(mission_data)

            # Step 2: Assess weather conditions
            weather_ok = self._assess_weather_impact(mission_data)
            if not weather_ok:
                raise IntegrationError("Weather conditions unsafe for mission", ErrorSeverity.HIGH, "weather")

            # Step 3: Prepare secure payload
            payload = self._prepare_secure_payload(mission_data)

            # Step 4: Execute transfer
            success = self._perform_transfer(payload, drone_id)

            # Step 5: Audit the operation
            self._audit_operation("mission_transfer", success, drone_id)

            self.monitor.update_health(f"mission_{drone_id}", "healthy")
            return success

        except IntegrationError as e:
            self.error_handler.handle_error(e, f"mission_{drone_id}")
            self.monitor.update_health(f"mission_{drone_id}", "error", str(e))
            return False
        except Exception as e:
            integration_error = IntegrationError(f"Unexpected error: {e}", ErrorSeverity.CRITICAL, f"mission_{drone_id}")
            self.error_handler.handle_error(integration_error, f"mission_{drone_id}")
            self.monitor.update_health(f"mission_{drone_id}", "critical", str(e))
            return False

    def _validate_mission_data(self, mission_data: Dict[str, Any]):
        """Validate mission data structure."""
        required_fields = ["name", "waypoints", "constraints"]
        for field in required_fields:
            if field not in mission_data:
                raise ValueError(f"Missing required field: {field}")

    def _assess_weather_impact(self, mission_data: Dict[str, Any]) -> bool:
        """Assess weather impact on mission."""
        try:
            # Create weather data (would come from API in real implementation)
            weather_data = WeatherData(
                timestamp=time.time(),
                location=GeoCoordinate(45.5, -73.5, 100.0),
                temperature_celsius=20.0,
                humidity_percent=60.0,
                wind_speed_mps=5.0,
                wind_direction_degrees=180.0,
                gust_speed_mps=8.0,
                visibility_meters=10000.0,
                precipitation_rate_mmh=0.0,
                pressure_hpa=1013.0,
                cloud_cover_percent=30.0,
                lightning_probability=5.0
            )

            self.weather_manager.update_weather(weather_data)

            # Create mission and drone specs
            mission = MissionPayload(mission_data["name"], list(range(16)))
            drone_specs = DroneSpecifications(
                max_wind_speed_mps=10.0, max_speed_mps=15.0, abort_gust_threshold_mps=12.0,
                power_wind_coefficient=5.0, mass_kg=2.0, battery_capacity_wh=1500.0, sensor_count=4
            )

            # Assess impact
            impact = self.weather_manager.assess_weather_impact(mission, drone_specs)
            return impact.overall_risk_score < 0.7

        except Exception as e:
            raise IntegrationError(f"Weather assessment failed: {e}", ErrorSeverity.MEDIUM, "weather")

    def _prepare_secure_payload(self, mission_data: Dict[str, Any]) -> bytes:
        """Prepare encrypted mission payload."""
        try:
            json_data = json.dumps(mission_data).encode('utf-8')

            # Generate session key
            session_key = self.crypto_engine.generate_nonce()

            # Encrypt data
            encrypted = self.crypto_engine.encrypt_data(session_key, json_data)
            return encrypted

        except Exception as e:
            raise IntegrationError(f"Payload preparation failed: {e}", ErrorSeverity.HIGH, "crypto")

    def _perform_transfer(self, payload: bytes, drone_id: str) -> bool:
        """Perform the actual secure transfer."""
        # Simulate transfer process
        time.sleep(0.5)  # Simulate processing time

        # In real implementation, this would:
        # 1. Generate QR code with payload
        # 2. Transmit ultrasonic synchronization
        # 3. Wait for drone acknowledgment

        return True  # Simulate success

    def _audit_operation(self, operation: str, success: bool, drone_id: str):
        """Audit the operation."""
        try:
            entry = AuditEntry(
                event_type="MissionTransfer",
                severity="High" if success else "Critical",
                actor="Integrator",
                operation=operation,
                success=success
            )

            audit_id = self.audit_system.record_event(entry)
            self.logger.info(f"Audit recorded: {audit_id} for {drone_id}")

        except Exception as e:
            self.logger.warning(f"Audit failed: {e}")


def main():
    """Main demonstration function."""
    print("🔧 RealGibber Integration and Error Handling Demo")
    print("=" * 60)

    # Configure logging
    logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')

    # System configuration
    config = {
        "audit_capacity": 1000,
        "weather_stations": 50,
        "retry_attempts": 3,
        "timeout_seconds": 30
    }

    # Initialize integrator
    integrator = RealGibberIntegrator(config)

    # Sample mission data
    mission_data = {
        "name": "Integration Test Mission",
        "waypoints": [[45.5, -73.5], [45.6, -73.6], [45.7, -73.7]],
        "constraints": {
            "max_altitude": 100.0,
            "max_speed": 10.0,
            "safety_margin": 0.1
        }
    }

    # Execute mission transfers
    drones = ["DRONE-001", "DRONE-002", "DRONE-003"]

    results = []
    for drone_id in drones:
        print(f"\n🚁 Executing mission transfer for {drone_id}...")
        success = integrator.execute_mission_transfer(mission_data, drone_id)
        results.append((drone_id, success))

        # Brief pause between transfers
        time.sleep(1)

    # Generate health report
    print("\n📊 System Health Report:")
    health_report = integrator.monitor.get_health_report()

    print(f"Overall Status: {health_report['overall_status']}")
    for component, status in health_report['components'].items():
        print(f"  {component}: {status['status']} (errors: {status['error_count']})")

    # Results summary
    print("\n🎯 Mission Transfer Results:")
    successful = sum(1 for _, success in results if success)
    print(f"Successful transfers: {successful}/{len(results)}")

    if successful == len(results):
        print("✅ All mission transfers completed successfully!")
    else:
        print("⚠️ Some transfers failed. Check error logs for details.")

    # Error summary
    error_count = len(integrator.error_handler.errors)
    if error_count > 0:
        print(f"\n⚠️ Total errors encountered: {error_count}")
        severity_counts = {}
        for error in integrator.error_handler.errors:
            severity_counts[error.severity.value] = severity_counts.get(error.severity.value, 0) + 1

        for severity, count in severity_counts.items():
            print(f"  {severity.upper()}: {count}")


if __name__ == "__main__":
    main()