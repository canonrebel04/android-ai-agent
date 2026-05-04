package com.yourdomain.agent

import org.junit.Test
import org.junit.Assert.*

class ChatMessageTest {
    @Test
    fun testChatMessageCreation() {
        val message = ChatMessage(
            id = "1",
            role = "user",
            content = "Hello world",
            timestamp = 123456789L
        )
        assertEquals("1", message.id)
        assertEquals("user", message.role)
        assertEquals("Hello world", message.content)
        assertEquals(123456789L, message.timestamp)
    }
}
