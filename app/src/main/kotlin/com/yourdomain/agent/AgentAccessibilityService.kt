package com.yourdomain.agent

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.content.Intent
import android.graphics.Path
import android.graphics.Rect
import android.util.DisplayMetrics
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import android.view.WindowManager
import android.os.Bundle

class AgentAccessibilityService : AccessibilityService() {

    // ── Lifecycle ──────────────────────────────────────────────

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // Intercepted by agent loop when screen content is requested
    }

    override fun onInterrupt() {
        // Called when the system wants to interrupt the service
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        // Service is ready; agent loop can start querying UI
    }

    // ── Screen content ─────────────────────────────────────────

    /**
     * Dumps the current window's UI tree as an indented string.
     * Each line uses `|-- ` indentation per depth level.
     */
    fun getScreenContent(): String {
        val root = rootInActiveWindow ?: return ""
        return nodeToString(root, 0).also { root.recycle() }
    }

    // ── Gesture helpers ────────────────────────────────────────

    /**
     * Tap at absolute screen coordinates (x, y).
     * [callback] receives `true` if the gesture was dispatched successfully.
     */
    fun tap(x: Int, y: Int, callback: (Boolean) -> Unit) {
        val path = Path().apply { moveTo(x.toFloat(), y.toFloat()) }
        val stroke = GestureDescription.StrokeDescription(path, 0L, 1L)
        val gesture = GestureDescription.Builder()
            .addStroke(stroke)
            .build()
        dispatchGesture(gesture, object : GestureResultCallback() {
            override fun onCompleted(gestureDescription: GestureDescription?) = callback(true)
            override fun onCancelled(gestureDescription: GestureDescription?) = callback(false)
        }, null)
    }

    /**
     * Swipe from (fromX, fromY) to (toX, toY) over [duration]ms.
     * [callback] receives `true` if the gesture was dispatched successfully.
     */
    fun swipe(
        fromX: Int, fromY: Int,
        toX: Int, toY: Int,
        duration: Long = 300L,
        callback: (Boolean) -> Unit
    ) {
        val path = Path().apply {
            moveTo(fromX.toFloat(), fromY.toFloat())
            lineTo(toX.toFloat(), toY.toFloat())
        }
        val stroke = GestureDescription.StrokeDescription(path, 0L, duration)
        val gesture = GestureDescription.Builder()
            .addStroke(stroke)
            .build()
        dispatchGesture(gesture, object : GestureResultCallback() {
            override fun onCompleted(gestureDescription: GestureDescription?) = callback(true)
            override fun onCancelled(gestureDescription: GestureDescription?) = callback(false)
        }, null)
    }

    // ── Text input ─────────────────────────────────────────────

    /**
     * Performs ACTION_SET_TEXT on the currently focused editable node.
     */
    fun typeText(text: String) {
        val focused = findFocus(AccessibilityNodeInfo.FOCUS_INPUT)
        if (focused != null) {
            val args = Bundle().apply { putCharSequence(
                AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, text
            )}
            focused.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args)
            focused.recycle()
        }
    }

    // ── App launch ─────────────────────────────────────────────

    /**
     * Launch an app by its package name using an Intent.
     */
    fun openApp(packageName: String) {
        val intent = packageManager.getLaunchIntentForPackage(packageName)
        if (intent != null) {
            intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            startActivity(intent)
        }
    }

    // ── Screen metrics ─────────────────────────────────────────

    /**
     * Returns the logical screen size as (width, height) in pixels.
     */
    fun getScreenSize(): Pair<Int, Int> {
        val metrics = DisplayMetrics().also {
            (getSystemService(WINDOW_SERVICE) as WindowManager).defaultDisplay.getRealMetrics(it)
        }
        return Pair(metrics.widthPixels, metrics.heightPixels)
    }

    // ── Private helpers ────────────────────────────────────────

    /**
     * Recursively convert an [AccessibilityNodeInfo] tree to an indented string.
     */
    private fun nodeToString(node: AccessibilityNodeInfo, depth: Int): String {
        val indent = "|-- ".repeat(depth)
        val sb = StringBuilder()

        val className = node.className?.toString() ?: "unknown"
        val text = if (!node.text.isNullOrEmpty()) " text=\"${node.text}\"" else ""
        val desc = if (!node.contentDescription.isNullOrEmpty()) " desc=\"${node.contentDescription}\"" else ""
        val id = if (node.viewIdResourceName != null) " id=\"${node.viewIdResourceName}\"" else ""
        val bounds = Rect()
        node.getBoundsInScreen(bounds)
        val rect = " bounds=[${bounds.left},${bounds.top}][${bounds.right},${bounds.bottom}]"
        val clickable = if (node.isClickable) " clickable" else ""
        val focused = if (node.isFocused) " focused" else ""

        sb.appendLine("$indent$className$text$desc$id$rect$clickable$focused")

        for (i in 0 until node.childCount) {
            val child = node.getChild(i) ?: continue
            sb.append(nodeToString(child, depth + 1))
            child.recycle()
        }

        return sb.toString()
    }
}
