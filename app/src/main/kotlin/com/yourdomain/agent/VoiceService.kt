package com.yourdomain.agent

import android.app.Service
import android.content.Intent
import android.content.SharedPreferences
import android.media.AudioAttributes
import android.media.AudioFormat
import android.media.AudioRecord
import android.media.AudioTrack
import android.media.MediaRecorder
import android.os.Build
import android.os.IBinder
import android.speech.tts.TextToSpeech
import android.util.Log
import kotlinx.coroutines.*
import java.io.File
import java.util.Locale

class VoiceService : Service() {

    private lateinit var tts: TextToSpeech
    private lateinit var prefs: SharedPreferences
    private var isListening = false
    private var isRecording = false
    private var wakeWord: String = "hey agent"
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    // Audio recording for wake word + STT
    private var audioRecord: AudioRecord? = null
    private val sampleRate = 16000
    private val bufferSize = AudioRecord.getMinBufferSize(
        sampleRate,
        AudioFormat.CHANNEL_IN_MONO,
        AudioFormat.ENCODING_PCM_16BIT
    )

    companion object {
        private const val TAG = "VoiceService"

        // Broadcast actions
        const val ACTION_START_LISTENING = "com.yourdomain.agent.START_LISTENING"
        const val ACTION_STOP_LISTENING = "com.yourdomain.agent.STOP_LISTENING"
        const val ACTION_WAKE_WORD_DETECTED = "com.yourdomain.agent.WAKE_WORD_DETECTED"
        const val ACTION_SPEECH_RECOGNIZED = "com.yourdomain.agent.SPEECH_RECOGNIZED"
        const val ACTION_TTS_SPEAK = "com.yourdomain.agent.TTS_SPEAK"
    }

    override fun onCreate() {
        super.onCreate()
        prefs = getSharedPreferences("AgentPrefs", MODE_PRIVATE)
        wakeWord = prefs.getString("wakeWord", "hey agent") ?: "hey agent"

        initTts()
        Log.d(TAG, "Voice service created. Wake word: '$wakeWord'")
    }

    // ── TTS ────────────────────────────────────────────────────────────────────

