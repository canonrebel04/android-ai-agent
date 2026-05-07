package com.yourdomain.agent

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.IBinder
import android.util.Log
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import org.json.JSONObject
import java.util.concurrent.TimeUnit

/**
 * WebSocketGateway — bidirectional communication channel for remote control.
 * 
 * Provides WebSocket server functionality for:
 * - Real-time agent control from external clients
 * - Event streaming (notifications, agent state changes)
 * - JSON-RPC style message exchange
 * 
 * Uses OkHttp's WebSocket client to connect to a WebSocket server.
 * In production, this would connect to a user-hosted server or a cloud service.
 */
class WebSocketGateway : Service() {

    private var webSocket: WebSocket? = null
    private var client: OkHttpClient? = null
    private var isConnected = false
    private var reconnectAttempts = 0
    private val maxReconnectAttempts = 5
    private val reconnectDelayMs = 3000L

    companion object {
        private const val TAG = "WebSocketGateway"
        const val CHANNEL_ID = "websocket_channel"
        const val NOTIFICATION_ID = 4001

        // Actions
        const val ACTION_CONNECT = "com.yourdomain.agent.WS_CONNECT"
        const val ACTION_DISCONNECT = "com.yourdomain.agent.WS_DISCONNECT"
        const val ACTION_SEND = "com.yourdomain.agent.WS_SEND"

        // Broadcast actions
        const val ACTION_CONNECTED = "com.yourdomain.agent.WS_CONNECTED"
        const val ACTION_DISCONNECTED = "com.yourdomain.agent.WS_DISCONNECTED"
        const val ACTION_MESSAGE_RECEIVED = "com.yourdomain.agent.WS_MESSAGE"
        const val ACTION_ERROR = "com.yourdomain.agent.WS_ERROR"

        // Intent extras
        const val EXTRA_URL = "url"
        const val EXTRA_MESSAGE = "message"
        const val EXTRA_ERROR = "error"
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        createClient()
        Log.d(TAG, "WebSocketGateway service created")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        intent?.action?.let { action ->
            when (action) {
                ACTION_CONNECT -> {
                    val url = intent.getStringExtra(EXTRA_URL)
                    if (url != null) {
                        connect(url)
                    } else {
                        Log.e(TAG, "No URL provided for connect action")
                        sendErrorBroadcast("No WebSocket URL provided")
                    }
                }
                ACTION_DISCONNECT -> {
                    disconnect()
                }
                ACTION_SEND -> {
                    val message = intent.getStringExtra(EXTRA_MESSAGE)
                    if (message != null && isConnected) {
                        sendMessage(message)
                    } else {
                        Log.e(TAG, "Cannot send message: not connected or no message")
                    }
                }
            }
        }
        return START_STICKY
    }

    private fun createClient() {
        client = OkHttpClient.Builder()
            .connectTimeout(10, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(30, TimeUnit.SECONDS)
            .pingInterval(15, TimeUnit.SECONDS)
            .retryOnConnectionFailure(true)
            .build()
    }

    private fun connect(url: String) {
        if (isConnected) {
            Log.d(TAG, "Already connected, disconnecting first")
            disconnect()
        }

        Log.d(TAG, "Connecting to WebSocket: $url")
        startForeground(NOTIFICATION_ID, buildNotification(url))

        val request = Request.Builder()
            .url(url)
            .build()

        webSocket = client?.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: okhttp3.Response) {
                super.onOpen(webSocket, response)
                isConnected = true
                reconnectAttempts = 0
                Log.d(TAG, "WebSocket connected: ${response.message}")
                sendConnectedBroadcast(url)
            }

            override fun onMessage(webSocket: WebSocket, text: String) {
                super.onMessage(webSocket, text)
                Log.d(TAG, "WebSocket message received: $text")
                sendMessageBroadcast(text)
            }

            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                super.onClosed(webSocket, code, reason)
                isConnected = false
                Log.d(TAG, "WebSocket closed: $code - $reason")
                sendDisconnectedBroadcast()
                attemptReconnect(url)
            }

