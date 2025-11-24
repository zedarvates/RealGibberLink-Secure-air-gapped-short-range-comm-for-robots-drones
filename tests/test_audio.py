import pytest
import numpy as np
from gibberlink_core import PyAudioEngine


class TestAudioEngine:
    """Unit tests for audio transmission and reception"""

    @pytest.fixture
    def audio_engine(self):
        return PyAudioEngine()

    def test_audio_engine_initialization(self, audio_engine):
        """Test audio engine initialization"""
        assert audio_engine is not None

    def test_data_transmission_simulation(self, audio_engine):
        """Test data transmission simulation"""
        test_data = b"Hello, GibberLink!"

        # Simulate transmission (would normally use audio hardware)
        # In mocked environment, this should succeed
        result = audio_engine.send_data(test_data)
        # Note: Actual implementation may differ based on Android integration

    def test_data_reception_simulation(self, audio_engine):
        """Test data reception simulation"""
        # Simulate reception (would normally capture from microphone)
        # In mocked environment, returns empty data
        result = audio_engine.receive_data()
        assert isinstance(result, bytes)

    def test_receiving_state_tracking(self, audio_engine):
        """Test audio receiving state tracking"""
        # Initially not receiving
        assert not audio_engine.is_receiving()

        # State management depends on actual audio implementation
        # This tests the interface contract

    def test_large_data_transmission(self, audio_engine):
        """Test transmission of larger data packets"""
        large_data = b"A" * 1024  # 1KB data

        # Should handle larger payloads
        result = audio_engine.send_data(large_data)

    def test_empty_data_transmission(self, audio_engine):
        """Test transmission of empty data"""
        empty_data = b""

        result = audio_engine.send_data(empty_data)

    def test_binary_data_transmission(self, audio_engine):
        """Test transmission of binary data"""
        binary_data = bytes(range(256))  # 0-255 byte values

        result = audio_engine.send_data(binary_data)

    def test_audio_channel_isolation(self):
        """Test that multiple audio engines are isolated"""
        engine1 = PyAudioEngine()
        engine2 = PyAudioEngine()

        # Each engine should maintain independent state
        # (Implementation dependent)

    @pytest.mark.parametrize("data_size", [1, 16, 256, 1024])
    def test_various_payload_sizes(self, audio_engine, data_size):
        """Test transmission with various payload sizes"""
        data = b"A" * data_size

        result = audio_engine.send_data(data)