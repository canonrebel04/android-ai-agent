package com.yourdomain.agent

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

data class AgentState(
    val status: String = "idle",
    val currentTask: String = "",
    val logLines: List<String> = emptyList(),
    val chatMessages: List<ChatMessage> = emptyList(),
    val pendingConfirmation: String? = null,
    val activeModel: String = "claude-3-5-sonnet",
    val monthlyCost: String = "$0.00"
)

class AgentViewModel : ViewModel() {
    private val _state = MutableStateFlow(AgentState())
    val state: StateFlow<AgentState> = _state

    init {
        refreshHistory()
        refreshBudget()
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
            // TODO: Parse JSON history. For now, just a dummy if empty.
            if (historyJson.contains("Unified Chat")) {
                // Dummy parsing or state update logic
            }
        }
    }

    fun refreshBudget() {
        viewModelScope.launch {
            val cost = withContext(Dispatchers.IO) { RustBridge.getMonthlyCost() }
            _state.value = _state.value.copy(monthlyCost = cost)
        }
    }
...
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
}
