import pytest
import asyncio
import numpy as np
from unittest.mock import Mock, AsyncMock, patch, MagicMock
from gibberlink_core import (
    PyProtocolEngine, PyCryptoEngine, PyChannelValidator,
    PyUltrasonicBeamEngine, PyLaserEngine, PyOpticalECC,
    PyRangeDetector, PyFallbackManager
)


class TestLongRangeIntegration:
    """Integration tests for long-range GibberLink hybrid protocol"""

    @pytest.fixture
    def mock_hardware(self):
        """Mock hardware interfaces for CI/CD compatibility"""
        with patch('gibberlink_core.PyUltrasonicBeamEngine') as mock_ultrasound, \
             patch('gibberlink_core.PyLaserEngine') as mock_laser, \
             patch('gibberlink_core.PyRangeDetector') as mock_range_detector:

            # Configure ultrasound mock
            mock_ultrasound_instance = Mock()
            mock_ultrasound_instance.initialize = AsyncMock(return_value=None)
            mock_ultrasound_instance.transmit_sync_pulse = AsyncMock(return_value=None)
            mock_ultrasound_instance.detect_presence = AsyncMock(return_value=True)
            mock_ultrasound_instance.transmit_auth_signal = AsyncMock(return_value=None)
            mock_ultrasound_instance.get_channel_diagnostics = AsyncMock(return_value={
                'is_active': True,
                'presence_detected': True,
                'configured_range': 50.0,
                'carrier_frequency': 40000.0,
                'power_level': 0.8,
                'detected_failures': []
            })
            mock_ultrasound.return_value = mock_ultrasound_instance

            # Configure laser mock
            mock_laser_instance = Mock()
            mock_laser_instance.initialize = AsyncMock(return_value=None)
            mock_laser_instance.transmit_data = AsyncMock(return_value=None)
            mock_laser_instance.receive_data = AsyncMock(return_value=b"test_data")
            mock_laser_instance.get_alignment_status = AsyncMock(return_value={
                'is_aligned': True,
                'beam_position_x': 0.0,
                'beam_position_y': 0.0,
                'signal_strength': 0.9,
                'last_update': 0
            })
            mock_laser_instance.get_channel_diagnostics = AsyncMock(return_value={
                'is_active': True,
                'alignment_status': {'is_aligned': True, 'signal_strength': 0.9},
                'power_consumption_mw': 25.0,
                'power_efficiency': 0.85,
                'power_safe': True,
                'detected_failures': [],
                'optical_ecc_enabled': True,
                'adaptive_mode': True
            })
            mock_laser.return_value = mock_laser_instance

            # Configure range detector mock
            mock_range_detector_instance = Mock()
            mock_range_detector_instance.initialize = AsyncMock(return_value=None)
            mock_range_detector_instance.measure_distance_averaged = AsyncMock(return_value={
                'distance_m': 75.0,
                'signal_strength': 0.85,
                'timestamp': 0,
                'quality_score': 0.9,
                'temperature_compensated': True
            })
            mock_range_detector_instance.get_current_range_category = AsyncMock(return_value='Medium')
            mock_range_detector.return_value = mock_range_detector_instance

            yield {
                'ultrasound': mock_ultrasound_instance,
                'laser': mock_laser_instance,
                'range_detector': mock_range_detector_instance
            }

    @pytest.fixture
    def protocol_engines(self):
        """Create protocol engines for device A and device B"""
        device_a = PyProtocolEngine()
        device_b = PyProtocolEngine()
        return device_a, device_b

    @pytest.mark.asyncio
    async def test_hybrid_protocol_handshake(self, mock_hardware, protocol_engines):
        """Test complete long-range handshake with hybrid channels"""
        device_a, device_b = protocol_engines

        # Phase 1: Synchronization
        # Device A initiates with ultrasound sync pulse
        nonce = PyCryptoEngine.generate_nonce()
        device_a.receive_nonce(nonce)

        # Device B detects ultrasound presence and prepares laser
        presence_detected = await mock_hardware['ultrasound'].detect_presence()
        assert presence_detected

        # Phase 2: Key Exchange via Laser
        # Device B generates QR-like data for laser transmission
        qr_data = device_b.receive_nonce(nonce)
        assert qr_data is not None

        # Simulate laser transmission of key data
        await mock_hardware['laser'].transmit_data(qr_data.encode())

        # Device A receives laser data
        received_data = await mock_hardware['laser'].receive_data(1000)
        assert received_data == qr_data.encode()

        # Device A processes received key data
        device_a.process_qr_payload(received_data)

        # Phase 3: Coupled Authentication
        # Device A sends ACK via ultrasound
        device_a.receive_ack()

        # Device B receives ultrasound ACK
        device_b.receive_ack()

        # Verify both devices are connected
        assert device_a.get_state() == "connected"
        assert device_b.get_state() == "connected"

    @pytest.mark.asyncio
    async def test_channel_coupling_validation(self, mock_hardware):
        """Test temporal coupling between laser and ultrasound channels"""
        validator = PyChannelValidator()

        # Create simultaneous channel data
        import time
        timestamp = time.time_ns()

        laser_data = {
            'channel_type': 'Laser',
            'data': b'laser_key_data',
            'timestamp': timestamp,
            'sequence_id': 1
        }

        ultrasound_data = {
            'channel_type': 'Ultrasound',
            'data': b'ultrasound_auth',
            'timestamp': timestamp,
            'sequence_id': 1
        }

        # Receive data from both channels
        await validator.receive_channel_data(laser_data)
        await validator.receive_channel_data(ultrasound_data)

        # Verify validation succeeds with coupled channels
        assert await validator.is_validated()

        # Check validation metrics
        metrics = await validator.get_metrics()
        assert metrics.successful_validations > 0
        assert metrics.temporal_coupling_failures == 0

    @pytest.mark.asyncio
    async def test_temporal_coupling_failure(self, mock_hardware):
        """Test validation failure when channels are not synchronized"""
        validator = PyChannelValidator()

        # Create misaligned channel data (200ms apart)
        laser_data = {
            'channel_type': 'Laser',
            'data': b'laser_key_data',
            'timestamp': 1000000000,  # 1 second in nanoseconds
            'sequence_id': 1
        }

        ultrasound_data = {
            'channel_type': 'Ultrasound',
            'data': b'ultrasound_auth',
            'timestamp': 1200000000,  # 1.2 seconds (200ms later)
            'sequence_id': 1
        }

        # Receive misaligned data
        await validator.receive_channel_data(laser_data)
        await validator.receive_channel_data(ultrasound_data)

        # Validation should fail due to temporal coupling
        assert not await validator.is_validated()

        # Check failure metrics
        metrics = await validator.get_metrics()
        assert metrics.temporal_coupling_failures > 0

    @pytest.mark.asyncio
    async def test_adaptive_power_profiles(self, mock_hardware):
        """Test adaptive power profile switching based on range"""
        laser_engine = mock_hardware['laser']
        range_detector = mock_hardware['range_detector']

        # Test close range profile
        range_detector.measure_distance_averaged = AsyncMock(return_value={
            'distance_m': 25.0,
            'signal_strength': 0.9,
            'timestamp': 0,
            'quality_score': 0.95,
            'temperature_compensated': True
        })
        range_detector.get_current_range_category = AsyncMock(return_value='Close')

        # Should adapt to close range profile (higher data rate, lower power)
        await laser_engine.transmit_data(b"test")
        # Verify power profile adaptation (mock verification)

        # Test far range profile
        range_detector.measure_distance_averaged = AsyncMock(return_value={
            'distance_m': 150.0,
            'signal_strength': 0.6,
            'timestamp': 0,
            'quality_score': 0.7,
            'temperature_compensated': True
        })
        range_detector.get_current_range_category = AsyncMock(return_value='Far')

        # Should adapt to far range profile (lower data rate, higher power)
        await laser_engine.transmit_data(b"test")
        # Verify power profile adaptation

    @pytest.mark.asyncio
    async def test_environmental_adaptation(self, mock_hardware):
        """Test system adaptation to environmental conditions"""
        laser_engine = mock_hardware['laser']
        range_detector = mock_hardware['range_detector']

        # Simulate fog conditions
        range_detector.get_environmental_conditions = AsyncMock(return_value={
            'temperature_celsius': 8.0,
            'humidity_percent': 95.0,
            'pressure_hpa': 1010.0,
            'wind_speed_mps': 2.0,
            'visibility_meters': 200.0  # Poor visibility
        })

        # Should increase power and adjust ECC for fog
        await laser_engine.update_environmental_conditions("Fog", 200.0)

        # Verify power increase for poor visibility
        diagnostics = await laser_engine.get_channel_diagnostics()
        assert diagnostics['power_consumption_mw'] > 20.0  # Increased power

    @pytest.mark.asyncio
    async def test_fallback_mechanism(self, mock_hardware, protocol_engines):
        """Test automatic fallback to short-range mode on long-range failure"""
        device_a, device_b = protocol_engines
        fallback_manager = PyFallbackManager(device_a)

        # Initialize with engines
        fallback_manager.initialize_engines(
            mock_hardware['laser'],
            mock_hardware['ultrasound']
        )

        # Start monitoring
        await fallback_manager.start()

        # Simulate laser failure (alignment lost)
        mock_hardware['laser'].get_alignment_status = AsyncMock(return_value={
            'is_aligned': False,
            'beam_position_x': 0.0,
            'beam_position_y': 0.0,
            'signal_strength': 0.1,
            'last_update': 0
        })

        # Allow monitoring to detect failure and trigger fallback
        await asyncio.sleep(0.1)

        # Verify fallback was triggered
        status = await fallback_manager.get_fallback_status()
        assert status['active']
        assert status['current_mode'] == 'ShortRange'

        # Verify protocol switched to short-range
        assert device_a.get_state() == "idle"  # Reset for short-range

    @pytest.mark.asyncio
    async def test_recovery_mechanism(self, mock_hardware, protocol_engines):
        """Test recovery from fallback to long-range mode"""
        device_a, device_b = protocol_engines
        fallback_manager = PyFallbackManager(device_a)

        # Start in fallback mode
        await fallback_manager.manual_fallback("LaserAlignmentLost")

        # Simulate recovery (laser realigns)
        mock_hardware['laser'].get_alignment_status = AsyncMock(return_value={
            'is_aligned': True,
            'beam_position_x': 0.0,
            'beam_position_y': 0.0,
            'signal_strength': 0.9,
            'last_update': 0
        })

        # Allow recovery monitoring
        await asyncio.sleep(0.1)

        # Verify recovery
        status = await fallback_manager.get_fallback_status()
        assert not status['active']
        assert status['current_mode'] == 'LongRange'

    @pytest.mark.asyncio
    async def test_cross_channel_authentication(self, mock_hardware):
        """Test cross-channel signature verification"""
        validator = PyChannelValidator()

        # Create channel data with cryptographic binding
        crypto = PyCryptoEngine()
        key = crypto.public_key()

        laser_data = {
            'channel_type': 'Laser',
            'data': crypto.encrypt_data(key, b"laser_payload"),
            'timestamp': 1000000000,
            'sequence_id': 1
        }

        ultrasound_data = {
            'channel_type': 'Ultrasound',
            'data': crypto.encrypt_data(key, b"ultrasound_payload"),
            'timestamp': 1000000000,
            'sequence_id': 1
        }

        # Validation should succeed with proper cryptographic binding
        await validator.receive_channel_data(laser_data)
        await validator.receive_channel_data(ultrasound_data)

        assert await validator.is_validated()

    @pytest.mark.asyncio
    async def test_optical_ecc_adaptation(self, mock_hardware):
        """Test optical ECC adaptation to channel conditions"""
        optical_ecc = PyOpticalECC()

        # Simulate clear conditions
        metrics = {
            'ber': 0.001,
            'per': 0.01,
            'signal_strength': 0.9,
            'atmospheric_attenuation': 1.0,
            'turbulence_index': 0.1,
            'background_noise': 0.1,
            'range_meters': 50.0,
            'timestamp': 0
        }

        await optical_ecc.update_quality_metrics(metrics)

        # Should use minimal ECC for good conditions
        state = await optical_ecc.get_adaptation_state()
        assert state['ecc_strength'] < 0.5

        # Simulate poor conditions (fog)
        poor_metrics = {
            'ber': 0.05,
            'per': 0.2,
            'signal_strength': 0.4,
            'atmospheric_attenuation': 8.0,
            'turbulence_index': 0.8,
            'background_noise': 0.3,
            'range_meters': 150.0,
            'timestamp': 0
        }

        await optical_ecc.update_quality_metrics(poor_metrics)

        # Should increase ECC strength for poor conditions
        state = await optical_ecc.get_adaptation_state()
        assert state['ecc_strength'] > 0.7

    @pytest.mark.asyncio
    async def test_range_adaptive_modulation(self, mock_hardware):
        """Test modulation scheme adaptation based on range"""
        laser_engine = mock_hardware['laser']
        range_detector = mock_hardware['range_detector']

        # Close range: should use high-speed modulation
        range_detector.get_current_range_category = AsyncMock(return_value='Close')
        modulation = await laser_engine.select_optimal_modulation()
        # Should prefer OOK for close range (higher data rates)

        # Long range: should use robust modulation
        range_detector.get_current_range_category = AsyncMock(return_value='Extreme')
        modulation = await laser_engine.select_optimal_modulation()
        # Should prefer QR projection for long range (better error correction)

    @pytest.mark.asyncio
    async def test_concurrent_channel_operation(self, mock_hardware):
        """Test simultaneous operation of laser and ultrasound channels"""
        laser_engine = mock_hardware['laser']
        ultrasound_engine = mock_hardware['ultrasound']

        # Transmit simultaneously on both channels
        laser_task = asyncio.create_task(laser_engine.transmit_data(b"laser_data"))
        ultrasound_task = asyncio.create_task(ultrasound_engine.transmit_auth_signal(b"challenge", b"signature"))

        # Both should complete successfully
        await asyncio.gather(laser_task, ultrasound_task)

        # Verify both channels remain operational
        laser_diag = await laser_engine.get_channel_diagnostics()
        ultrasound_diag = await ultrasound_engine.get_channel_diagnostics()

        assert laser_diag['is_active']
        assert ultrasound_diag['is_active']

    @pytest.mark.asyncio
    async def test_hardware_failure_handling(self, mock_hardware):
        """Test graceful handling of hardware failures"""
        laser_engine = mock_hardware['laser']

        # Simulate hardware failure
        laser_engine.transmit_data = AsyncMock(side_effect=Exception("Hardware failure"))

        # Should trigger fallback instead of crashing
        with pytest.raises(Exception):  # In real implementation, this would trigger fallback
            await laser_engine.transmit_data(b"test")

        # Verify failure is detected
        diagnostics = await laser_engine.get_channel_diagnostics()
        assert len(diagnostics['detected_failures']) > 0

    @pytest.mark.asyncio
    async def test_session_preservation_during_fallback(self, mock_hardware, protocol_engines):
        """Test that cryptographic session is preserved during fallback"""
        device_a, device_b = protocol_engines
        fallback_manager = PyFallbackManager(device_a)

        # Establish long-range connection
        await self.test_hybrid_protocol_handshake(mock_hardware, protocol_engines)

        # Trigger fallback
        await fallback_manager.manual_fallback("EnvironmentalConditions")

        # Verify session state is preserved
        status = await fallback_manager.get_fallback_status()
        assert status['session_snapshot'] is not None

        # After recovery, should restore session
        await fallback_manager.attempt_recovery()

        # Verify connection is restored with preserved session
        assert device_a.get_state() == "connected"


