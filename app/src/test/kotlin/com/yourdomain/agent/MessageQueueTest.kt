package com.yourdomain.agent

import org.junit.Test
import org.junit.Assert.*

class MessageQueueTest {
    @Test
    fun testMarkAllAsProcessed() {
        val queue = MessageQueue()
        queue.addMessage("sender1", "content1", "pkg1")
        queue.addMessage("sender2", "content2", "pkg2")

        assertEquals(2, queue.getUnprocessedCount())

        queue.markAllAsProcessed()

        assertEquals(0, queue.getUnprocessedCount())
        assertEquals(2, queue.getTotalCount())

        val messages = queue.getAllMessages()
        assertTrue(messages[0].isProcessed)
        assertTrue(messages[1].isProcessed)
    }
}
