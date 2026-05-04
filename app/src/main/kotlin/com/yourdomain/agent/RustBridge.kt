package com.yourdomain.agent

object RustBridge {
    init { System.loadLibrary("agent_core") }
    external fun nativeInit(openrouterKey: String): String
    external fun nativeRun(prompt: String): String
    external fun nativeStatus(): String
    external fun nativeGetLogs(count: Int): String
    external fun nativeConfirm(approved: Boolean): String

    // Unified Chat (Phase 1)
    external fun sendMessage(json: String): String
    external fun getHistory(): String

    // Budget tracker (Phase 6)
    external fun getMonthlyCost(): String
    external fun setBudgetThreshold(usd: String): String
    external fun isOverBudget(): Boolean
}
