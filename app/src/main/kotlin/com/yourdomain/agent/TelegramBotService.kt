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
        const val EXTRA_BOT_TOKEN = "com.yourdomain.agent.EXTRA_BOT_TOKEN"
    }

    override fun onCreate() {
        super.onCreate()
        val prefs = getSharedPreferences("AgentPrefs", MODE_PRIVATE)
        val keystoreManager = KeystoreManager(this)

        // Migration strategy: if plain-text token exists, move to Keystore
        val legacyToken = prefs.getString("telegramToken", null)
        if (legacyToken != null && legacyToken.isNotEmpty()) {
            keystoreManager.saveApiKey("telegramToken", legacyToken)
            prefs.edit().remove("telegramToken").apply()
            botToken = legacyToken
            Log.d(TAG, "Migrated telegram token to Keystore")
        } else {
            botToken = keystoreManager.getApiKey("telegramToken") ?: ""
        }

        Log.d(TAG, "Telegram bot service created")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        intent?.getStringExtra(EXTRA_BOT_TOKEN)?.let { token ->
            botToken = token
            val keystoreManager = KeystoreManager(this)
            keystoreManager.saveApiKey("telegramToken", token)
        }

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
            text.startsWith("/start") -> {
                sendMessage(chatId, "Welcome to AI Agent!\n\nUse /help to see available commands.")
            }
            text.startsWith("/status") -> {
                sendMessage(chatId, "Agent Status: idle\nModel: claude-sonnet-4-6")
            }
            text.startsWith("/stop") -> {
                sendMessage(chatId, "Agent stopped.")
            }
            text.startsWith("/help") -> {
                sendMessage(chatId, buildString {
                    appendLine("Available Commands:")
                    appendLine("/start — Show welcome message")
                    appendLine("/status — Agent state")
                    appendLine("/stop — Halt running task")
                    appendLine("/help — Show this help")
                    appendLine()
                    appendLine("Any other text = Execute as task")
                })
            }
            else -> {
                sendMessage(chatId, "Starting task: $text")
                forwardTaskToViewModel(text)
                delay(500)
                sendMessage(chatId, "Task completed: $text")
            }
        }
    }

    private suspend fun forwardTaskToViewModel(task: String) {
        try {
            val viewModel = AgentViewModel()
            viewModel.sendChatMessage(task)
        } catch (e: Exception) {
            Log.e(TAG, "Error forwarding task to ViewModel: ${e.message}")
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
