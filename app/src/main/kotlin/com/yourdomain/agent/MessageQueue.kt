package com.yourdomain.agent

import java.util.concurrent.CopyOnWriteArrayList

/**
 * Data class representing a message in the queue
 */
data class MessageItem(
    val sender: String,
    val content: String,
    val packageName: String,
    val timestamp: Long,
    var isProcessed: Boolean = false
)

/**
 * Interface for message queue listeners
 */
interface MessageQueueListener {
    fun onMessageAdded(message: MessageItem)
    fun onMessageProcessed(message: MessageItem)
    fun onMessageCleared()
    fun onAllMessagesProcessed()
}

/**
 * Thread-safe message queue using CopyOnWriteArrayList
 * Max capacity of 20 items - oldest items are removed when limit is reached
 */
class MessageQueue {
    private val messages = CopyOnWriteArrayList<MessageItem>()
    private val listeners = CopyOnWriteArrayList<MessageQueueListener>()
    
    companion object {
        const val MAX_MESSAGES = 20
    }

    /**
     * Add a new message to the queue
     * If the queue is at capacity, the oldest message is removed
     */
    fun addMessage(sender: String, content: String, packageName: String): MessageItem {
        val newMessage = MessageItem(
            sender = sender,
            content = content,
            packageName = packageName,
            timestamp = System.currentTimeMillis(),
            isProcessed = false
        )
        
        // Remove oldest if at capacity
        while (messages.size >= MAX_MESSAGES) {
            messages.removeAt(0)
        }
        
        messages.add(newMessage)
        notifyMessageAdded(newMessage)
        return newMessage
    }

    /**
     * Get all unprocessed messages
     */
    fun getUnprocessedMessages(): List<MessageItem> {
        return messages.filter { !it.isProcessed }
    }

    /**
     * Get all messages (both processed and unprocessed)
     */
    fun getAllMessages(): List<MessageItem> {
        return ArrayList(messages)
    }

    /**
     * Mark a specific message as processed
     */
    fun markAsProcessed(message: MessageItem) {
        val index = messages.indexOfFirst { it === message }
        if (index != -1) {
            val updated = messages[index].copy(isProcessed = true)
            messages[index] = updated
            notifyMessageProcessed(updated)
        }
    }

    /**
     * Mark all messages as processed
     */
    fun markAllAsProcessed() {
        var anyChanged = false
        messages.replaceAll(java.util.function.UnaryOperator { msg ->
            if (!msg.isProcessed) {
                anyChanged = true
                msg.copy(isProcessed = true)
            } else {
                msg
            }
        })
        if (anyChanged) {
            notifyAllMessagesProcessed()
        }
    }

    /**
     * Clear all messages from the queue
     */
    fun clear() {
        messages.clear()
        notifyMessageCleared()
    }

    /**
     * Get the count of unprocessed messages
     */
    fun getUnprocessedCount(): Int {
        return messages.count { !it.isProcessed }
    }

    /**
     * Get the total count of messages
     */
    fun getTotalCount(): Int {
        return messages.size
    }

    // Observer pattern methods

    /**
     * Add a listener to be notified of queue changes
     */
    fun addListener(listener: MessageQueueListener) {
        listeners.add(listener)
    }

    /**
     * Remove a listener
     */
    fun removeListener(listener: MessageQueueListener) {
        listeners.remove(listener)
    }

    private fun notifyMessageAdded(message: MessageItem) {
        for (listener in listeners) {
            try {
                listener.onMessageAdded(message)
            } catch (e: Exception) {
                // Don't let listener exceptions affect the queue
            }
        }
    }

    private fun notifyMessageProcessed(message: MessageItem) {
        for (listener in listeners) {
            try {
                listener.onMessageProcessed(message)
            } catch (e: Exception) {
                // Don't let listener exceptions affect the queue
            }
        }
    }

    private fun notifyAllMessagesProcessed() {
        for (listener in listeners) {
            try {
                listener.onAllMessagesProcessed()
            } catch (e: Exception) {
                // Don't let listener exceptions affect the queue
            }
        }
    }

    private fun notifyMessageCleared() {
        for (listener in listeners) {
            try {
                listener.onMessageCleared()
            } catch (e: Exception) {
                // Don't let listener exceptions affect the queue
            }
        }
    }
}

/**
 * Global singleton instance of the message queue
 */
object GlobalMessageQueue {
    private val queue = MessageQueue()

    fun addMessage(sender: String, content: String, packageName: String): MessageItem {
        return queue.addMessage(sender, content, packageName)
    }

    fun getUnprocessedMessages(): List<MessageItem> {
        return queue.getUnprocessedMessages()
    }

    fun getAllMessages(): List<MessageItem> {
        return queue.getAllMessages()
    }

    fun markAsProcessed(message: MessageItem) {
        queue.markAsProcessed(message)
    }

    fun markAllAsProcessed() {
        queue.markAllAsProcessed()
    }

    fun clear() {
        queue.clear()
    }

    fun getUnprocessedCount(): Int {
        return queue.getUnprocessedCount()
    }

    fun getTotalCount(): Int {
        return queue.getTotalCount()
    }

    fun addListener(listener: MessageQueueListener) {
        queue.addListener(listener)
    }

    fun removeListener(listener: MessageQueueListener) {
        queue.removeListener(listener)
    }
}
