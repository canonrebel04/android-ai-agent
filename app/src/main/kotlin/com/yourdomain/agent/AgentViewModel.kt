package com.yourdomain.agent

import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONArray

data class AgentState(
    val status: String = "idle",
    val currentTask: String = "",
    val logLines: List<String> = emptyList(),
    val chatMessages: List<ChatMessage> = emptyList(),
    val pendingConfirmation: String? = null,
    val activeModel: String = "claude-3-5-sonnet",
    val monthlyCost: String = "$0.00"
)

class AgentViewModel(private val context: Context? = null) : ViewModel(), MessageQueueListener {
    private val _state = MutableStateFlow(AgentState())
    val state: StateFlow<AgentState> = _state

    init {
        refreshHistory()
        refreshBudget()
        // Register as a listener for message queue changes
        GlobalMessageQueue.addListener(this)
    }

    override fun onCleared() {
        super.onCleared()
        // Unregister from message queue
        GlobalMessageQueue.removeListener(this)
    }

    fun initAgent(openrouterKey: String) {
        viewModelScope.launch { withContext(Dispatchers.IO) { RustBridge.nativeInit(openrouterKey) } }
    }

    fun sendChatMessage(text: String) {
        val userMsg = ChatMessage(
            id = System.currentTimeMillis().toString(),
            role = "user",
            content = text,
            timestamp = System.currentTimeMillis()
        )
        _state.value = _state.value.copy(
            chatMessages = _state.value.chatMessages + userMsg,
            status = "running"
        )

        viewModelScope.launch {
            val response = withContext(Dispatchers.IO) {
                // In Phase 1 we use a simple JSON string for the bridge
                val json = "{\"id\":\"${userMsg.id}\",\"role\":\"user\",\"content\":\"${userMsg.content}\",\"timestamp\":${userMsg.timestamp}}"
                RustBridge.sendMessage(json)
            }

            val agentMsg = ChatMessage(
                id = (System.currentTimeMillis() + 1).toString(),
                role = "assistant",
                content = response,
                timestamp = System.currentTimeMillis()
            )
            _state.value = _state.value.copy(
                chatMessages = _state.value.chatMessages + agentMsg,
                status = "idle"
            )
            refreshBudget()
        }
    }

    fun refreshHistory() {
        viewModelScope.launch {
            val historyJson = withContext(Dispatchers.IO) { RustBridge.getHistory() }
            try {
                val jsonArray = JSONArray(historyJson)
                val messages = List(jsonArray.length()) { i ->
                    val obj = jsonArray.getJSONObject(i)
                    ChatMessage(
                        id = obj.getString("id"),
                        role = obj.getString("role"),
                        content = obj.getString("content"),
                        timestamp = obj.getLong("timestamp")
                    )
                }
                _state.value = _state.value.copy(chatMessages = messages)
            } catch (e: Exception) {
                Log.e("AgentViewModel", "Error parsing history JSON", e)
            }
        }
    }

    fun refreshBudget() {
        viewModelScope.launch {
            val cost = withContext(Dispatchers.IO) { RustBridge.getMonthlyCost() }
            _state.value = _state.value.copy(monthlyCost = cost)
        }
    }

    fun stopTask() { _state.value = _state.value.copy(status = "idle") }

    fun refreshStatus() {
        viewModelScope.launch { withContext(Dispatchers.IO) {
            _state.value = _state.value.copy(status = RustBridge.nativeStatus())
        }}
    }

    fun refreshLogs(count: Int = 20) {
        viewModelScope.launch { withContext(Dispatchers.IO) {
            val logs = RustBridge.nativeGetLogs(count)
            _state.value = _state.value.copy(logLines = logs.split("\n").filter { it.isNotBlank() })
        }}
    }

    fun confirmAction(approved: Boolean) {
        viewModelScope.launch { withContext(Dispatchers.IO) {
            RustBridge.nativeConfirm(approved)
            _state.value = _state.value.copy(pendingConfirmation = null, status = "running")
        }}
    }

    // MessageQueueListener callbacks
    override fun onMessageAdded(message: MessageItem) {
        // Update state to reflect new message count
        updateMessageQueueState()
    }

    override fun onMessageProcessed(message: MessageItem) {
        updateMessageQueueState()
    }

    override fun onMessageCleared() {
        updateMessageQueueState()
    }

    override fun onAllMessagesProcessed() {
        updateMessageQueueState()
    }

    private fun updateMessageQueueState() {
        viewModelScope.launch {
            _state.value = _state.value.copy(
                // Keep existing state but this triggers recomposition
            )
        }
    }

    // Message queue processing methods
    fun processNextNotification(): Boolean {
        val unprocessed = GlobalMessageQueue.getUnprocessedMessages()
        if (unprocessed.isEmpty()) {
            return false
        }
        
        val message = unprocessed.first()
        // Process the message (mark as processed and potentially send to agent)
        GlobalMessageQueue.markAsProcessed(message)
        
        // Send the message content to the agent for processing
        sendMessageToAgent(message)
        
        return true
    }

    fun processAllNotifications() {
        val unprocessed = GlobalMessageQueue.getUnprocessedMessages()
        for (message in unprocessed) {
            sendMessageToAgent(message)
            GlobalMessageQueue.markAsProcessed(message)
        }
    }

    private fun sendMessageToAgent(message: MessageItem) {
        val userMsg = ChatMessage(
            id = "notif_${message.timestamp}_${message.sender.hashCode()}",
            role = "user",
            content = "Notification from ${message.sender}: ${message.content}",
            timestamp = message.timestamp
        )
        _state.value = _state.value.copy(
            chatMessages = _state.value.chatMessages + userMsg,
            status = "running"
        )

        viewModelScope.launch {
            val response = withContext(Dispatchers.IO) {
                val json = "{\"id\":\"${userMsg.id}\",\"role\":\"user\",\"content\":\"${userMsg.content}\",\"timestamp\":${userMsg.timestamp}}"
                RustBridge.sendMessage(json)
            }

            val agentMsg = ChatMessage(
                id = "${userMsg.id}_response",
                role = "assistant",
                content = response,
                timestamp = System.currentTimeMillis()
            )
            _state.value = _state.value.copy(
                chatMessages = _state.value.chatMessages + agentMsg,
                status = "idle"
            )
            refreshBudget()
        }
    }

    // Floating overlay control methods
    fun showFloatingOverlay() {
        context?.let { ctx ->
            val intent = Intent(ctx, FloatingAgentOverlay::class.java).apply {
                putExtra("status", _state.value.status)
                putExtra("action", _state.value.currentTask)
            }
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
                ctx.startForegroundService(intent)
            } else {
                ctx.startService(intent)
            }
        }
    }

    fun updateFloatingOverlay(status: String, action: String) {
        context?.let { ctx ->
            val intent = Intent(ctx, FloatingAgentOverlay::class.java).apply {
                putExtra("status", status)
                putExtra("action", action)
                action = "UPDATE_OVERLAY"
            }
            ctx.startService(intent)
        }
    }
}
