package com.yourdomain.agent

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.content.Intent
import android.graphics.Path
import android.graphics.Rect
import android.os.Bundle
import android.util.DisplayMetrics
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import android.view.WindowManager

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

    // ── Node finding ────────────────────────────────────────────

    /**
     * Find a node by its text content (exact match).
     * Returns the first matching node or null if not found.
     */
    fun findNodeByText(text: String): AccessibilityNodeInfo? {
        val root = rootInActiveWindow ?: return null
        return findNodeByTextRecursive(root, text)?.also { root.recycle() }
    }

    private fun findNodeByTextRecursive(
        node: AccessibilityNodeInfo,
        text: String
    ): AccessibilityNodeInfo? {
        if (node.text?.toString() == text) {
            return node
        }
        for (i in 0 until node.childCount) {
            val child = node.getChild(i) ?: continue
            val result = findNodeByTextRecursive(child, text)
            if (result != null) {
                child.recycle()
                return result
            }
            child.recycle()
        }
        return null
    }

    // ── Tap operations ──────────────────────────────────────────

    /**
     * Tap at absolute screen coordinates (x, y).
     * callback receives true if the gesture was dispatched successfully.
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
     * Tap on a specific AccessibilityNodeInfo at its center.
     * callback receives true if the gesture was dispatched successfully.
     */
    fun tapNode(node: AccessibilityNodeInfo, callback: (Boolean) -> Unit) {
        val bounds = Rect().also { node.getBoundsInScreen(it) }
        val centerX = bounds.centerX()
        val centerY = bounds.centerY()
        tap(centerX, centerY, callback)
    }

    // ── Swipe operations ────────────────────────────────────────

    /**
     * Swipe from (fromX, fromY) to (toX, toY) over [duration]ms.
     * callback receives true if the gesture was dispatched successfully.
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

    // ── Scroll operations ────────────────────────────────────────

    /**
     * Scroll forward (down) on the screen.
     * callback receives true if the gesture was dispatched successfully.
     */
    fun scroll(direction: ScrollDirection = ScrollDirection.FORWARD, callback: (Boolean) -> Unit) {
        val root = rootInActiveWindow ?: return callback(false)
        val bounds = Rect().also { root.getBoundsInScreen(it) }
        root.recycle()

        val startX = bounds.centerX()
        val startY = if (direction == ScrollDirection.FORWARD) bounds.bottom - 100 else bounds.top + 100
        val endY = if (direction == ScrollDirection.FORWARD) bounds.top + 100 else bounds.bottom - 100

        swipe(startX, startY, startX, endY, 300L, callback)
    }

    enum class ScrollDirection {
        FORWARD,
        BACKWARD
    }

    // ── Text input ─────────────────────────────────────────────

    /**
     * Input text into the currently focused editable node.
     * Uses ACTION_SET_TEXT to directly set the text content.
     */
    fun inputText(text: String): Boolean {
        val focused = findFocus(AccessibilityNodeInfo.FOCUS_INPUT)
        if (focused != null) {
            val args = Bundle().apply {
                putCharSequence(
                    AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE,
                    text
                )
            }
            val result = focused.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, args)
            focused.recycle()
            return result
        }
        return false
    }

    // ── Screen text retrieval ────────────────────────────────────

    /**
     * Get all visible text from the current screen.
     * Returns a concatenated string of all text nodes.
     */
    fun getScreenText(): String {
        val root = rootInActiveWindow ?: return ""
        return collectTextRecursive(root).also { root.recycle() }
    }

    private fun collectTextRecursive(node: AccessibilityNodeInfo): String {
        val sb = StringBuilder()
        if (!node.text.isNullOrEmpty()) {
            sb.append(node.text).append("\n")
        }
        for (i in 0 until node.childCount) {
            val child = node.getChild(i) ?: continue
            sb.append(collectTextRecursive(child))
            child.recycle()
        }
        return sb.toString().trim()
    }

    /**
     * Get the current package name of the active window.
     */
    fun getCurrentPackage(): String? {
        val root = rootInActiveWindow ?: return null
        val packageName = root.packageName?.toString()
        root.recycle()
        return packageName
    }

    // ── Navigation ─────────────────────────────────────────────

    /**
     * Simulate a back button press.
     */
    fun goBack(): Boolean {
        return performGlobalAction(GLOBAL_ACTION_BACK)
    }

    /**
     * Go to the home screen.
     */
    fun goHome(): Boolean {
        return performGlobalAction(GLOBAL_ACTION_HOME)
    }

    /**
     * Refresh the current screen by performing a swipe down gesture.
     */
    fun refresh(callback: (Boolean) -> Unit) {
        val root = rootInActiveWindow ?: return callback(false)
        val bounds = Rect().also { root.getBoundsInScreen(it) }
        root.recycle()

        val startX = bounds.centerX()
        val startY = bounds.top + 50
        val endY = bounds.bottom - 50

        swipe(startX, startY, startX, endY, 500L, callback)
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

    // ── Legacy helpers ──────────────────────────────────────────

    /**
     * Dumps the current window's UI tree as an indented string.
     * Each line uses `|-- ` indentation per depth level.
     */
    fun getScreenContent(): String {
        val root = rootInActiveWindow ?: return ""
        return nodeToString(root, 0).also { root.recycle() }
    }

    /**
     * Performs ACTION_SET_TEXT on the currently focused editable node.
     */
    fun typeText(text: String) {
        inputText(text)
    }

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