            override fun onFailure(webSocket: WebSocket, t: Throwable, response: okhttp3.Response?) {
                super.onFailure(webSocket, t, response)
                isConnected = false
                Log.e(TAG, "WebSocket failure: ${t.message}", t)
                sendErrorBroadcast(t.message ?: "Unknown error")
                attemptReconnect(url)
            }
        })
    }

    private fun attemptReconnect(url: String) {
        if (reconnectAttempts < maxReconnectAttempts) {
            reconnectAttempts++
            Log.d(TAG, "Reconnect attempt $reconnectAttempts/$maxReconnectAttempts in ${reconnectDelayMs}ms")
            // Schedule reconnect
            val handler = android.os.Handler(mainLooper)
            handler.postDelayed({
                if (!isConnected) {
                    connect(url)
                }
            }, reconnectDelayMs)
        } else {
            Log.w(TAG, "Max reconnect attempts reached")
            stopSelf()
        }
    }

    private fun disconnect() {
        webSocket?.close(1000, "Normal closure")
        webSocket = null
        isConnected = false
        reconnectAttempts = 0
        Log.d(TAG, "WebSocket disconnected")
        stopForeground(true)
    }

    private fun sendMessage(message: String) {
        webSocket?.send(message)
        Log.d(TAG, "WebSocket message sent: $message")
    }

    /**
     * Send a structured JSON-RPC style message
     */
    fun sendJsonRpc(method: String, params: Map<String, Any>, id: Int = 1) {
        val json = JSONObject().apply {
            put("jsonrpc", "2.0")
            put("method", method)
            put("params", JSONObject(params.mapValues { it.value.toString() }.toMap()))
            put("id", id)
        }
        sendMessage(json.toString())
    }

    /**
     * Send an agent event to connected clients
     */
    fun broadcastEvent(eventType: String, data: Map<String, Any>) {
        val json = JSONObject().apply {
            put("type", "event")
            put("event", eventType)
            put("data", JSONObject(data.mapValues { it.value.toString() }.toMap()))
            put("timestamp", System.currentTimeMillis())
        }
        sendMessage(json.toString())
    }

    // ── Broadcast Helpers ────────────────────────────────────────────────────────────

    private fun sendConnectedBroadcast(url: String) {
        val intent = Intent(ACTION_CONNECTED).apply {
            putExtra(EXTRA_URL, url)
        }
        sendBroadcast(intent)
    }

    private fun sendDisconnectedBroadcast() {
        sendBroadcast(Intent(ACTION_DISCONNECTED))
    }

    private fun sendMessageBroadcast(message: String) {
        val intent = Intent(ACTION_MESSAGE_RECEIVED).apply {
            putExtra(EXTRA_MESSAGE, message)
        }
        sendBroadcast(intent)
    }

    private fun sendErrorBroadcast(error: String) {
        val intent = Intent(ACTION_ERROR).apply {
            putExtra(EXTRA_ERROR, error)
        }
        sendBroadcast(intent)
    }

    // ── Notification ────────────────────────────────────────────────────────────────

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "WebSocket Gateway",
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = "WebSocket connection for remote control"
        }
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(channel)
    }

    private fun buildNotification(url: String): Notification {
        return Notification.Builder(this, CHANNEL_ID)
            .setContentTitle("WebSocket Gateway")
            .setContentText("Connected to $url")
            .setSmallIcon(android.R.drawable.ic_menu_mylocation)
            .setOngoing(true)
            .build()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        disconnect()
        client?.dispatcher?.executorService()?.shutdown()
        client = null
        super.onDestroy()
        Log.d(TAG, "WebSocketGateway service destroyed")
    }

    // ── Utility Methods ─────────────────────────────────────────────────────────────

    /**
     * Check if WebSocket is currently connected
     */
    fun isConnected(): Boolean = isConnected

    /**
     * Get current connection URL
     */
    fun getConnectionUrl(): String? = webSocket?.request()?.url?.toString()

    /**
     * Reconnect to the last URL
     */
    fun reconnect() {
        getConnectionUrl()?.let { url ->
            disconnect()
            connect(url)
        }
    }
}
