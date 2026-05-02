package com.yourdomain.agent

import android.content.Context
import android.util.Log
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.io.File
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL

data class ModelDownload(
    val id: String,
    val name: String,
    val url: String,
    val sizeMB: Int,
    val status: DownloadStatus = DownloadStatus.NOT_DOWNLOADED,
    val progress: Float = 0f,
)

enum class DownloadStatus {
    NOT_DOWNLOADED, DOWNLOADING, DOWNLOADED, FAILED
}

class VoiceModelManager(private val context: Context) {

    private val _models = MutableStateFlow(
        listOf(
            ModelDownload("whisper-tiny", "Whisper Tiny (75 MB)", WhisperModel.TINY.downloadUrl, 75),
            ModelDownload("whisper-base", "Whisper Base (142 MB)", WhisperModel.BASE.downloadUrl, 142),
            ModelDownload("whisper-small", "Whisper Small (466 MB)", WhisperModel.SMALL.downloadUrl, 466),
            ModelDownload("piper-low", "Piper Lessac Low (20 MB)", PiperVoice.LOW.downloadUrl, 20),
            ModelDownload("piper-medium", "Piper Lessac Medium (65 MB)", PiperVoice.MEDIUM.downloadUrl, 65),
        )
    )
    val models: StateFlow<List<ModelDownload>> = _models.asStateFlow()

    private val modelsDir: File
        get() = File(context.filesDir, "voice_models").also { it.mkdirs() }

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    fun downloadModel(modelId: String) {
        val index = _models.value.indexOfFirst { it.id == modelId }
        if (index == -1) return

        val model = _models.value[index]
        updateStatus(index, DownloadStatus.DOWNLOADING, 0f)

        scope.launch {
            try {
                val outputFile = File(modelsDir, "${model.id}.bin")
                val url = URL(model.url)
                val connection = url.openConnection() as HttpURLConnection
                connection.connect()

                val totalSize = connection.contentLength
                val inputStream = connection.inputStream
                val outputStream = FileOutputStream(outputFile)

                val buffer = ByteArray(8192)
                var downloaded = 0L
                var bytesRead: Int

                while (inputStream.read(buffer).also { bytesRead = it } != -1) {
                    outputStream.write(buffer, 0, bytesRead)
                    downloaded += bytesRead
                    if (totalSize > 0) {
                        val progress = downloaded.toFloat() / totalSize.toFloat()
                        updateStatus(index, DownloadStatus.DOWNLOADING, progress)
                    }
                }

                outputStream.close()
                inputStream.close()
                updateStatus(index, DownloadStatus.DOWNLOADED, 1f)
                Log.d("VoiceModel", "Downloaded ${model.name} -> ${outputFile.absolutePath}")
            } catch (e: Exception) {
                Log.e("VoiceModel", "Download failed: ${e.message}")
                updateStatus(index, DownloadStatus.FAILED, 0f)
            }
        }
    }

    fun deleteModel(modelId: String) {
        val file = File(modelsDir, "${modelId}.bin")
        file.delete()
        val index = _models.value.indexOfFirst { it.id == modelId }
        if (index != -1) updateStatus(index, DownloadStatus.NOT_DOWNLOADED, 0f)
    }

    fun isDownloaded(modelId: String): Boolean {
        val file = File(modelsDir, "${modelId}.bin")
        return file.exists() && file.length() > 0
    }

    fun getModelPath(modelId: String): String? {
        val file = File(modelsDir, "${modelId}.bin")
        return if (file.exists()) file.absolutePath else null
    }

    fun getTotalDownloadedMB(): Long {
        return modelsDir.listFiles()?.sumOf { it.length() / (1024 * 1024) } ?: 0
    }

    private fun updateStatus(index: Int, status: DownloadStatus, progress: Float) {
        val updated = _models.value.toMutableList()
        updated[index] = updated[index].copy(status = status, progress = progress)
        _models.value = updated
    }

    fun refreshStatus() {
        val updated = _models.value.map { model ->
            if (isDownloaded(model.id)) {
                model.copy(status = DownloadStatus.DOWNLOADED, progress = 1f)
            } else {
                model.copy(status = DownloadStatus.NOT_DOWNLOADED, progress = 0f)
            }
        }
        _models.value = updated
    }
}
