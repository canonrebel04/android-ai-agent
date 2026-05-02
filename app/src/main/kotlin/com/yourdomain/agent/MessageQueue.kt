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

class MessageQueue(val maxSize: Int = 20) {
    private val _messages = MutableStateFlow<List<MessageItem>>(emptyList())
    val messages: StateFlow<List<MessageItem>> = _messages.asStateFlow()

    fun add(sender: String, content: String, packageName: String) {
        val item = MessageItem(sender, content, packageName)
        val current = _messages.value.toMutableList()
        current.add(0, item)
        if (current.size > maxSize) current.removeAt(current.size - 1)
        _messages.value = current
    }

    fun clear() { _messages.value = emptyList() }
    fun size(): Int = _messages.value.size
}