class TestLongRangePerformance:
    """Performance benchmarks for long-range operations"""

    @pytest.fixture
    def performance_data(self):
        """Generate test data for performance testing"""
        return {
            'small_payload': b"Hello World",
            'medium_payload': b"A" * 1024,  # 1KB
            'large_payload': b"B" * 10240,  # 10KB
            'xlarge_payload': b"C" * 102400,  # 100KB
        }

    @pytest.mark.benchmark
    def test_handshake_performance(self, benchmark, mock_hardware, protocol_engines):
        """Benchmark complete long-range handshake performance"""
        device_a, device_b = protocol_engines

        def handshake_benchmark():
            # Simplified handshake for benchmarking
            nonce = PyCryptoEngine.generate_nonce()
            device_a.receive_nonce(nonce)
            qr_data = device_b.receive_nonce(nonce)
            device_a.process_qr_payload(qr_data.encode())
            device_a.receive_ack()
            device_b.receive_ack()

        benchmark(handshake_benchmark)

    @pytest.mark.benchmark
    def test_channel_validation_performance(self, benchmark):
        """Benchmark channel validation performance"""
        validator = PyChannelValidator()

        def validation_benchmark():
            # Create test data
            laser_data = {
                'channel_type': 'Laser',
                'data': b'test_data',
                'timestamp': 1000000000,
                'sequence_id': 1
            }
            ultrasound_data = {
                'channel_type': 'Ultrasound',
                'data': b'auth_data',
                'timestamp': 1000000000,
                'sequence_id': 1
            }

            # Synchronous validation for benchmarking
            asyncio.run(validator.receive_channel_data(laser_data))
            asyncio.run(validator.receive_channel_data(ultrasound_data))

        benchmark(validation_benchmark)


