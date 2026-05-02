package com.yourdomain.agent

import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.graphics.PixelFormat
import android.os.Build
import android.os.IBinder
import android.util.Log
import android.view.Gravity
import android.view.LayoutInflater
import android.view.View
import android.view.WindowManager
import android.widget.TextView

/**
 * Persistent floating status pill that displays agent state and
 * the most recent notification from monitored contacts.
 *
 * Ported from the KimiClaw FloatingLobsterService pattern:
 * - WindowManager overlay with pass-through touch flags
 * - BroadcastReceiver for NOTIFICATION_RECEIVED intents
 * - START_STICKY to survive low-memory kills
 */
class FloatingAgentOverlay : Service() {

    companion object {
        private const val TAG = "FloatingAgentOverlay"
    }

    // ── Window members ──────────────────────────────────────────

    private lateinit var windowManager: WindowManager
    private var overlayView: View? = null
    private var statusText: TextView? = null
    private var actionText: TextView? = null

    // ── Receiver ────────────────────────────────────────────────

    private val notificationReceiver = object : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            if (intent?.action != NotificationMonitorService.BROADCAST_ACTION) return

            val sender = intent.getStringExtra("sender") ?: "Unknown"
            val content = intent.getStringExtra("content") ?: ""

            // Brief preview: truncate content to keep the pill compact
            val preview = if (content.length > 24) content.take(24) + "…" else content

            updateStatus("◉ Alert", sender)
            updateAction(preview)

            Log.d(TAG, "Overlay updated from broadcast: $sender — $preview")
        }
    }

    // ── Lifecycle ───────────────────────────────────────────────

    override fun onCreate() {
        super.onCreate()
        windowManager = getSystemService(WINDOW_SERVICE) as WindowManager
        Log.d(TAG, "Floating overlay service created")
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // Register notification broadcast receiver
        val filter = IntentFilter(NotificationMonitorService.BROADCAST_ACTION)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            registerReceiver(notificationReceiver, filter, RECEIVER_NOT_EXPORTED)
        } else {
            registerReceiver(notificationReceiver, filter)
        }

        // Show the overlay if not already visible
        if (overlayView == null) {
            showOverlay()
        }

        // If launched with extras, apply them immediately
        intent?.let {
            val status = it.getStringExtra("status") ?: "◉ Idle"
            val action = it.getStringExtra("action") ?: ""
            updateStatus(status, action)
        }

        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        hide()
        try {
            unregisterReceiver(notificationReceiver)
        } catch (_: IllegalArgumentException) {
            // Receiver was not registered
        }
        Log.d(TAG, "Floating overlay service destroyed")
        super.onDestroy()
    }

    // ── Public API ──────────────────────────────────────────────

    /** Inflate and add the overlay to the WindowManager. */
    fun showOverlay() {
        if (overlayView != null) return

        val inflater = getSystemService(LAYOUT_INFLATER_SERVICE) as LayoutInflater
        overlayView = inflater.inflate(R.layout.floating_agent, null)

        statusText = overlayView?.findViewById(R.id.agentStatus)
        actionText = overlayView?.findViewById(R.id.agentAction)

        val params = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY
            } else {
                @Suppress("DEPRECATION")
                WindowManager.LayoutParams.TYPE_PHONE
            },
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or
                    WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL,
            PixelFormat.TRANSLUCENT
        ).apply {
            gravity = Gravity.TOP or Gravity.START
            x = 16
            y = 120
        }

        windowManager.addView(overlayView, params)
        Log.d(TAG, "Overlay shown")
    }

    /**
     * Update the left-hand status label (e.g. "◉ Idle", "◉ Thinking", "◉ Alert").
     */
    fun updateStatus(status: String, action: String = "") {
        statusText?.text = status
        if (action.isNotEmpty()) {
            updateAction(action)
        }
    }

    /**
     * Update the right-hand action / task label.
     */
    fun updateAction(action: String) {
        actionText?.text = action
    }

    /** Remove the overlay from the WindowManager. */
    fun hide() {
        overlayView?.let {
            windowManager.removeView(it)
            overlayView = null
            statusText = null
            actionText = null
        }
    }
}
