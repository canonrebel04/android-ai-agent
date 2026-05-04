package com.yourdomain.agent

data class ChatMessage(
    val id: String,
    val role: String,
    val content: String,
    val timestamp: Long
)
