# Keep JNI methods
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep Rust JNI exports
-keep class com.yourdomain.agent.AgentCore { *; }
-keep class com.yourdomain.agent.AgentJNI { *; }

# Compose
-dontwarn androidx.compose.**
-keep class androidx.compose.** { *; }

# OkHttp
-dontwarn okhttp3.**
-dontwarn okio.**
-keep class okhttp3.** { *; }

# Coroutines
-keepnames class kotlinx.coroutines.internal.MainDispatcherFactory {}
-keepnames class kotlinx.coroutines.CoroutineExceptionHandler {}

# Keep data classes used with Gson/serialization
-keepattributes Signature
-keepattributes *Annotation*
-keep class com.yourdomain.agent.** { *; }

# Accessibility service
-keep class com.yourdomain.agent.AgentAccessibilityService { *; }

# Default
-keepattributes InnerClasses
-keepattributes EnclosingMethod
