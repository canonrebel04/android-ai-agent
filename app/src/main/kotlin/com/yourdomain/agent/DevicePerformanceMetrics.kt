package com.yourdomain.agent

import android.app.ActivityManager
import android.content.Context
import android.os.Build
import android.os.Environment
import android.os.StatFs
import java.io.File

data class DeviceSpecs(
    val totalRamMB: Long,
    val availableRamMB: Long,
    val cpuCores: Int,
    val cpuArch: String,
    val freeStorageMB: Long,
    val androidVersion: String,
    val manufacturer: String,
    val model: String,
) {
    /** Whisper STT model recommendation based on available RAM */
    fun recommendWhisperModel(): WhisperModel {
        return when {
            totalRamMB >= 12000 -> WhisperModel.SMALL      // 852 MB RAM needed, plenty
            totalRamMB >= 8000  -> WhisperModel.BASE        // 388 MB RAM, best tradeoff
            totalRamMB >= 4000  -> WhisperModel.TINY        // 273 MB RAM, works
            else                -> WhisperModel.TINY        // minimum viable
        }
    }

    /** Piper TTS voice recommendation based on available RAM */
    fun recommendTtsVoice(): PiperVoice {
        return when {
            totalRamMB >= 8000  -> PiperVoice.MEDIUM       // ~65 MB, natural
            else                -> PiperVoice.LOW            // ~20 MB, acceptable
        }
    }

    /** Estimated performance tier */
    fun performanceTier(): PerformanceTier {
        return when {
            totalRamMB >= 12000 && cpuCores >= 8 -> PerformanceTier.HIGH
            totalRamMB >= 6000 && cpuCores >= 6  -> PerformanceTier.MEDIUM
            else                                  -> PerformanceTier.LOW
        }
    }

    fun summary(): String = buildString {
        appendLine("Device: $manufacturer $model")
        appendLine("Android: $androidVersion | CPU: $cpuCores cores ($cpuArch)")
        appendLine("RAM: ${totalRamMB}MB total | ${freeStorageMB}MB free storage")
        appendLine("Performance tier: ${performanceTier().label}")
        appendLine()
        appendLine("Recommended models:")
        appendLine("  STT (Whisper): ${recommendWhisperModel().label}")
        appendLine("  TTS (Piper):   ${recommendTtsVoice().label}")
    }
}

enum class WhisperModel(
    val label: String,
    val diskMB: Int,
    val ramMB: Int,
    val speedVsLarge: Float,
    val accuracyWER: Float,
    val downloadUrl: String,
) {
    TINY(
        label = "Tiny (39M params, 75 MB)",
        diskMB = 75, ramMB = 273, speedVsLarge = 10f, accuracyWER = 7.6f,
        downloadUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin"
    ),
    BASE(
        label = "Base (74M params, 142 MB)",
        diskMB = 142, ramMB = 388, speedVsLarge = 7f, accuracyWER = 5.0f,
        downloadUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin"
    ),
    SMALL(
        label = "Small (244M params, 466 MB)",
        diskMB = 466, ramMB = 852, speedVsLarge = 4f, accuracyWER = 3.4f,
        downloadUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin"
    ),
    TURBO(
        label = "Turbo (809M params, 547 MB q5)",
        diskMB = 547, ramMB = 2300, speedVsLarge = 8f, accuracyWER = 2.5f,
        downloadUrl = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin"
    ),
}

enum class PiperVoice(
    val label: String,
    val diskMB: Int,
    val quality: String,
    val downloadUrl: String,
) {
    LOW(
        label = "Lessac Low (~20 MB, US English)",
        diskMB = 20, quality = "Good",
        downloadUrl = "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/low/en_US-lessac-low.onnx"
    ),
    MEDIUM(
        label = "Lessac Medium (~65 MB, US English)",
        diskMB = 65, quality = "Natural",
        downloadUrl = "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx"
    ),
}

enum class PerformanceTier(val label: String) {
    HIGH("High — all models supported"),
    MEDIUM("Medium — base/small Whisper recommended"),
    LOW("Low — tiny Whisper recommended"),
}

object DevicePerformanceMetrics {

    fun measure(context: Context): DeviceSpecs {
        val activityManager = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
        val memInfo = ActivityManager.MemoryInfo()
        activityManager.getMemoryInfo(memInfo)

        val totalRamMB = memInfo.totalMem / (1024 * 1024)
        val availableRamMB = memInfo.availMem / (1024 * 1024)
        val cpuCores = Runtime.getRuntime().availableProcessors()
        val cpuArch = Build.SUPPORTED_ABIS.firstOrNull() ?: "unknown"

        val statFs = StatFs(Environment.getDataDirectory().path)
        val freeStorageMB = (statFs.availableBytes / (1024 * 1024))

        return DeviceSpecs(
            totalRamMB = totalRamMB,
            availableRamMB = availableRamMB,
            cpuCores = cpuCores,
            cpuArch = cpuArch,
            freeStorageMB = freeStorageMB,
            androidVersion = Build.VERSION.RELEASE,
            manufacturer = Build.MANUFACTURER,
            model = Build.MODEL,
        )
    }

    fun canRunModel(model: WhisperModel, specs: DeviceSpecs): Boolean {
        return specs.availableRamMB > model.ramMB && specs.freeStorageMB > model.diskMB
    }

    fun canRunVoice(voice: PiperVoice, specs: DeviceSpecs): Boolean {
        return specs.freeStorageMB > voice.diskMB
    }
}
