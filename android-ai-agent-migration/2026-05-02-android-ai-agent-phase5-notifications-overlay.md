# KimiClaw vs Android AI Agent — Feature Comparison

> Source: `catcatboy-cyber/KimiClaw` (Java, 8 source files) + `www.kimi.com/help/kimi-claw`

## What KimiClaw Is

A desktop floating pet app (lobster) for Android. Runs as a foreground service with a `WindowManager` overlay. Keyword-based AI chat. Monitors social app notifications (WeChat, QQ, Weibo, DingTalk, WhatsApp, Telegram) for specific contacts.

**Not** an AI agent — it's a pet with simple pattern-matching chat. No LLM, no phone automation, no accessibility control.

## Feature Matrix

| Feature | KimiClaw | android-ai-agent | Gap |
|---|---|---|---|
| Rust core | ❌ | ✅ libagent_core.so | |
| LLM-powered (OpenRouter) | ❌ keyword-matching | ✅ 300+ models | |
| Tiered model routing | ❌ | ✅ 4 tiers + fallbacks | |
| AccessibilityService (screen control) | ❌ | ✅ tap/swipe/type/openApp | |
| **NotificationListenerService** | ✅ full impl | ❌ stub only | **KimiClaw ahead** |
| **Floating window overlay** | ✅ WindowManager | ❌ | **KimiClaw ahead** |
| **Message queue / aggregation** | ✅ CopyOnWriteArrayList | ❌ | **KimiClaw ahead** |
| **Multi-app notification parsing** | ✅ 6 apps (WeChat, QQ, etc.) | ❌ | **KimiClaw ahead** |
| **Contact-specific monitoring** | ✅ SharedPreferences set | ❌ | **KimiClaw ahead** |
| Telegram bot channel | ❌ | ✅ spec | |
| WhatsApp (via Accessibility) | ❌ | ✅ spec | |
| Voice mode (STT/TTS) | ❌ | ✅ spec | |
| Persistent MEMORY.md | ❌ | ✅ | |
| Skill plugin system | ❌ | ✅ 19 built-in skills | |
| Safety policy enforcement | ❌ | ✅ tier gating + permission guard | |
| Android Keystore (API keys) | ❌ | ✅ hardware-backed AES/GCM | |
| WebSocket gateway | ❌ | ✅ optional | |
| Compose UI (Material3) | ❌ (legacy XML views) | ✅ 6 screens | |

## What We Should Steal from KimiClaw

### 1. NotificationListenerService — FULL implementation

KimiClaw has a production-ready `MessageMonitorService` that we should adapt:

```java
// Already implemented: onNotificationPosted, extractSender per app, contact matching
// Supports: WeChat, QQ, Weibo, DingTalk, WhatsApp, Telegram
// Uses: SharedPreferences for monitored contacts, broadcast to notify overlay
```

**Port to Kotlin** as `NotificationMonitorService.kt` in our project. Directly lifts:
- `onNotificationPosted(StatusBarNotification)` → read `extras.getString(EXTRA_TITLE)`, `EXTRA_TEXT`, `EXTRA_BIG_TEXT`
- `isMonitoredApp(packageName)` → check against known social app packages
- `extractSender(title, text, packageName)` → app-specific parsing (WeChat: title or "sender: msg" format; QQ: title)
- `isMonitoredContact(sender, contacts)` → fuzzy match against `SharedPreferences.getStringSet`
- `notifyFloatingLobster()` → broadcast to agent overlay with sender/content/package

### 2. Floating Agent Overlay

KimiClaw uses `WindowManager` + `TYPE_APPLICATION_OVERLAY` for its pet. We can repurpose this for an **agent status indicator**:

```
┌────────────────────┐
│ ◉ AGENT [Running]  │  ← floating pill showing:
│ "Opening Gmail..."  │     - agent status (idle/running/waiting)
└────────────────────┘     - current action
                           - tap to open full UI
```

Implementation pattern from KimiClaw:
```kotlin
val params = WindowManager.LayoutParams(
    WRAP_CONTENT, WRAP_CONTENT,
    TYPE_APPLICATION_OVERLAY,  // Always on top
    FLAG_NOT_FOCUSABLE or FLAG_NOT_TOUCH_MODAL,
    PixelFormat.TRANSLUCENT
)
params.gravity = Gravity.TOP or Gravity.START
windowManager.addView(floatingView, params)
```

