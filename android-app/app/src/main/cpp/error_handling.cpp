#include "error_handling.hpp"
#include <android/log.h>
#include <string>
#include <exception>

// Logging macros
#define LOG_TAG "RgibberLinkJNI"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)
#define LOGW(...) __android_log_print(ANDROID_LOG_WARN, LOG_TAG, __VA_ARGS__)

// Error handling utilities

void log_error(const std::string& operation, const std::exception& e) {
    LOGE("%s failed: %s", operation.c_str(), e.what());
}

void log_error(const std::string& operation, const char* message) {
    LOGE("%s failed: %s", operation.c_str(), message ? message : "unknown error");
}

void log_info(const std::string& message) {
    LOGI("%s", message.c_str());
}

void log_warning(const std::string& message) {
    LOGW("%s", message.c_str());
}

// JNI error handling wrapper
bool safe_jni_call(const std::string& operation, std::function<bool()> func) {
    try {
        return func();
    } catch (const std::exception& e) {
        log_error(operation, e);
        return false;
    } catch (...) {
        log_error(operation, "Unknown exception");
        return false;
    }
}