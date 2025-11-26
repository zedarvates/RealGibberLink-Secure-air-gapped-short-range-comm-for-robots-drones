#ifndef UTILITIES_HPP
#define UTILITIES_HPP

#include <jni.h>
#include <string>
#include <vector>

// Hardware capability detection
extern "C" {
    uint8_t* detect_hardware_capabilities(size_t* out_len);
    bool check_ultrasonic_hardware_available();
    bool check_laser_hardware_available();
    bool check_photodiode_hardware_available();
    bool check_camera_hardware_available();
}

// JNI utility functions
void register_hardware_event_callback(JNIEnv* env, jobject callback);
void unregister_hardware_event_callback(JNIEnv* env);

#endif // UTILITIES_HPP