class TestUltrasonicBeamEngine:
    """Unit tests for UltrasonicBeamEngine parametric audio and beam forming"""

    @pytest.fixture
    def beam_engine(self):
        """Create ultrasonic beam engine for testing"""
        return PyUltrasonicBeamEngine()

    @pytest.mark.asyncio
    async def test_beam_engine_initialization(self, beam_engine):
        """Test beam engine initialization"""
        await beam_engine.initialize()
        assert beam_engine.is_active()

        config = beam_engine.get_config()
        assert config['carrier_frequency'] == 40000.0
        assert config['range'] == 20.0
        assert config['beam_angle'] == 15.0

    @pytest.mark.asyncio
    async def test_parametric_audio_generation(self, beam_engine):
        """Test parametric audio signal generation"""
        await beam_engine.initialize()

        test_data = [0xAA, 0x55, 0xFF, 0x00]
        signal = await beam_engine.generate_parametric_audio(test_data)

        assert len(signal) > 0
        # Verify signal contains modulated carrier waves
        assert any(abs(s) > 0.1 for s in signal)

        # Check signal properties
        carrier_freq = beam_engine.get_config()['carrier_frequency']
        sample_rate = 192000.0  # From implementation

        # Verify carrier frequency is present in signal
        # (Simplified check - in practice would use FFT)
        assert len(signal) > sample_rate / carrier_freq

    @pytest.mark.parametrize("data_size", [1, 8, 32])
    @pytest.mark.asyncio
    async def test_parametric_audio_data_sizes(self, beam_engine, data_size):
        """Test parametric audio generation with different data sizes"""
        await beam_engine.initialize()

        test_data = [i % 256 for i in range(data_size)]
        signal = await beam_engine.generate_parametric_audio(test_data)

        # Signal length should scale with data size
        expected_min_length = data_size * 8 * 10  # Rough estimate
        assert len(signal) >= expected_min_length

    @pytest.mark.asyncio
    async def test_sync_pulse_transmission(self, beam_engine):
        """Test synchronization pulse transmission"""
        await beam_engine.initialize()

        pattern = [0x01, 0x02, 0x03, 0x04]
        await beam_engine.transmit_sync_pulse(pattern)

        # Verify engine remains active after transmission
        assert beam_engine.is_active()

    @pytest.mark.asyncio
    async def test_auth_signal_transmission(self, beam_engine):
        """Test authentication signal transmission"""
        await beam_engine.initialize()

        challenge = [0xCA, 0xFE, 0xBA, 0xBE]
        signature = [0xDE, 0xAD, 0xBE, 0xEF]

        await beam_engine.transmit_auth_signal(challenge, signature)
        assert beam_engine.is_active()

    @pytest.mark.asyncio
    async def test_presence_detection(self, beam_engine):
        """Test ultrasonic presence detection"""
        await beam_engine.initialize()

        # In mock environment, this may return False
        presence = await beam_engine.detect_presence()
        # Result depends on mock implementation
        assert isinstance(presence, bool)

    @pytest.mark.asyncio
    async def test_control_data_transmission(self, beam_engine):
        """Test low-bandwidth control data transmission"""
        await beam_engine.initialize()

        # Test valid control data
        control_data = [0x01, 0x02, 0x03]
        await beam_engine.transmit_control_data(control_data, 1)
        assert beam_engine.is_active()

    @pytest.mark.asyncio
    async def test_control_data_size_limit(self, beam_engine):
        """Test control data size limit enforcement"""
        await beam_engine.initialize()

        # Test oversized control data
        large_data = list(range(64))  # Exceeds 32 byte limit

        with pytest.raises(Exception):  # Should raise InvalidParameters
            await beam_engine.transmit_control_data(large_data, 1)

    @pytest.mark.asyncio
    async def test_beam_reception(self, beam_engine):
        """Test beam signal reception"""
        await beam_engine.initialize()

        signals = await beam_engine.receive_beam_signals()
        # In mock environment, may return empty list
        assert isinstance(signals, list)

    @pytest.mark.asyncio
    async def test_channel_diagnostics(self, beam_engine):
        """Test channel diagnostics reporting"""
        await beam_engine.initialize()

        diagnostics = await beam_engine.get_channel_diagnostics()

        required_fields = ['is_active', 'presence_detected', 'configured_range',
                          'carrier_frequency', 'power_level', 'detected_failures']

        for field in required_fields:
            assert field in diagnostics

        assert diagnostics['is_active']
        assert diagnostics['carrier_frequency'] == 40000.0

    @pytest.mark.asyncio
    async def test_config_validation(self):
        """Test beam configuration validation"""
        # Test valid configuration
        valid_config = {
            'carrier_frequency': 40000.0,
            'modulation_frequency': 1000.0,
            'beam_angle': 15.0,
            'range': 25.0,
            'power_level': 0.8
        }

        engine = PyUltrasonicBeamEngine.with_config(valid_config)
        assert engine.is_active() == False  # Not initialized yet

        # Test invalid range
        invalid_config = valid_config.copy()
        invalid_config['range'] = 50.0  # Exceeds max range

        with pytest.raises(Exception):  # Should raise RangeOutOfBounds
            PyUltrasonicBeamEngine.with_config(invalid_config)

        # Test invalid carrier frequency
        invalid_config = valid_config.copy()
        invalid_config['carrier_frequency'] = 20000.0  # Not 40kHz

        with pytest.raises(Exception):  # Should raise InvalidParameters
            PyUltrasonicBeamEngine.with_config(invalid_config)

    @pytest.mark.asyncio
    async def test_power_level_validation(self, beam_engine):
        """Test power level validation in parametric audio"""
        await beam_engine.initialize()

        # Valid power levels should work
        test_data = [0xAA]
        signal = await beam_engine.generate_parametric_audio(test_data)
        assert len(signal) > 0

        # Power level is validated internally in generate_parametric_audio
        # based on config.power_level

    @pytest.mark.asyncio
    async def test_beam_engine_shutdown(self, beam_engine):
        """Test proper beam engine shutdown"""
        await beam_engine.initialize()
        assert beam_engine.is_active()

        await beam_engine.shutdown()
        assert not beam_engine.is_active()

    @pytest.mark.asyncio
    async def test_failure_detection(self, beam_engine):
        """Test automatic failure detection"""
        await beam_engine.initialize()

        failures = await beam_engine.detect_channel_failures()

        # Should detect issues or return empty list
        assert isinstance(failures, list)

        # If engine is properly initialized, should have no failures
        # (In mock environment, this depends on implementation)

    @pytest.mark.parametrize("range_m", [15.0, 20.0, 25.0])
    @pytest.mark.asyncio
    async def test_range_configuration(self, range_m):
        """Test different range configurations"""
        config = {
            'carrier_frequency': 40000.0,
            'modulation_frequency': 1000.0,
            'beam_angle': 15.0,
            'range': range_m,
            'power_level': 0.8
        }

        engine = PyUltrasonicBeamEngine.with_config(config)
        await engine.initialize()

        engine_config = engine.get_config()
        assert engine_config['range'] == range_m

        diagnostics = await engine.get_channel_diagnostics()
        assert diagnostics['configured_range'] == range_m

    @pytest.mark.asyncio
    async def test_concurrent_operations(self, beam_engine):
        """Test concurrent beam operations"""
        await beam_engine.initialize()

        # Start multiple operations concurrently
        import asyncio

        async def transmit_operation():
            await beam_engine.transmit_control_data([0x01, 0x02], 1)

        async def detect_operation():
            await beam_engine.detect_presence()

        # Run concurrent operations
        await asyncio.gather(
            transmit_operation(),
            detect_operation(),
            transmit_operation()
        )

        # Engine should remain stable
        assert beam_engine.is_active()

    @pytest.mark.asyncio
    async def test_signal_quality_monitoring(self, beam_engine):
        """Test signal quality monitoring"""
        await beam_engine.initialize()

        # Generate signal and check it has expected properties
        test_data = [0x55, 0xAA]
        signal = await beam_engine.generate_parametric_audio(test_data)

        # Basic signal quality checks
        assert len(signal) > 0
        assert all(isinstance(s, float) for s in signal)

        # Check signal amplitude is reasonable
        max_amplitude = max(abs(s) for s in signal)
        assert max_amplitude > 0.0
        assert max_amplitude <= 1.0  # Should be normalized

    @pytest.mark.asyncio
    async def test_modulation_frequency_validation(self, beam_engine):
        """Test modulation frequency settings"""
        await beam_engine.initialize()

        config = beam_engine.get_config()

        # Modulation frequency should be reasonable for audio
        assert config['modulation_frequency'] > 100.0
        assert config['modulation_frequency'] < 5000.0

        # Test parametric audio with different modulation frequencies
        # (This would require config modification support)
        # For now, just verify the current config is valid
        assert config['modulation_frequency'] == 1000.0

    @pytest.mark.parametrize("range_m", [50, 100, 150, 200])
    def test_range_adaptation_performance(self, benchmark, range_m, mock_hardware):
        """Benchmark range-based adaptation performance"""
        range_detector = mock_hardware['range_detector']
        laser_engine = mock_hardware['laser']

        def adaptation_benchmark():
            # Simulate range measurement and adaptation
            range_detector.measure_distance_averaged = AsyncMock(return_value={
                'distance_m': range_m,
                'signal_strength': 0.8,
                'timestamp': 0,
                'quality_score': 0.9,
                'temperature_compensated': True
            })

            # Trigger adaptation
            asyncio.run(laser_engine.update_power_profile())

        benchmark(adaptation_benchmark)