    private fun initTts() {
        tts = TextToSpeech(this) { status ->
            if (status == TextToSpeech.SUCCESS) {
                tts.language = Locale.US
                tts.setSpeechRate(1.0f)
                tts.setPitch(1.0f)

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
                    tts.setAudioAttributes(
                        AudioAttributes.Builder()
                            .setUsage(AudioAttributes.USAGE_ASSISTANT)
                            .setContentType(AudioAttributes.CONTENT_TYPE_SPEECH)
                            .build()
                    )
                }
                Log.d(TAG, "TTS initialized")
            } else {
                Log.e(TAG, "TTS init failed: $status")
            }
        }
    }

    fun speak(text: String) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
            tts.speak(text, TextToSpeech.QUEUE_FLUSH, null, "agent_response_${System.currentTimeMillis()}")
        } else {
            tts.speak(text, TextToSpeech.QUEUE_FLUSH, null)
        }
    }

    // ── Wake Word Detection ─────────────────────────────────────────────────────

    private suspend fun startWakeWordDetection() {
        isListening = true

        // In production: use Porcupine SDK for real wake word detection
        // PorcupineManager.Builder()
        //     .setAccessKey("YOUR_PICOVOICE_KEY")
        //     .setKeyword(Porcupine.BuiltInKeyword.HEY_GOOGLE)
        //     .build(context) { keywordIndex ->
        //         onWakeWordDetected()
        //     }

        // Fallback: simple energy-based detection as placeholder
        withContext(Dispatchers.IO) {
            audioRecord = AudioRecord(
                MediaRecorder.AudioSource.MIC,
                sampleRate,
                AudioFormat.CHANNEL_IN_MONO,
                AudioFormat.ENCODING_PCM_16BIT,
                bufferSize
            )

            if (audioRecord?.state != AudioRecord.STATE_INITIALIZED) {
                Log.e(TAG, "AudioRecord init failed")
                return@withContext
            }

            audioRecord?.startRecording()
            val buffer = ShortArray(bufferSize)

            while (isListening && isActive) {
                val read = audioRecord?.read(buffer, 0, buffer.size) ?: -1
                if (read > 0) {
                    // Bolt ⚡ Optimization: Calculate energy with a manual loop instead of .take().map().average()
                    // to avoid allocating intermediate collections in this hot audio processing loop.
                    var sumSquares = 0.0
                    for (i in 0 until read) {
                        val sample = buffer[i].toDouble()
                        sumSquares += sample * sample
                    }
                    val energy = sumSquares / read
                    // Simple threshold — Porcupine SDK replaces this
                    if (energy > 1000.0) {
                        onWakeWordDetected()
                    }
                }
            }
        }
    }

    private suspend fun onWakeWordDetected() {
        Log.d(TAG, "Wake word detected!")
        isRecording = true
        sendBroadcast(Intent(ACTION_WAKE_WORD_DETECTED))

        // Speak acknowledgment
        withContext(Dispatchers.Main) {
            speak("Yes?")
        }

        // Start STT recording (5-second window)
        delay(500) // brief pause after wake word

        val recordedPcm = recordSpeech(timeoutMs = 5000)
        if (recordedPcm != null && isRecording) {
            val recognizedText = transcribePcm(recordedPcm)
            if (recognizedText.isNotBlank()) {
                Log.d(TAG, "Recognized: $recognizedText")
                val intent = Intent(ACTION_SPEECH_RECOGNIZED).apply {
                    putExtra("text", recognizedText)
                }
                sendBroadcast(intent)
            }
        }
        isRecording = false
    }

    private fun recordSpeech(timeoutMs: Int): ShortArray? {
        val chunks = mutableListOf<Short>()
        val buffer = ShortArray(bufferSize)
        val startTime = System.currentTimeMillis()

        audioRecord?.startRecording()
        while (System.currentTimeMillis() - startTime < timeoutMs && isRecording) {
            val read = audioRecord?.read(buffer, 0, buffer.size) ?: -1
            if (read > 0) {
                chunks.addAll(buffer.take(read))
            }
        }

        return if (chunks.isNotEmpty()) chunks.toShortArray() else null
    }

    // ── STT (Whisper.cpp) ──────────────────────────────────────────────────────

    private fun transcribePcm(pcm: ShortArray): String {
        // In production: call whisper.cpp via JNI
        // val modelPath = File(filesDir, "models/ggml-base.en.bin").absolutePath
        // val result = WhisperNative.transcribe(modelPath, pcm)

        // Fallback: return simulated result
        // Bolt ⚡ Optimization: Calculate energy without allocating collections
        var sumSquares = 0.0
        for (sample in pcm) {
            val dSample = sample.toDouble()
            sumSquares += dSample * dSample
        }
        val energy = if (pcm.isNotEmpty()) sumSquares / pcm.size else 0.0
        return if (energy > 500.0) {
            "What time is it?"
        } else {
            ""
        }
    }

    // ── Service Lifecycle ───────────────────────────────────────────────────────

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START_LISTENING -> {
                if (!isListening) {
                    scope.launch { startWakeWordDetection() }
                }
            }
            ACTION_STOP_LISTENING -> {
                isListening = false
                stopRecording()
            }
            ACTION_TTS_SPEAK -> {
                val text = intent.getStringExtra("text") ?: ""
                if (text.isNotBlank()) speak(text)
            }
        }
        return START_STICKY
    }

    private fun stopRecording() {
        audioRecord?.apply {
            if (recordingState == AudioRecord.RECORDSTATE_RECORDING) {
                stop()
            }
            release()
        }
        audioRecord = null
        isRecording = false
    }

    override fun onDestroy() {
        isListening = false
        stopRecording()
        tts.stop()
        tts.shutdown()
        scope.cancel()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}
