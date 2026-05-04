// JNI exports for the Kotlin bridge on Android.
// Uses jni 0.22 API: EnvUnowned with with_env() pattern.

#[cfg(target_os = "android")]
pub mod android {
    use crate::complexity_classifier;
    use jni::objects::{JClass, JString};
    use jni::sys::{jboolean, jint, jstring};
    use jni::EnvUnowned;

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeInit<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        openrouter_key: JString<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let key: String = env.get_string(&openrouter_key)?.into();
            crate::unified_agent::get_agent().init(key);
            Ok(env.new_string("Agent initialized").unwrap().into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeRun<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        prompt: JString<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let input: String = env.get_string(&prompt)?.into();
            let complexity = complexity_classifier::classify(&input);
            let result = format!("[{:?}] Processing: {}", complexity, input);
            Ok(env.new_string(&result)?.into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeStatus<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            Ok(env.new_string("idle")?.into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeGetLogs<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        count: jint,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let n = (count as usize).min(100);
            let logs: String = (0..n)
                .map(|i| format!("[log] entry {} of {}", i + 1, n))
                .collect::<Vec<_>>()
                .join("\n");
            Ok(env.new_string(&logs)?.into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_nativeConfirm<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        approved: jboolean,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let approved_bool = approved as u8 != 0;
            let msg = if approved_bool { "confirmed" } else { "rejected" };
            Ok(env.new_string(msg)?.into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_getMonthlyCost<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let cost = crate::budget_tracker::get_tracker().monthly_cost();
            Ok(env.new_string(&format!("{:.2}", cost))?.into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_setBudgetThreshold<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        usd: JString<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let input: String = env.get_string(&usd)?.into();
            if let Ok(val) = input.parse::<f64>() {
                crate::budget_tracker::get_tracker().set_threshold(val);
                Ok(env.new_string("ok")?.into_raw())
            } else {
                Ok(env.new_string("invalid_number")?.into_raw())
            }
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_isOverBudget<'local>(
        _unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
    ) -> jboolean {
        crate::budget_tracker::get_tracker().is_over_budget() as jboolean
    }

    // Unified Chat (Phase 1)

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_sendMessage<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
        json: JString<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            let input: String = env.get_string(&json)?.into();
            let msg: crate::chat_models::ChatMessage = serde_json::from_str(&input).map_err(|e| {
                jni::errors::Error::JavaException {
                    class: "java/lang/IllegalArgumentException".to_string(),
                    msg: format!("Invalid JSON: {}", e),
                }
            })?;

            // Execute async process_message on the current thread's runtime
            let response = tokio::runtime::Handle::current().block_on(async {
                crate::unified_agent::get_agent().process_message(msg.content).await
            });

            Ok(env.new_string(&response)?.into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }

    #[unsafe(no_mangle)]
    pub extern "system" fn Java_com_yourdomain_agent_RustBridge_getHistory<'local>(
        mut unowned_env: EnvUnowned<'local>,
        _class: JClass<'local>,
    ) -> jstring {
        let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
            // TODO: Fetch real history from memory_manager
            let dummy_history = vec![
                crate::chat_models::ChatMessage {
                    id: "0".to_string(),
                    role: "system".to_string(),
                    content: "Welcome to Unified Chat".to_string(),
                    timestamp: 0,
                }
            ];
            let json = serde_json::to_string(&dummy_history).unwrap();
            Ok(env.new_string(&json)?.into_raw())
        });
        outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
    }
}

#[cfg(not(target_os = "android"))]
pub mod android {
    // Stub module for host compilation
}
