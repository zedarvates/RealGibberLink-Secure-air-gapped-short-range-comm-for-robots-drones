#include "utilities.hpp"
#include <android/log.h>
#include <jni.h>
#include <atomic>
#include <string>

// Logging macros
#define LOG_TAG "RgibberLinkJNI"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// Global callback object for hardware events
static std::atomic<jobject> g_callback_object(nullptr);
static JavaVM* g_java_vm = nullptr;

// JNI utility functions implementation

jbyteArray create_byte_array(JNIEnv* env, const uint8_t* data, size_t len) {
    if (!data || len == 0) return nullptr;

    jbyteArray result = env->NewByteArray(len);
    if (!result) return nullptr;

    env->SetByteArrayRegion(result, 0, len, reinterpret_cast<const jbyte*>(data));
    return result;
}

std::vector<uint8_t> get_byte_array_data(JNIEnv* env, jbyteArray array) {
    if (!array) return {};

    jsize len = env->GetArrayLength(array);
    if (len <= 0) return {};

    std::vector<uint8_t> result(len);
    env->GetByteArrayRegion(array, 0, len, reinterpret_cast<jbyte*>(result.data()));
    return result;
}

jstring create_string(JNIEnv* env, const char* str) {
    if (!str) return nullptr;
    return env->NewStringUTF(str);
}

// Hardware capability detection implementations

extern "C" JNIEXPORT jbyteArray JNICALL
Java_com_Rgibberlink_RgibberLinkJNI_detectHardwareCapabilities(JNIEnv* env, jobject /* this */) {
    try {
        size_t out_len = 0;
        uint8_t* result = detect_hardware_capabilities(&out_len);

        if (!result) return nullptr;

        jbyteArray array = create_byte_array(env, result, out_len);
        gibberlink_free_data(result);
        return array;
    } catch (const std::exception& e) {
        LOGE("Detect hardware capabilities failed: %s", e.what());
        return nullptr;
    }
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_Rgibberlink_RgibberLinkJNI_checkUltrasonicHardwareAvailable(JNIEnv* env, jobject /* this */) {
    try {
        return check_ultrasonic_hardware_available() ? JNI_TRUE : JNI_FALSE;
    } catch (const std::exception& e) {
        LOGE("Check ultrasonic hardware failed: %s", e.what());
        return JNI_FALSE;
    }
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_Rgibberlink_RgibberLinkJNI_checkLaserHardwareAvailable(JNIEnv* env, jobject /* this */) {
    try {
        return check_laser_hardware_available() ? JNI_TRUE : JNI_FALSE;
    } catch (const std::exception& e) {
        LOGE("Check laser hardware failed: %s", e.what());
        return JNI_FALSE;
    }
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_Rgibberlink_RgibberLinkJNI_checkPhotodiodeHardwareAvailable(JNIEnv* env, jobject /* this */) {
    try {
        return check_photodiode_hardware_available() ? JNI_TRUE : JNI_FALSE;
    } catch (const std::exception& e) {
        LOGE("Check photodiode hardware failed: %s", e.what());
        return JNI_FALSE;
    }
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_Rgibberlink_RgibberLinkJNI_checkCameraHardwareAvailable(JNIEnv* env, jobject /* this */) {
    try {
        return check_camera_hardware_available() ? JNI_TRUE : JNI_FALSE;
    } catch (const std::exception& e) {
        LOGE("Check camera hardware failed: %s", e.what());
        return JNI_FALSE;
    }
}

// Hardware event callback management

void register_hardware_event_callback(JNIEnv* env, jobject callback) {
    // Store global reference to callback object
    if (g_callback_object != nullptr) {
        env->DeleteGlobalRef(g_callback_object.load());
    }

    if (callback) {
        g_callback_object = env->NewGlobalRef(callback);
    } else {
        g_callback_object = nullptr;
    }
}

void unregister_hardware_event_callback(JNIEnv* env) {
    if (g_callback_object != nullptr) {
        env->DeleteGlobalRef(g_callback_object.load());
        g_callback_object = nullptr;
    }
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_Rgibberlink_RgibberLinkJNI_registerHardwareEventCallback(JNIEnv* env, jobject /* this */, jobject callback) {
    register_hardware_event_callback(env, callback);
    return JNI_TRUE;
}

extern "C" JNIEXPORT jboolean JNICALL
Java_com_Rgibberlink_RgibberLinkJNI_unregisterHardwareEventCallback(JNIEnv* env, jobject /* this */) {
    unregister_hardware_event_callback(env);
    return JNI_TRUE;
}