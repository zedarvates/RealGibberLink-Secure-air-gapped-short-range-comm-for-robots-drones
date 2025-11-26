//! Python bindings for cryptographic and protocol components

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use crate::crypto::{CryptoEngine, CryptoError};
use crate::visual::{VisualEngine, VisualError, VisualPayload};
use crate::audio::AudioEngine;
use crate::protocol::{ProtocolEngine, ProtocolError, ProtocolState};
use crate::RgibberLink;

/// Python wrapper for CryptoEngine
#[pyclass]
pub struct PyCryptoEngine {
    inner: CryptoEngine,
}

#[pymethods]
impl PyCryptoEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: CryptoEngine::new(),
        }
    }

    fn public_key(&self) -> Vec<u8> {
        self.inner.public_key().to_vec()
    }

    fn derive_shared_secret(&mut self, peer_public_key: Vec<u8>) -> PyResult<[u8; 32]> {
        self.inner.derive_shared_secret(&peer_public_key)
            .map_err(|e| PyRuntimeError::new_err(format!("Crypto error: {}", e)))
    }

    #[staticmethod]
    fn encrypt_data(key: Vec<u8>, data: Vec<u8>) -> PyResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(PyRuntimeError::new_err("Key must be 32 bytes"));
        }
        let key_array: [u8; 32] = key.try_into().map_err(|_| PyRuntimeError::new_err("Invalid key length"))?;
        CryptoEngine::encrypt_data(&key_array, &data)
            .map_err(|e| PyRuntimeError::new_err(format!("Encryption error: {}", e)))
    }

    #[staticmethod]
    fn decrypt_data(key: Vec<u8>, encrypted_data: Vec<u8>) -> PyResult<Vec<u8>> {
        if key.len() != 32 {
            return Err(PyRuntimeError::new_err("Key must be 32 bytes"));
        }
        let key_array: [u8; 32] = key.try_into().map_err(|_| PyRuntimeError::new_err("Invalid key length"))?;
        CryptoEngine::decrypt_data(&key_array, &encrypted_data)
            .map_err(|e| PyRuntimeError::new_err(format!("Decryption error: {}", e)))
    }

    #[staticmethod]
    fn generate_secure_random_bytes(length: usize) -> Vec<u8> {
        CryptoEngine::generate_secure_random_bytes(length)
    }

    #[staticmethod]
    fn generate_nonce() -> [u8; 16] {
        CryptoEngine::generate_nonce()
    }
}

/// Python wrapper for VisualEngine
#[pyclass]
pub struct PyVisualEngine {
    inner: VisualEngine,
}

#[pymethods]
impl PyVisualEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: VisualEngine::new(),
        }
    }

    fn encode_payload(&self, payload: &PyVisualPayload) -> PyResult<String> {
        self.inner.encode_payload(&payload.inner)
            .map_err(|e| PyRuntimeError::new_err(format!("Visual error: {}", e)))
    }

    fn encode_qr_code(&self, py: Python, data: Vec<u8>) -> PyResult<String> {
        py.allow_threads(|| {
            let code = qrcode::QrCode::new(&data)
                .map_err(|_| PyRuntimeError::new_err("QR code generation failed"))?;
            Ok(code.render::<qrcode::render::svg::Color>().build())
        })
    }

    fn decode_payload(&self, qr_data: Vec<u8>) -> PyResult<PyVisualPayload> {
        let payload = self.inner.decode_payload(&qr_data)
            .map_err(|e| PyRuntimeError::new_err(format!("Visual error: {}", e)))?;
        Ok(PyVisualPayload { inner: payload })
    }
}

/// Python wrapper for VisualPayload
#[pyclass]
#[derive(Clone)]
pub struct PyVisualPayload {
    inner: VisualPayload,
}

