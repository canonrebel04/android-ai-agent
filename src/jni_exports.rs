// JNI exports for the Kotlin bridge on Android.
// The android submodule is only compiled when targeting Android;
// a stub module is provided for host compilation.

#[cfg(target_os = "android")]
pub mod android {
    use crate::complexity_classifier;
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jint, jstring};
    use jni::JNIEnv;

    /// Initialize the agent with an OpenRouter API key.
    /// Returns a status string with a masked key suffix.
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_bridge_RustBridge_nativeInit(
        mut env: JNIEnv,
        _class: JClass,
        openrouter_key: JString,
    ) -> jstring {
        let key: String = match env.get_string(&openrouter_key) {
            Ok(s) => s.into(),
            Err(_) => return env.new_string("init_err:bad_string").unwrap().into_raw(),
        };
        let suffix = if key.len() >= 4 {
            &key[key.len().saturating_sub(4)..]
        } else {
            "????"
        };
        let result = format!("init_ok:{}", suffix);
        env.new_string(&result).unwrap().into_raw()
    }

    /// Classify a prompt and return the complexity level along with a processing message.
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_bridge_RustBridge_nativeRun(
        mut env: JNIEnv,
        _class: JClass,
        prompt: JString,
    ) -> jstring {
        let input: String = match env.get_string(&prompt) {
            Ok(s) => s.into(),
            Err(_) => return env.new_string("[Error] Processing: invalid prompt").unwrap().into_raw(),
        };
        let complexity = complexity_classifier::classify(&input);
        let result = format!("[{:?}] Processing: {}", complexity, input);
        env.new_string(&result).unwrap().into_raw()
    }

    /// Report the agent's current operational status.
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_bridge_RustBridge_nativeStatus(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jstring {
        env.new_string("idle").unwrap().into_raw()
    }

    /// Retrieve the last N log entries (simulated).
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_bridge_RustBridge_nativeGetLogs(
        mut env: JNIEnv,
        _class: JClass,
        count: jint,
    ) -> jstring {
        let n = count.max(0) as usize;
        let mut logs = String::new();
        for i in 0..n {
            logs.push_str(&format!("[log] entry {} of {}\n", i + 1, n));
        }
        // Trim trailing newline for cleanliness.
        let trimmed = logs.trim_end();
        env.new_string(trimmed).unwrap().into_raw()
    }

    /// Confirm or reject a pending action.
    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_bridge_RustBridge_nativeConfirm(
        mut env: JNIEnv,
        _class: JClass,
        approved: jboolean,
    ) -> jstring {
        let msg = if approved != 0 { "confirmed" } else { "rejected" };
        env.new_string(msg).unwrap().into_raw()
    }
}

#[cfg(not(target_os = "android"))]
pub mod android {
    // Stub module so the crate compiles on host (Linux, macOS, Windows) without JNI.
    // All functions are no-ops — they cannot be called from non-Android targets.
}
