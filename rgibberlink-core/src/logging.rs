//! # Signed Logging Module
//!
//! Implements append-only logs with Ed25519 signatures for audit trail and traceability.

use crate::crypto::{CryptoEngine, CryptoError};
use std::collections::VecDeque;
use std::time::{Instant, Duration};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEvent {
    SessionStarted { peer_fingerprint: [u8; 32], timestamp: Instant },
    KeyExchanged { key_id: [u8; 16], ephemeral: bool },
    MessageSent { sequence_id: u64, size_bytes: usize },
    MessageReceived { sequence_id: u64, size_bytes: usize },
    ValidationPassed { channel_type: String },
    ValidationFailed { channel_type: String, reason: String },
    SessionExpired { key_id: [u8; 16] },
    AuthenticationGranted { permissions: Vec<String> },
    AuthenticationDenied { reason: String },
    ChannelConnected { channel_type: String },
    ChannelDisconnected { channel_type: String, reason: String },
    ErrorOccurred { error_type: String, details: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub sequence_number: u64,
    pub event: LogEvent,
    pub device_fingerprint: [u8; 32],
    pub signature: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("Log signature verification failed")]
    SignatureVerificationFailed,
    #[error("Invalid log entry format")]
    InvalidFormat,
    #[error("Log sequence violation")]
    SequenceViolation,
    #[error("Log tampering detected")]
    TamperingDetected,
}

pub struct SignedLogger {
    crypto_engine: CryptoEngine,
    device_fingerprint: [u8; 32],
    log_entries: VecDeque<LogEntry>,
    next_sequence: u64,
    max_entries: usize,
}

impl SignedLogger {
    pub fn new(crypto_engine: CryptoEngine, device_id: &[u8], max_entries: usize) -> Self {
        let device_fingerprint = CryptoEngine::generate_device_fingerprint(device_id);
        Self {
            crypto_engine,
            device_fingerprint,
            log_entries: VecDeque::new(),
            next_sequence: 1,
            max_entries,
        }
    }

    pub fn log_event(&mut self, event: LogEvent) -> Result<(), LogError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| LogError::InvalidFormat)?
            .as_millis() as u64;

        // Create unsigned entry first
        let mut entry = LogEntry {
            timestamp,
            sequence_number: self.next_sequence,
            event,
            device_fingerprint: self.device_fingerprint,
            signature: Vec::new(),
        };

        // Sign the entry data (excluding the signature field)
        let entry_data = self.serialize_entry_without_signature(&entry)?;
        entry.signature = self.crypto_engine.sign_log_entry(&entry_data)
            .map_err(|_| LogError::InvalidFormat)?;

        // Add to log
        self.log_entries.push_back(entry);
        self.next_sequence += 1;

        // Maintain max entries
        while self.log_entries.len() > self.max_entries {
            self.log_entries.pop_front();
        }

        Ok(())
    }

    pub fn verify_log_integrity(&self) -> Result<(), LogError> {
        let mut expected_sequence = 1u64;

        for entry in &self.log_entries {
            // Check sequence number
            if entry.sequence_number != expected_sequence {
                return Err(LogError::SequenceViolation);
            }

            // Verify signature
            let entry_data = self.serialize_entry_without_signature(entry)?;
            if CryptoEngine::verify_log_signature(
                self.crypto_engine.ed25519_public_key(),
                &entry_data,
                &entry.signature,
            ).is_err() {
                return Err(LogError::TamperingDetected);
            }

            expected_sequence += 1;
        }

        Ok(())
    }

    pub fn get_entries_since(&self, timestamp: u64) -> Vec<&LogEntry> {
        self.log_entries.iter()
            .filter(|entry| entry.timestamp >= timestamp)
            .collect()
    }

    pub fn export_log(&self) -> Vec<u8> {
        bincode::serialize(&self.log_entries).unwrap_or_default()
    }

    pub fn import_log(&mut self, data: &[u8]) -> Result<(), LogError> {
        let entries: VecDeque<LogEntry> = bincode::deserialize(data)
            .map_err(|_| LogError::InvalidFormat)?;

        // Verify all entries before importing
        let temp_logger = Self {
            crypto_engine: self.crypto_engine.clone(),
            device_fingerprint: self.device_fingerprint,
            log_entries: entries,
            next_sequence: 0,
            max_entries: 0,
        };

        temp_logger.verify_log_integrity()?;

        self.log_entries = temp_logger.log_entries;
        self.next_sequence = self.log_entries.back()
            .map(|e| e.sequence_number + 1)
            .unwrap_or(1);

        Ok(())
    }

    fn serialize_entry_without_signature(&self, entry: &LogEntry) -> Result<Vec<u8>, LogError> {
        // Create a temporary entry without signature for signing/verification
        let temp_entry = LogEntry {
            timestamp: entry.timestamp,
            sequence_number: entry.sequence_number,
            event: entry.event.clone(),
            device_fingerprint: entry.device_fingerprint,
            signature: Vec::new(),
        };

        bincode::serialize(&temp_entry).map_err(|_| LogError::InvalidFormat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_creation_and_verification() {
        let crypto = CryptoEngine::new();
        let mut logger = SignedLogger::new(crypto, b"test_device", 100);

        let event = LogEvent::SessionStarted {
            peer_fingerprint: [1u8; 32],
            timestamp: Instant::now(),
        };

        logger.log_event(event).unwrap();
        assert_eq!(logger.log_entries.len(), 1);

        // Verify integrity
        assert!(logger.verify_log_integrity().is_ok());
    }

    #[test]
    fn test_sequence_violation_detection() {
        let crypto = CryptoEngine::new();
        let mut logger = SignedLogger::new(crypto, b"test_device", 100);

        let event1 = LogEvent::SessionStarted {
            peer_fingerprint: [1u8; 32],
            timestamp: Instant::now(),
        };
        logger.log_event(event1).unwrap();

        // Manually corrupt sequence (skip sequence check for testing)
        if let Some(entry) = logger.log_entries.back_mut() {
            entry.sequence_number = 5; // Should be 1
        }

        // This should fail verification
        assert!(matches!(logger.verify_log_integrity(), Err(LogError::SequenceViolation)));
    }

    #[test]
    fn test_log_import_export() {
        let crypto1 = CryptoEngine::new();
        let mut logger1 = SignedLogger::new(crypto1.clone(), b"test_device", 100);

        let event = LogEvent::MessageSent {
            sequence_id: 1,
            size_bytes: 256,
        };
        logger1.log_event(event).unwrap();

        // Export log
        let exported = logger1.export_log();

        // Import into new logger with same crypto engine
        let mut logger2 = SignedLogger::new(crypto1, b"test_device", 100);
        logger2.import_log(&exported).unwrap();

        // Verify contents match
        assert_eq!(logger1.log_entries.len(), logger2.log_entries.len());
        assert!(logger2.verify_log_integrity().is_ok());
    }
}
