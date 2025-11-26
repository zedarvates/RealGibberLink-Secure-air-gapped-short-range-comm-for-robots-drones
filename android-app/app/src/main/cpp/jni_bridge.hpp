#ifndef JNI_BRIDGE_HPP
#define JNI_BRIDGE_HPP

#include <jni.h>
#include <string>
#include <vector>
#include <memory>
#include <mutex>

// Thread safety helpers
class JNIGuard {
private:
    std::mutex& mutex_;
    bool locked_;

public:
    explicit JNIGuard(std::mutex& mutex) : mutex_(mutex), locked_(false) {
        mutex_.lock();
        locked_ = true;
    }

    ~JNIGuard() {
        if (locked_) {
            mutex_.unlock();
        }
    }
};

// Global mutexes for thread safety
extern std::mutex g_protocol_mutex;
extern std::mutex g_ultrasonic_mutex;
extern std::mutex g_laser_mutex;
extern std::mutex g_range_detector_mutex;
extern std::mutex g_hardware_mutex;

// JNI utility functions
jbyteArray create_byte_array(JNIEnv* env, const uint8_t* data, size_t len);
std::vector<uint8_t> get_byte_array_data(JNIEnv* env, jbyteArray array);
jstring create_string(JNIEnv* env, const char* str);

// Hardware event callback
extern std::atomic<jobject> g_callback_object;
extern JavaVM* g_java_vm;

#endif // JNI_BRIDGE_HPP