package com.yourdomain.agent

interface ChatBridge {
    fun sendMessage(message: ChatMessage): String
    fun getHistory(): List<ChatMessage>
    fun onMessageReceived(message: ChatMessage)
}
