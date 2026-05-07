package com.yourdomain.agent

import android.app.Notification
import android.content.Intent
import android.content.SharedPreferences
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import android.util.Log

class NotificationMonitorService : NotificationListenerService() {

    companion object {
        private const val TAG = "NotificationMonitor"
        private const val PREFS_NAME = "AgentPrefs"

        // Monitored social app package names
        private const val WECHAT_PACKAGE = "com.tencent.mm"
        private const val QQ_PACKAGE = "com.tencent.mobileqq"
        private const val WEIBO_PACKAGE = "com.sina.weibo"
        private const val DINGTALK_PACKAGE = "com.alibaba.android.rimet"
        private const val WHATSAPP_PACKAGE = "com.whatsapp"
        private const val TELEGRAM_PACKAGE = "org.telegram.messenger"

        const val BROADCAST_ACTION = "com.yourdomain.agent.NOTIFICATION_RECEIVED"
    }

    private lateinit var prefs: SharedPreferences

    // ── Lifecycle ──────────────────────────────────────────────

    override fun onCreate() {
        super.onCreate()
        prefs = getSharedPreferences(PREFS_NAME, MODE_PRIVATE)
        Log.d(TAG, "Notification monitor service started")
    }

    override fun onListenerConnected() {
        super.onListenerConnected()
        Log.d(TAG, "Notification listener connected")
    }

    override fun onListenerDisconnected() {
        super.onListenerDisconnected()
        Log.d(TAG, "Notification listener disconnected")
    }

    // ── Notification handling ─────────────────────────────────

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        if (sbn == null) return

        val packageName = sbn.packageName
        val notification = sbn.notification ?: return
        val extras = notification.extras ?: return

        // Extract notification content
        var title = extras.getString(Notification.EXTRA_TITLE, "") ?: ""
        var text = extras.getString(Notification.EXTRA_TEXT, "") ?: ""
        val bigText = extras.getCharSequence(Notification.EXTRA_BIG_TEXT)
        if (bigText != null && bigText.length > text.length) {
            text = bigText.toString()
        }

        // Check if this is a monitored app
        if (!isMonitoredApp(packageName)) return

        // Extract sender
        val sender = extractSender(title, text, packageName) ?: return

        // Check if sender is monitored
        if (!isMonitoredContact(sender)) return

