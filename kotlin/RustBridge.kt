package com.yourdomain.agent.bridge

object RustBridge {
    init { System.loadLibrary("agent_core") }
    external fun nativeInit(openrouterKey: String): String
    external fun nativeRun(prompt: String): String
    external fun nativeStatus(): String
    external fun nativeGetLogs(count: Int): String
    external fun nativeConfirm(approved: Boolean): String
}
