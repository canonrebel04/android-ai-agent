package com.yourdomain.agent.ui

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
    val pendingConfirmation: String? = null,
)

class AgentViewModel : ViewModel() {
    private val _state = MutableStateFlow(AgentState())
    val state: StateFlow<AgentState> = _state

    fun initAgent(openrouterKey: String) {
        viewModelScope.launch { withContext(Dispatchers.IO) { RustBridge.nativeInit(openrouterKey) } }
    }

    fun startTask(prompt: String) {
        _state.value = _state.value.copy(status = "running", currentTask = prompt)
        viewModelScope.launch { withContext(Dispatchers.IO) {
            val result = RustBridge.nativeRun(prompt)
            _state.value = _state.value.copy(status = "idle", logLines = _state.value.logLines + result)
        }}
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
}