        // Notify agent
        notifyAgent(sender, text, packageName)
    }

    override fun onNotificationRemoved(sbn: StatusBarNotification?) {
        // No action needed on removal
    }

    // ── App monitoring check ──────────────────────────────────

    private fun isMonitoredApp(packageName: String): Boolean {
        val monitorWeChat = prefs.getBoolean("monitorWeChat", true)
        val monitorQQ = prefs.getBoolean("monitorQQ", true)
        val monitorWeibo = prefs.getBoolean("monitorWeibo", true)
        val monitorDingTalk = prefs.getBoolean("monitorDingTalk", true)
        val monitorWhatsApp = prefs.getBoolean("monitorWhatsApp", true)
        val monitorTelegram = prefs.getBoolean("monitorTelegram", true)

        return when (packageName) {
            WECHAT_PACKAGE -> monitorWeChat
            QQ_PACKAGE -> monitorQQ
            WEIBO_PACKAGE -> monitorWeibo
            DINGTALK_PACKAGE -> monitorDingTalk
            WHATSAPP_PACKAGE -> monitorWhatsApp
            TELEGRAM_PACKAGE -> monitorTelegram
            else -> false
        }
    }

    // ── Sender extraction ─────────────────────────────────────

    private fun extractSender(title: String, text: String, packageName: String): String? {
        return when (packageName) {
            WECHAT_PACKAGE -> extractWeChatSender(title, text)
            QQ_PACKAGE -> extractQQSender(title, text)
            WEIBO_PACKAGE -> extractWeiboSender(title, text)
            DINGTALK_PACKAGE -> extractDingTalkSender(title, text)
            WHATSAPP_PACKAGE -> extractWhatsAppSender(title, text)
            TELEGRAM_PACKAGE -> extractTelegramSender(title, text)
            else -> null
        }
    }

    private fun extractWeChatSender(title: String, text: String): String? {
        // WeChat: title is usually the sender or group name
        if (title.isNotEmpty() && title != "微信") return title
        // Some ROMs hide details; title may be "微信", try parsing from text
        // Common format: "sender: message" or "sender：message"
        return parseWeChatSender(text)
    }

    private fun extractQQSender(title: String, text: String): String? {
        // QQ: title is usually the sender
        if (title.isNotEmpty() && title != "QQ") return title
        // Try to parse from text if title is generic
        // Common format: "sender: message" or "sender：message"
        var colonIdx = text.indexOf(':')
        if (colonIdx == -1) colonIdx = text.indexOf('：')
        if (colonIdx > 0) {
            val sender = text.substring(0, colonIdx).trim()
            if (sender.isNotEmpty() && sender.length < 30) return sender
        }
        return null
    }

    private fun extractWeiboSender(title: String, text: String): String? {
        // Weibo: title is usually the username
        if (title.isNotEmpty() && title != "微博") return title
        // Try to parse from text
        var colonIdx = text.indexOf(':')
        if (colonIdx == -1) colonIdx = text.indexOf('：')
        if (colonIdx > 0) {
            val sender = text.substring(0, colonIdx).trim()
            if (sender.isNotEmpty() && sender.length < 30) return sender
        }
        return null
    }

    private fun extractDingTalkSender(title: String, text: String): String? {
        // DingTalk: title is usually the sender or group name
        if (title.isNotEmpty() && title != "钉钉") return title
        // Try to parse from text
        var colonIdx = text.indexOf(':')
        if (colonIdx == -1) colonIdx = text.indexOf('：')
        if (colonIdx > 0) {
            val sender = text.substring(0, colonIdx).trim()
            if (sender.isNotEmpty() && sender.length < 30) return sender
        }
        return null
    }

    private fun extractWhatsAppSender(title: String, text: String): String? {
        // WhatsApp: title is usually the sender name
        if (title.isNotEmpty() && title != "WhatsApp") return title
        // Try to parse from text
        var colonIdx = text.indexOf(':')
        if (colonIdx == -1) colonIdx = text.indexOf('：')
        if (colonIdx > 0) {
            val sender = text.substring(0, colonIdx).trim()
            if (sender.isNotEmpty() && sender.length < 30) return sender
        }
        return null
    }

    private fun extractTelegramSender(title: String, text: String): String? {
        // Telegram: title is usually the sender or group name
        if (title.isNotEmpty() && title != "Telegram") return title
        // Try to parse from text
        var colonIdx = text.indexOf(':')
        if (colonIdx == -1) colonIdx = text.indexOf('：')
        if (colonIdx > 0) {
            val sender = text.substring(0, colonIdx).trim()
            if (sender.isNotEmpty() && sender.length < 30) return sender
        }
        return null
    }

    private fun parseWeChatSender(text: String): String? {
        if (text.isEmpty()) return null

        // Try matching "sender: content" or "sender：content"
        var colonIdx = text.indexOf(':')
        if (colonIdx == -1) colonIdx = text.indexOf('：')
        if (colonIdx > 0) {
            val sender = text.substring(0, colonIdx).trim()
            if (sender.isNotEmpty() && sender.length < 30) return sender
        }

        // If text itself is short (possibly just a name), return it
        if (text.length < 15 && !text.contains("收到") && !text.contains("条新消息")) {
            return text.trim()
        }

        return null
    }

    // ── Contact matching ──────────────────────────────────────

    private fun isMonitoredContact(sender: String, monitoredContacts: Set<String>?): Boolean {
        if (monitoredContacts.isNullOrEmpty()) return false
        for (contact in monitoredContacts) {
            if (sender.contains(contact) || contact.contains(sender)) return true
        }
        return false
    }

    private fun isMonitoredContact(sender: String): Boolean {
        val monitoredContacts = prefs.getStringSet("monitoredContacts", null)
        return isMonitoredContact(sender, monitoredContacts)
    }

    // ── Broadcast ─────────────────────────────────────────────

    private fun notifyAgent(sender: String, content: String, packageName: String) {
        // Add message to the global queue
        GlobalMessageQueue.addMessage(sender, content, packageName)

        val intent = Intent(BROADCAST_ACTION).apply {
            setPackage(this@NotificationMonitorService.packageName)
            putExtra("sender", sender)
            putExtra("content", content)
            putExtra("packageName", packageName)
        }
        sendBroadcast(intent)

        // Log with masked info
        val maskedSender = maskSensitiveInfo(sender)
        val maskedContent = maskSensitiveInfo(content)
        Log.d(TAG, "Notification captured: $maskedSender — $maskedContent")
    }

    private fun broadcastNotification(sender: String, content: String, packageName: String) {
        notifyAgent(sender, content, packageName)
    }

    // ── Privacy masking ───────────────────────────────────────

    /**
     * Mask sensitive info: if text is 4 chars or fewer, show "***";
     * otherwise show first 2 chars + "***" + last 2 chars.
     */
    private fun maskSensitiveInfo(text: String?): String {
        if (text.isNullOrEmpty()) return ""
        if (text.length <= 4) return "***"
        return text.substring(0, 2) + "***" + text.substring(text.length - 2)
    }
}