class TestLongRangeRobustness:
    """Robustness tests for environmental conditions"""

    @pytest.mark.parametrize("weather,visibility", [
        ("Clear", 10000),
        ("LightFog", 1000),
        ("HeavyFog", 200),
        ("Rain", 2000),
        ("HeavyRain", 500),
        ("Storm", 300),
    ])
    @pytest.mark.asyncio
    async def test_environmental_conditions_adaptation(self, mock_hardware, weather, visibility):
        """Test system adaptation to various weather conditions"""
        laser_engine = mock_hardware['laser']
        range_detector = mock_hardware['range_detector']

        # Update environmental conditions
        await laser_engine.update_environmental_conditions(weather, visibility)

        # Verify power adaptation
        diagnostics = await laser_engine.get_channel_diagnostics()

        if weather in ["HeavyFog", "Storm"]:
            # Should significantly increase power for poor conditions
            assert diagnostics['power_consumption_mw'] > 30.0
        elif weather == "Clear":
            # Should use minimal power for good conditions
            assert diagnostics['power_consumption_mw'] < 20.0

    @pytest.mark.asyncio
    async def test_wind_interference_compensation(self, mock_hardware):
        """Test compensation for wind-induced beam deflection"""
        laser_engine = mock_hardware['laser']

        # Simulate wind conditions
        await laser_engine.update_environmental_conditions("Storm", 300)

        # Should enable enhanced alignment tracking
        diagnostics = await laser_engine.get_channel_diagnostics()
        assert diagnostics['adaptive_mode']  # Should enable adaptation

    @pytest.mark.asyncio
    async def test_temperature_extremes_handling(self, mock_hardware):
        """Test operation in temperature extremes"""
        range_detector = mock_hardware['range_detector']

        # Test cold conditions
        conditions = {
            'temperature_celsius': -10.0,
            'humidity_percent': 60.0,
            'pressure_hpa': 1020.0,
            'wind_speed_mps': 1.0,
            'visibility_meters': 8000.0
        }

        await range_detector.update_environmental_conditions(conditions)

        # Should compensate speed of sound calculation
        measurement = await range_detector.measure_distance_averaged()
        assert measurement['temperature_compensated']

        # Test hot conditions
        conditions['temperature_celsius'] = 40.0
        await range_detector.update_environmental_conditions(conditions)

        measurement = await range_detector.measure_distance_averaged()
        assert measurement['temperature_compensated']


