//! Hardware interface for laser communication

#[cfg(target_os = "android")]
use std::os::raw::{c_char, c_int};

#[cfg(target_os = "android")]
extern "C" {
    fn laser_init_hardware() -> c_int;
    fn laser_set_power(power_mw: f32) -> c_int;
    fn laser_get_photodiode_reading() -> f32;
    fn laser_get_camera_frame(buffer: *mut u8, size: usize) -> c_int;
    fn laser_set_alignment(x: f32, y: f32) -> c_int;
}

pub struct HardwareInterface;

impl HardwareInterface {
    pub fn new() -> Self {
        Self
    }

    pub fn initialize(&self) -> Result<(), super::error::LaserError> {
        #[cfg(target_os = "android")]
        {
            let result = unsafe { laser_init_hardware() };
            if result != 0 {
                return Err(super::error::LaserError::HardwareUnavailable);
            }
        }

        // For non-Android platforms, this is a no-op as hardware may not be available
        Ok(())
    }

    pub fn set_power(&self, power_mw: f32) -> Result<(), super::error::LaserError> {
        #[cfg(target_os = "android")]
        {
            let result = unsafe { laser_set_power(power_mw) };
            if result != 0 {
                return Err(super::error::LaserError::TransmissionFailed);
            }
        }

        #[cfg(not(target_os = "android"))]
        {
            // Mock implementation for non-Android platforms
            // In a real implementation, this would interface with the laser hardware
            let _ = power_mw; // Suppress unused variable warning
        }

        Ok(())
    }

    pub fn get_photodiode_reading(&self) -> f32 {
        #[cfg(target_os = "android")]
        {
            unsafe { laser_get_photodiode_reading() }
        }

        #[cfg(not(target_os = "android"))]
        {
            // Mock implementation - return a random-ish value for testing
            0.5
        }
    }

    pub fn get_camera_frame(&self, buffer: &mut [u8]) -> Result<(), super::error::LaserError> {
        #[cfg(target_os = "android")]
        {
            let result = unsafe { laser_get_camera_frame(buffer.as_mut_ptr(), buffer.len()) };
            if result != 0 {
                return Err(super::error::LaserError::ReceptionFailed);
            }
        }

        #[cfg(not(target_os = "android"))]
        {
            // Mock implementation - fill buffer with test data
            for (i, byte) in buffer.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }
        }

        Ok(())
    }

    pub fn set_alignment(&self, x: f32, y: f32) -> Result<(), super::error::LaserError> {
        #[cfg(target_os = "android")]
        {
            let result = unsafe { laser_set_alignment(x, y) };
            if result != 0 {
                return Err(super::error::LaserError::AlignmentLost);
            }
        }

        #[cfg(not(target_os = "android"))]
        {
            // Mock implementation
            let (_x, _y) = (x, y); // Suppress unused variable warnings
        }

        Ok(())
    }

    pub fn is_hardware_available(&self) -> bool {
        #[cfg(target_os = "android")]
        {
            // On Android, we assume hardware is available if the extern functions are linked
            true
        }

        #[cfg(not(target_os = "android"))]
        {
            // On other platforms, hardware may not be available
            false
        }
    }
}