#[pymethods]
impl PyVisualPayload {
    #[new]
    fn new(session_id: [u8; 16], public_key: Vec<u8>, nonce: [u8; 16], signature: Vec<u8>) -> Self {
        Self {
            inner: VisualPayload {
                session_id,
                public_key,
                nonce,
                signature,
            },
        }
    }

    #[getter]
    fn session_id(&self) -> [u8; 16] {
        self.inner.session_id
    }

    #[getter]
    fn public_key(&self) -> Vec<u8> {
        self.inner.public_key.clone()
    }

    #[getter]
    fn nonce(&self) -> [u8; 16] {
        self.inner.nonce
    }

    #[getter]
    fn signature(&self) -> Vec<u8> {
        self.inner.signature.clone()
    }
}

/// Python wrapper for AudioEngine
#[pyclass]
pub struct PyAudioEngine {
    inner: AudioEngine,
}

#[pymethods]
impl PyAudioEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: AudioEngine::new(),
        }
    }

    fn send_data(&self, py: Python, data: Vec<u8>) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, return Ok since audio engine is not fully implemented
            Ok(())
        })
    }

    fn receive_data(&self, py: Python) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return empty data since audio engine is not fully implemented
            Ok(vec![])
        })
    }

    fn is_receiving(&self, py: Python) -> bool {
        py.allow_threads(|| false)
    }
}

/// Python wrapper for ProtocolEngine
#[pyclass]
pub struct PyProtocolEngine {
    inner: ProtocolEngine,
}

#[pymethods]
impl PyProtocolEngine {
    #[new]
    fn new() -> Self {
        Self {
            inner: ProtocolEngine::new(),
        }
    }

    fn initiate_handshake(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate handshake initiation
            Ok(())
        })
    }

    fn receive_nonce(&self, py: Python, nonce: Vec<u8>) -> PyResult<String> {
        py.allow_threads(|| {
            // For now, return a mock QR code
            Ok("<svg>Mock QR Code</svg>".to_string())
        })
    }

    fn process_qr_payload(&self, py: Python, qr_data: Vec<u8>) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate QR processing
            Ok(())
        })
    }

    fn receive_ack(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate ACK reception
            Ok(())
        })
    }

    fn get_state(&self, py: Python) -> PyResult<String> {
        py.allow_threads(|| {
            Ok("idle".to_string())
        })
    }

    fn encrypt_message(&self, py: Python, data: Vec<u8>) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return data unchanged (no encryption)
            Ok(data)
        })
    }

    fn decrypt_message(&self, py: Python, encrypted_data: Vec<u8>) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return data unchanged (no decryption)
            Ok(encrypted_data)
        })
    }
}

/// Python wrapper for RgibberLink
#[pyclass]
pub struct PyRgibberLink {
    inner: RgibberLink,
}

#[pymethods]
impl PyRgibberLink {
    #[new]
    fn new() -> Self {
        Self {
            inner: RgibberLink::new(),
        }
    }

    fn initiate_handshake(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate handshake initiation
            Ok(())
        })
    }

    fn receive_nonce(&self, py: Python, nonce: Vec<u8>) -> PyResult<String> {
        py.allow_threads(|| {
            // For now, return a mock QR code
            Ok("<svg>Mock QR Code</svg>".to_string())
        })
    }

    fn process_qr_payload(&self, py: Python, qr_data: Vec<u8>) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate QR processing
            Ok(())
        })
    }

    fn receive_ack(&self, py: Python) -> PyResult<()> {
        py.allow_threads(|| {
            // For now, simulate ACK reception
            Ok(())
        })
    }

    fn get_state(&self, py: Python) -> PyResult<String> {
        py.allow_threads(|| {
            Ok("idle".to_string())
        })
    }

    fn encrypt_message(&self, py: Python, data: Vec<u8>) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return data unchanged (no encryption)
            Ok(data)
        })
    }

    fn decrypt_message(&self, py: Python, encrypted_data: Vec<u8>) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            // For now, return data unchanged (no decryption)
            Ok(encrypted_data)
        })
    }
}