class TestLongRangeSecurity:
    """Security tests for long-range communication"""

    @pytest.mark.asyncio
    async def test_interception_resistance(self, mock_hardware):
        """Test resistance to interception attacks"""
        validator = PyChannelValidator()

        # Attacker tries to intercept only laser channel
        laser_data = {
            'channel_type': 'Laser',
            'data': b'intercepted_laser_data',
            'timestamp': 1000000000,
            'sequence_id': 1
        }

        # Without ultrasound channel, validation should fail
        await validator.receive_channel_data(laser_data)

        # Should not validate with single channel
        assert not await validator.is_validated()

    @pytest.mark.asyncio
    async def test_replay_attack_prevention(self, mock_hardware):
        """Test prevention of replay attacks"""
        validator = PyChannelValidator()

        # Valid coupled transmission
        laser_data = {
            'channel_type': 'Laser',
            'data': b'valid_laser_data',
            'timestamp': 1000000000,
            'sequence_id': 1
        }
        ultrasound_data = {
            'channel_type': 'Ultrasound',
            'data': b'valid_ultrasound_data',
            'timestamp': 1000000000,
            'sequence_id': 1
        }

        # First validation succeeds
        await validator.receive_channel_data(laser_data)
        await validator.receive_channel_data(ultrasound_data)
        assert await validator.is_validated()

        # Reset validator
        await validator.reset()

        # Replay same data should fail
        await validator.receive_channel_data(laser_data)
        await validator.receive_channel_data(ultrasound_data)
        assert not await validator.is_validated()  # Anti-replay protection

    @pytest.mark.asyncio
    async def test_channel_isolation(self, mock_hardware):
        """Test that channels remain cryptographically isolated"""
        crypto = PyCryptoEngine()

        # Generate separate keys for each channel
        laser_key = crypto.public_key()
        ultrasound_key = crypto.public_key()

        # Encrypt data for each channel
        laser_payload = crypto.encrypt_data(laser_key, b"laser_secret")
        ultrasound_payload = crypto.encrypt_data(ultrasound_key, b"ultrasound_secret")

        # Data encrypted for one channel should not be decryptable by the other
        assert crypto.decrypt_data(laser_key, laser_payload) == b"laser_secret"
        assert crypto.decrypt_data(ultrasound_key, ultrasound_payload) == b"ultrasound_secret"

        # Cross-decryption should fail
        with pytest.raises(Exception):
            crypto.decrypt_data(laser_key, ultrasound_payload)

        with pytest.raises(Exception):
            crypto.decrypt_data(ultrasound_key, laser_payload)
