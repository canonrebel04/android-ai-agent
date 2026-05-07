package com.yourdomain.agent

object RustBridge {
    init { System.loadLibrary("agent_core") }
    external fun nativeInit(openrouterKey: String): String
    external fun nativeRun(prompt: String): String
    external fun nativeStatus(): String
    external fun nativeGetLogs(count: Int): String
    external fun nativeConfirm(approved: Boolean): String
    external fun classifyPrompt(prompt: String): String
    external fun getModelPricing(modelId: String): String
    external fun getAllModelPricing(): String
    external fun estimateCost(modelId: String, inputTokens: Int, outputTokens: Int): String

    // Unified Chat (Phase 1)
    external fun sendMessage(json: String): String
    external fun getHistory(): String
}