### 3. Message Queue for Notifications

KimiClaw's `CopyOnWriteArrayList<MessageItem>` with max 20 items and `RecyclerView` adapter is directly portable. This solves: "agent sees a message, needs to read it, process it, notify user."

## Features to Add Based on Comparison

Priority order:

1. **NotificationMonitorService.kt** — Port KimiClaw's `MessageMonitorService` to Kotlin. Add to `agent/src/main/kotlin/.../service/`. Wire to AgentViewModel so incoming messages can trigger agent actions.

2. **FloatingAgentOverlay.kt** — Port KimiClaw's floating window pattern. Show agent status pill when agent is active. Tap to open full UI.

3. **MessageQueue.kt** — Thread-safe message buffer using Kotlin `MutableStateFlow<List<MessageItem>>` (better than Java's CopyOnWriteArrayList).

4. **Enhanced notification parsing** — Add support for the 6 apps KimiClaw handles, plus any others from our spec (we already list Telegram, WhatsApp in permission guard).<｜end▁of▁thinking｜>

<｜｜DSML｜｜tool_calls>
<｜｜DSML｜｜invoke name="write_file">
<｜｜DSML｜｜parameter name="content" string="true"># Android AI Agent — Phase 5: Notification Monitor + Floating Overlay

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Port KimiClaw's production-ready `NotificationListenerService` pattern to Kotlin, add a floating agent status overlay, and build a thread-safe message queue — all patterns proven in the KimiClaw app (catcatboy-cyber/KimiClaw).

**Architecture:** `NotificationMonitorService` runs as a foreground service, reads incoming notifications from 6 supported apps (WeChat, QQ, Weibo, DingTalk, WhatsApp, Telegram), matches against monitored contacts from SharedPreferences, and broadcasts matched messages to the agent overlay and AgentViewModel. `FloatingAgentOverlay` shows a persistent status pill using `WindowManager`. `MessageQueue` buffers incoming notifications.

**Tech Stack:** Kotlin, Android `NotificationListenerService`, `WindowManager`, `SharedPreferences`, Material3 Compose (for the floating view).

---

### Task 1: NotificationMonitorService — port KimiClaw's MessageMonitorService to Kotlin

**Objective:** Production-ready notification listener supporting 6 social apps. Ported directly from KimiClaw's `MessageMonitorService.java`.

**Files:**
- Create: `app/src/main/kotlin/com/yourdomain/agent/NotificationMonitorService.kt`

**Code:**

```kotlin
package com.yourdomain.agent

import android.app.Notification
import android.app.PendingIntent
import android.content.Intent
import android.content.SharedPreferences
import android.os.Bundle
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import android.util.Log

class NotificationMonitorService : NotificationListenerService() {

    private lateinit var prefs: SharedPreferences

    companion object {
        private const val TAG = "NotificationMonitor"
        private const val WECHAT = "com.tencent.mm"
        private const val QQ = "com.tencent.mobileqq"
        private const val WEIBO = "com.sina.weibo"
        private const val DINGTALK = "com.alibaba.android.rimet"
        private const val WHATSAPP = "com.whatsapp"
        private const val TELEGRAM = "org.telegram.messenger"
    }

    override fun onCreate() {
        super.onCreate()
        prefs = getSharedPreferences("AgentPrefs", MODE_PRIVATE)
        Log.d(TAG, "Notification monitor started")
    }

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        if (sbn == null) return
        val notification = sbn.notification ?: return
        val extras = notification.extras ?: return
        val packageName = sbn.packageName

        if (!isMonitoredApp(packageName)) return

        var title = extras.getString(Notification.EXTRA_TITLE, "") ?: ""
        var text = extras.getString(Notification.EXTRA_TEXT, "") ?: ""
        val bigText = extras.getCharSequence(Notification.EXTRA_BIG_TEXT)
        if (bigText != null && bigText.length > text.length) {
            text = bigText.toString()
        }

        val monitoredContacts = prefs.getStringSet("monitoredContacts", emptySet()) ?: emptySet()
        if (monitoredContacts.isEmpty()) return

        val sender = extractSender(title, text, packageName)
        if (sender != null && isMonitoredContact(sender, monitoredContacts)) {
            notifyAgent(sender, text, packageName, notification.contentIntent)
        }
    }

    private fun isMonitoredApp(packageName: String): Boolean {
        return when (packageName) {
            WECHAT -> prefs.getBoolean("monitorWeChat", true)
            QQ -> prefs.getBoolean("monitorQQ", true)
            WEIBO, DINGTALK, WHATSAPP, TELEGRAM -> true
            else -> false
        }
    }

    private fun extractSender(title: String, text: String, packageName: String): String? {
        return when (packageName) {
            WECHAT -> {
                if (title.isNotEmpty() && title != "微信") return title
                parseWeChatSender(text)
            }
            QQ -> {
                if (title.isNotEmpty() && title != "QQ") return title
                null
            }
            else -> {
                if (title.isNotEmpty() && title.length < 20) title else null
            }
        }
    }

    private fun parseWeChatSender(text: String): String? {
        if (text.isEmpty()) return null
        var idx = text.indexOf(':')
        if (idx == -1) idx = text.indexOf('：')
        if (idx > 0) {
            val sender = text.substring(0, idx).trim()
            if (sender.isNotEmpty() && sender.length < 30) return sender
        }
        if (text.length < 15 && !text.contains("收到") && !text.contains("条新消息")) {
            return text.trim()
        }
        return null
    }

    private fun isMonitoredContact(sender: String, contacts: Set<String>): Boolean {
        return contacts.any { sender.contains(it) || it.contains(sender) }
    }

    private fun notifyAgent(sender: String, content: String, packageName: String, contentIntent: PendingIntent?) {
        val intent = Intent("com.yourdomain.agent.NOTIFICATION_RECEIVED").apply {
            setPackage(packageName)
            putExtra("sender", sender)
            putExtra("content", content)
            putExtra("packageName", packageName)
        }
        sendBroadcast(intent)

        val maskedSender = if (sender.length <= 4) "***" else "${sender.take(2)}***${sender.takeLast(2)}"
        Log.d(TAG, "Monitored message: $maskedSender")
    }

    override fun onNotificationRemoved(sbn: StatusBarNotification?) {}
    override fun onListenerConnected() { Log.d(TAG, "Listener connected") }
    override fun onListenerDisconnected() { Log.d(TAG, "Listener disconnected") }
}
```

**Step 2: Register in AndroidManifest.xml**
```xml
<service
    android:name=".NotificationMonitorService"
    android:permission="android.permission.BIND_NOTIFICATION_LISTENER_SERVICE"
    android:exported="true">
    <intent-filter>
        <action android:name="android.service.notification.NotificationListenerService" />
    </intent-filter>
</service>
```

**Step 3: Commit**
```bash
git add -A && git commit -m "feat: add NotificationMonitorService ported from KimiClaw pattern"
```

---

### Task 2: Floating Agent Overlay — status pill

**Objective:** Persistent floating overlay showing agent status using `WindowManager`. Pattern from KimiClaw's `FloatingLobsterService`.

**Files:**
- Create: `app/src/main/kotlin/com/yourdomain/agent/FloatingAgentOverlay.kt`

**Code:**

```kotlin
package com.yourdomain.agent

import android.app.PendingIntent
import android.app.Service
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.graphics.PixelFormat
import android.os.Build
import android.os.IBinder
import android.view.Gravity
import android.view.LayoutInflater
import android.view.View
import android.view.WindowManager
import android.widget.TextView

class FloatingAgentOverlay : Service() {

    private lateinit var windowManager: WindowManager
    private var floatingView: View? = null
    private var statusText: TextView? = null
    private var actionText: TextView? = null
    private val receiver = NotificationReceiver()

    companion object {
        private const val TAG = "FloatingAgent"
    }

    override fun onCreate() {
        super.onCreate()
        windowManager = getSystemService(WINDOW_SERVICE) as WindowManager
        registerReceiver(receiver, IntentFilter("com.yourdomain.agent.NOTIFICATION_RECEIVED"))
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        showOverlay()
        return START_STICKY
    }

    private fun showOverlay() {
        if (floatingView != null) return

        floatingView = LayoutInflater.from(this).inflate(R.layout.floating_agent, null)
        statusText = floatingView?.findViewById(R.id.agentStatus)
        actionText = floatingView?.findViewById(R.id.agentAction)

        val type = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY
        } else {
            WindowManager.LayoutParams.TYPE_PHONE
        }

        val params = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            type,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_NOT_TOUCH_MODAL,
            PixelFormat.TRANSLUCENT
        ).apply {
            gravity = Gravity.TOP or Gravity.START
            x = 16
            y = 100
        }

        windowManager.addView(floatingView, params)
    }

    fun updateStatus(status: String, action: String = "") {
        statusText?.text = "◉ $status"
        actionText?.text = action
    }

    fun hide() {
        floatingView?.let { windowManager.removeView(it) }
        floatingView = null
    }

    override fun onDestroy() {
        hide()
        unregisterReceiver(receiver)
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    inner class NotificationReceiver : BroadcastReceiver() {
        override fun onReceive(context: Context?, intent: Intent?) {
            val sender = intent?.getStringExtra("sender") ?: return
            val content = intent?.getStringExtra("content") ?: ""
            updateStatus("Message", "$sender: $content")
        }
    }
}
```

**Step 2: Create layout `res/layout/floating_agent.xml`**
```xml
<?xml version="1.0" encoding="utf-8"?>
<LinearLayout xmlns:android="http://schemas.android.com/apk/res/android"
    android:layout_width="wrap_content"
    android:layout_height="wrap_content"
    android:orientation="vertical"
    android:background="@drawable/card_bg"
    android:padding="12dp"
    android:elevation="8dp">
    <TextView
        android:id="@+id/agentStatus"
        android:layout_width="wrap_content"
        android:layout_height="wrap_content"
        android:text="◉ Idle"
        android:textColor="@android:color/white"
        android:textSize="14sp"
        android:textStyle="bold" />
    <TextView
        android:id="@+id/agentAction"
        android:layout_width="wrap_content"
        android:layout_height="wrap_content"
        android:text=""
        android:textColor="#AAFFFFFF"
        android:textSize="12sp" />
</LinearLayout>
```

**Step 3: Add permission to manifest**
```xml
<uses-permission android:name="android.permission.SYSTEM_ALERT_WINDOW" />
```

**Step 4: Commit**
```bash
git add -A && git commit -m "feat: add floating agent overlay using WindowManager pattern from KimiClaw"
```

---

### Task 3: MessageQueue — thread-safe notification buffer

**Objective:** Replace KimiClaw's `CopyOnWriteArrayList<MessageItem>` with a Kotlin `StateFlow`-based message queue. Max 20 items, observable from ViewModel.

**Files:**
- Create: `app/src/main/kotlin/com/yourdomain/agent/MessageQueue.kt`

**Code:**

```kotlin
package com.yourdomain.agent

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

data class MessageItem(
    val sender: String,
    val content: String,
    val packageName: String,
    val timestamp: Long = System.currentTimeMillis(),
)

class MessageQueue(maxSize: Int = 20) {
    private val _messages = MutableStateFlow<List<MessageItem>>(emptyList())
    val messages: StateFlow<List<MessageItem>> = _messages.asStateFlow()

    fun add(sender: String, content: String, packageName: String) {
        val item = MessageItem(sender, content, packageName)
        val current = _messages.value.toMutableList()
        current.add(0, item) // newest first
        if (current.size > maxSize) {
            current.removeAt(current.size - 1)
        }
        _messages.value = current
    }

    fun clear() {
        _messages.value = emptyList()
    }

    fun size(): Int = _messages.value.size
}
```

**Step 2: Commit**
```bash
git add -A && git commit -m "feat: add StateFlow-based message queue with max capacity"
```

---

## Completion Checklist

- [ ] NotificationMonitorService monitors 6 social apps
- [ ] Contact-specific matching from SharedPreferences
- [ ] Floating overlay shows agent status + incoming messages
- [ ] MessageQueue buffers last 20 messages via StateFlow
- [ ] Manifest updated with SYSTEM_ALERT_WINDOW + notification listener permission

## After Phase 5

The Android agent now has:
- **Rust core**: 24 source files, 30+ tests
- **Phone control**: AccessibilityService (tap/swipe/type/openApp)
- **Notifications**: 6-app notification monitoring (KimiClaw pattern)
- **Overlay**: Floating status pill (KimiClaw pattern)
- **UI**: 6 Compose screens with navigation
- **Security**: KeystoreManager + policy enforcer + permission guard
- **Bridge**: JNI exports + RustBridge + AgentViewModel