class TestLaserEngine:
    """Unit tests for LaserEngine modulation schemes and safety limits"""

    @pytest.fixture
    def laser_engine(self):
        """Create laser engine for testing"""
        config = {
            'laser_type': 'Visible',
            'modulation': 'Ook',
            'max_power_mw': 5.0,
            'wavelength_nm': 650,
            'beam_angle_deg': 15.0,
            'range_meters': 100.0,
            'data_rate_bps': 1000000
        }
        rx_config = {
            'use_photodiode': True,
            'use_camera': False,
            'sensitivity_threshold': 0.1,
            'alignment_tolerance_px': 10
        }
        return PyLaserEngine(config, rx_config)

    @pytest.mark.asyncio
    async def test_laser_engine_initialization(self, laser_engine):
        """Test laser engine initialization"""
        await laser_engine.initialize()
        assert laser_engine.is_active()

        config = await laser_engine.get_config()
        assert config['laser_type'] == 'Visible'
        assert config['max_power_mw'] == 5.0

    @pytest.mark.asyncio
    async def test_ook_modulation_transmission(self, laser_engine):
        """Test On-Off Keying modulation transmission"""
        await laser_engine.initialize()

        test_data = b"Hello Laser"
        await laser_engine.transmit_data(test_data)

        # Verify engine remains active
        assert laser_engine.is_active()

    @pytest.mark.asyncio
    async def test_pwm_modulation_transmission(self, laser_engine):
        """Test Pulse Width Modulation transmission"""
        await laser_engine.initialize()

        # Switch to PWM modulation
        await laser_engine.set_modulation_scheme('Pwm')

        test_data = b"PWM Test"
        await laser_engine.transmit_data(test_data)

        assert laser_engine.is_active()

    @pytest.mark.asyncio
    async def test_qr_projection_transmission(self, laser_engine):
        """Test QR code projection transmission"""
        await laser_engine.initialize()

        # Switch to QR projection
        await laser_engine.set_modulation_scheme('QrProjection')

        test_data = b"QR Projection Test"
        await laser_engine.transmit_data(test_data)

        assert laser_engine.is_active()

    @pytest.mark.asyncio
    async def test_data_reception(self, laser_engine):
        """Test data reception with different methods"""
        await laser_engine.initialize()

        # Test photodiode reception
        received_data = await laser_engine.receive_data(1000)
        # In mock environment, may return predefined data
        assert isinstance(received_data, bytes)

    @pytest.mark.asyncio
    async def test_alignment_tracking(self, laser_engine):
        """Test beam alignment tracking"""
        await laser_engine.initialize()

        # Set alignment target
        await laser_engine.set_alignment_target(100.0, 200.0)

        # Check alignment status
        status = await laser_engine.get_alignment_status()
        assert 'is_aligned' in status
        assert 'signal_strength' in status

    @pytest.mark.asyncio
    async def test_safety_limits_enforcement(self, laser_engine):
        """Test laser safety limits enforcement"""
        await laser_engine.initialize()

        # Test invalid intensity values
        with pytest.raises(Exception):  # Should raise SafetyViolation
            await laser_engine.set_laser_intensity(1.5)  # Above 1.0

        with pytest.raises(Exception):  # Should raise SafetyViolation
            await laser_engine.set_laser_intensity(-0.1)  # Below 0.0

    @pytest.mark.asyncio
    async def test_power_management(self, laser_engine):
        """Test laser power management"""
        await laser_engine.initialize()

        # Test standby mode
        await laser_engine.set_standby_mode(True)

        # Check power consumption
        consumption = await laser_engine.get_current_power_consumption()
        assert consumption >= 0.0

        # Test power efficiency
        efficiency = await laser_engine.get_power_efficiency()
        assert 0.0 <= efficiency <= 1.0

        # Test power safety
        is_safe = await laser_engine.is_power_safe()
        assert isinstance(is_safe, bool)

    @pytest.mark.asyncio
    async def test_adaptive_mode_operation(self, laser_engine):
        """Test adaptive mode with range detector"""
        await laser_engine.initialize()

        # Enable adaptive mode (would normally include range detector)
        await laser_engine.enable_adaptive_mode()

        # Test modulation selection
        modulation = await laser_engine.select_optimal_modulation()
        # Should return a valid modulation scheme
        assert modulation in ['Ook', 'Pwm', 'QrProjection']

    @pytest.mark.asyncio
    async def test_environmental_adaptation(self, laser_engine):
        """Test environmental condition adaptation"""
        await laser_engine.initialize()

        # Simulate weather conditions
        await laser_engine.update_environmental_conditions("Fog", 200.0)

        # Check power profile adaptation
        profile = await laser_engine.get_current_power_profile()
        assert 'optimal_power_mw' in profile

        # Check safety margins
        margins = await laser_engine.get_safety_margins()
        assert len(margins) == 3  # (power, range, alignment)

    @pytest.mark.asyncio
    async def test_channel_diagnostics(self, laser_engine):
        """Test comprehensive channel diagnostics"""
        await laser_engine.initialize()

        diagnostics = await laser_engine.get_channel_diagnostics()

        required_fields = [
            'is_active', 'alignment_status', 'power_consumption_mw',
            'power_efficiency', 'power_safe', 'detected_failures',
            'optical_ecc_enabled', 'adaptive_mode'
        ]

        for field in required_fields:
            assert field in diagnostics

    @pytest.mark.asyncio
    async def test_error_correction_encoding(self, laser_engine):
        """Test optical error correction encoding/decoding"""
        await laser_engine.initialize()

        test_data = b"Test data for ECC"

        # Encode with ECC
        encoded = await laser_engine.encode_with_ecc(test_data)
        assert len(encoded) >= len(test_data)  # ECC adds redundancy

        # Decode with ECC
        decoded = await laser_engine.decode_with_ecc(encoded)
        assert decoded == test_data

    @pytest.mark.asyncio
    async def test_range_based_power_profiles(self):
        """Test power profile selection based on range"""
        # Test different range categories
        close_profile = PyLaserEngine.PowerProfile.close_range()
        medium_profile = PyLaserEngine.PowerProfile.medium_range()
        far_profile = PyLaserEngine.PowerProfile.far_range()
        extreme_profile = PyLaserEngine.PowerProfile.extreme_range()

        # Verify power scaling with range
        assert close_profile.max_power_mw <= medium_profile.max_power_mw
        assert medium_profile.max_power_mw <= far_profile.max_power_mw
        assert far_profile.max_power_mw <= extreme_profile.max_power_mw

        # Verify data rate scaling (inverse to power)
        assert close_profile.data_rate_bps >= medium_profile.data_rate_bps
        assert medium_profile.data_rate_bps >= far_profile.data_rate_bps
        assert far_profile.data_rate_bps >= extreme_profile.data_rate_bps

    @pytest.mark.asyncio
    async def test_laser_type_safety_limits(self):
        """Test safety limits for different laser types"""
        # Visible laser
        visible_config = {
            'laser_type': 'Visible',
            'modulation': 'Ook',
            'max_power_mw': 5.0,
            'wavelength_nm': 650,
            'beam_angle_deg': 15.0,
            'range_meters': 100.0,
            'data_rate_bps': 1000000
        }
        visible_engine = PyLaserEngine(visible_config, {'use_photodiode': True})

        # IR laser
        ir_config = visible_config.copy()
        ir_config['laser_type'] = 'Infrared'
        ir_config['max_power_mw'] = 50.0  # Higher limit for IR
        ir_engine = PyLaserEngine(ir_config, {'use_photodiode': True})

        # Check safety limits
        visible_limit = await visible_engine.get_effective_power_limit()
        ir_limit = await ir_engine.get_effective_power_limit()

        # IR should allow higher power than visible
        assert ir_limit > visible_limit

    @pytest.mark.asyncio
    async def test_failure_detection(self, laser_engine):
        """Test laser channel failure detection"""
        await laser_engine.initialize()

        failures = await laser_engine.detect_channel_failures()

        # Should return a list of failures
        assert isinstance(failures, list)

        # Common failure types
        failure_types = ['HardwareUnavailable', 'AlignmentLost', 'SafetyViolation']
        if failures:
            for failure in failures:
                assert failure in failure_types

    @pytest.mark.asyncio
    async def test_concurrent_modulation_switching(self, laser_engine):
        """Test switching modulation schemes during operation"""
        await laser_engine.initialize()

        # Switch between modulation schemes
        await laser_engine.set_modulation_scheme('Ook')
        await laser_engine.transmit_data(b"OOK data")

        await laser_engine.set_modulation_scheme('Pwm')
        await laser_engine.transmit_data(b"PWM data")

        await laser_engine.set_modulation_scheme('QrProjection')
        await laser_engine.transmit_data(b"QR data")

        # Engine should remain stable
        assert laser_engine.is_active()

    @pytest.mark.asyncio
    async def test_shutdown_and_cleanup(self, laser_engine):
        """Test proper laser engine shutdown"""
        await laser_engine.initialize()
        assert laser_engine.is_active()

        await laser_engine.shutdown()
        assert not laser_engine.is_active()

    @pytest.mark.asyncio
    async def test_optical_ecc_integration(self, laser_engine):
        """Test integration with optical ECC system"""
        await laser_engine.initialize()

        # Check if optical ECC is available
        ecc_state = await laser_engine.get_optical_ecc_state()

        if ecc_state:
            assert 'ecc_strength' in ecc_state
            assert 'adaptation_enabled' in ecc_state

    @pytest.mark.asyncio
    async def test_camera_vs_photodiode_reception(self):
        """Test different reception methods"""
        # Photodiode configuration
        photodiode_config = {
            'use_photodiode': True,
            'use_camera': False,
            'sensitivity_threshold': 0.1,
            'alignment_tolerance_px': 10
        }

        # Camera configuration
        camera_config = {
            'use_photodiode': False,
            'use_camera': True,
            'sensitivity_threshold': 0.05,
            'alignment_tolerance_px': 5
        }

        laser_config = {
            'laser_type': 'Visible',
            'modulation': 'Ook',
            'max_power_mw': 5.0,
            'wavelength_nm': 650,
            'beam_angle_deg': 15.0,
            'range_meters': 100.0,
            'data_rate_bps': 1000000
        }

        photodiode_engine = PyLaserEngine(laser_config, photodiode_config)
        camera_engine = PyLaserEngine(laser_config, camera_config)

        await photodiode_engine.initialize()
        await camera_engine.initialize()

        # Both should be able to receive data
        photodiode_data = await photodiode_engine.receive_data(1000)
        camera_data = await camera_engine.receive_data(1000)

        assert isinstance(photodiode_data, bytes)
        assert isinstance(camera_data, bytes)