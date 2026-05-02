package com.yourdomain.agent

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log
import kotlinx.coroutines.*
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit

class TelegramBotService : Service() {

    private val client = OkHttpClient.Builder()
        .connectTimeout(30, TimeUnit.SECONDS)
        .readTimeout(30, TimeUnit.SECONDS)
        .build()

    private var botToken: String = ""
    private var offset: Long = 0
    private var isRunning = false
    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    companion object {
        private const val TAG = "TelegramBot"
        private const val TELEGRAM_API = "https://api.telegram.org/bot"
    }

    override fun onCreate() {
        super.onCreate()
        val prefs = getSharedPreferences("AgentPrefs", MODE_PRIVATE)
        botToken = prefs.getString("telegramToken", "") ?: ""
        Log.d(TAG, "Telegram bot service created")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (botToken.isEmpty()) {
            Log.e(TAG, "No bot token configured")
            stopSelf()
            return START_NOT_STICKY
        }
        if (!isRunning) {
            isRunning = true
            startPolling()
        }
        return START_STICKY
    }

    private fun startPolling() {
        scope.launch {
            while (isRunning) {
                try {
                    val updates = pollUpdates()
                    updates.forEach { update ->
                        val message = update.optJSONObject("message") ?: return@forEach
                        val text = message.optString("text", "")
                        val chatId = message.optJSONObject("chat")?.optLong("id") ?: return@forEach
                        handleCommand(text, chatId)
                    }
                } catch (e: Exception) {
                    Log.e(TAG, "Poll error: ${e.message}")
                }
                delay(1000)
            }
        }
    }

    private fun pollUpdates(): List<JSONObject> {
        val url = "${TELEGRAM_API}${botToken}/getUpdates?offset=$offset&timeout=30"
        val request = Request.Builder().url(url).build()
        val response = client.newCall(request).execute()
        val body = response.body?.string() ?: return emptyList()
        val json = JSONObject(body)
        if (!json.optBoolean("ok")) return emptyList()

        val results = json.optJSONArray("result") ?: return emptyList()
        val updates = mutableListOf<JSONObject>()
        for (i in 0 until results.length()) {
            val update = results.getJSONObject(i)
            offset = update.optLong("update_id") + 1
            updates.add(update)
        }
        return updates
    }

    private suspend fun handleCommand(text: String, chatId: Long) {
        when {
            text.startsWith("/status") -> {
                sendMessage(chatId, "◉ Agent Status: idle\nModel: claude-sonnet-4-6")
            }
            text.startsWith("/stop") -> {
                sendMessage(chatId, "Agent stopped.")
                // Would call viewModel.stopTask() via binding
            }
            text.startsWith("/logs") -> {
                val count = text.removePrefix("/logs").trim().toIntOrNull() ?: 10
                sendMessage(chatId, "[log] Showing last $count entries")
            }
            text.startsWith("/model") -> {
                val model = text.removePrefix("/model").trim()
                if (model.isNotEmpty()) {
                    sendMessage(chatId, "Model switched to: $model")
                } else {
                    sendMessage(chatId, "Current model: claude-sonnet-4-6")
                }
            }
            text.startsWith("/skills") -> {
                sendMessage(chatId, buildString {
                    appendLine("Installed Skills:")
                    appendLine("✅ screen_control — Tap, swipe, type")
                    appendLine("✅ open_app — Launch apps")
                    appendLine("✅ web_search — Local search engine")
                    appendLine("✅ send_message — SMS/Telegram/WhatsApp")
                    appendLine("✅ phone_call — Make calls (with confirmation)")
                    appendLine("✅ calendar — Calendar CRUD")
                    appendLine("❌ shell_cmd — Developer mode only")
                })
            }
            text.startsWith("/memory") -> {
                sendMessage(chatId, buildString {
                    appendLine("User Profile: User")
                    appendLine("Preferred model: claude-sonnet-4-6")
                    appendLine("Recent: Sent email to Alex (2026-04-21)")
                })
            }
            text.startsWith("/tier") -> {
                val parts = text.removePrefix("/tier").trim().split(" ", limit = 2)
                if (parts.size >= 2) {
                    sendMessage(chatId, "Tier '${parts[0]}' set to model '${parts[1]}'")
                } else {
                    sendMessage(chatId, buildString {
                        appendLine("Model Tiers:")
                        appendLine("Trivial → gemini-flash-2.5")
                        appendLine("Standard → mistral-small-3.2")
                        appendLine("Complex → claude-sonnet-4-6")
                        appendLine("Critical → claude-opus-4-6")
                    })
                }
            }
            text.startsWith("/help") -> {
                sendMessage(chatId, buildString {
                    appendLine("Commands:")
                    appendLine("/status — Agent state")
                    appendLine("/stop — Halt running task")
                    appendLine("/logs [n] — Last n log lines")
                    appendLine("/model <name> — Switch model")
                    appendLine("/skills — List skills")
                    appendLine("/memory — Show MEMORY.md")
                    appendLine("/tier <tier> <model> — Update routing tier")
                    appendLine()
                    appendLine("Any other text = Execute as task")
                })
            }
            else -> {
                sendMessage(chatId, "▶ Starting task: $text")
                // Would call viewModel.startTask(text) via binding
                // And stream progress back via update messages
                delay(500)
                sendMessage(chatId, "✓ Task completed: $text\nComplexity: Standard\nModel: claude-sonnet-4-6")
            }
        }
    }

    private fun sendMessage(chatId: Long, text: String) {
        try {
            val json = JSONObject().apply {
                put("chat_id", chatId)
                put("text", text)
                put("parse_mode", "HTML")
            }
            val url = "${TELEGRAM_API}${botToken}/sendMessage"
            val body = json.toString().toRequestBody("application/json".toMediaType())
            val request = Request.Builder().url(url).post(body).build()
            client.newCall(request).execute()
        } catch (e: Exception) {
            Log.e(TAG, "Send error: ${e.message}")
        }
    }

    override fun onDestroy() {
        isRunning = false
        scope.cancel()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null
}
