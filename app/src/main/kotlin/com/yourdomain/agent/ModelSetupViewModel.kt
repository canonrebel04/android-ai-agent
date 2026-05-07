package com.yourdomain.agent

import android.content.Context
import android.content.SharedPreferences
import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * ViewModel for managing model setup configuration.
 * Handles state management, validation, and persistence of model configuration.
 */
class ModelSetupViewModel(
    private val context: Context
) : ViewModel() {

    // State for model configuration
    private val _uiState = MutableStateFlow(ModelSetupState())
    val uiState: StateFlow<ModelSetupState> = _uiState.asStateFlow()

    // SharedPreferences for persistence
    private val prefs: SharedPreferences = context.getSharedPreferences("model_setup_prefs", Context.MODE_PRIVATE)

    // Keystore for secure API key storage
    private val keystoreManager = KeystoreManager(context)

    // Constants for SharedPreferences keys
    companion object {
        private const val KEY_PROVIDER = "selected_provider"
        private const val KEY_MODEL = "selected_model"
        private const val KEY_API_KEY = "api_key_encrypted"
        private const val KEY_MAX_TOKENS = "max_tokens"
        private const val KEY_TEMPERATURE = "temperature"
        private const val KEY_CONFIG_VALIDATED = "config_validated"
    }

    init {
        loadConfig()
    }

    /**
     * Update the selected provider
     */
    fun setProvider(provider: ProviderType) {
        _uiState.value = _uiState.value.copy(
            selectedProvider = provider,
            selectedModel = providerModels[provider]?.firstOrNull() ?: "",
            maxTokensError = null,
            temperatureError = null
        )
    }

    /**
     * Update the selected model
     */
    fun setModel(model: String) {
        _uiState.value = _uiState.value.copy(
            selectedModel = model,
            maxTokensError = null,
            temperatureError = null
        )
    }

    /**
     * Update the API key
     */
    fun setApiKey(apiKey: String) {
        _uiState.value = _uiState.value.copy(
            apiKey = apiKey,
            maxTokensError = null,
            temperatureError = null
        )
        // Don't save to SavedStateHandle - API keys should be secure
    }

    /**
     * Update max tokens
     */
    fun setMaxTokens(maxTokens: String) {
        _uiState.value = _uiState.value.copy(
            maxTokens = maxTokens,
            maxTokensError = null
        )
    }

    /**
     * Update temperature
     */
    fun setTemperature(temperature: String) {
        _uiState.value = _uiState.value.copy(
            temperature = temperature,
            temperatureError = null
        )
    }

    /**
     * Validate all inputs
     * Returns true if all inputs are valid
     */
    fun validateInputs(): Boolean {
        val currentState = _uiState.value
        var isValid = true
        var maxTokensError: String? = null
        var temperatureError: String? = null

        // Validate maxTokens
        maxTokensError = try {
            val tokens = currentState.maxTokens.toInt()
            if (tokens < 1 || tokens > 32768) {
                isValid = false
                "Max tokens must be between 1 and 32768"
            } else {
                null
            }
        } catch (e: NumberFormatException) {
            isValid = false
            "Max tokens must be a valid number"
        }

        // Validate temperature
        temperatureError = try {
            val temp = currentState.temperature.toDouble()
            if (temp < 0.0 || temp > 2.0) {
                isValid = false
                "Temperature must be between 0.0 and 2.0"
            } else {
                null
            }
        } catch (e: NumberFormatException) {
            isValid = false
            "Temperature must be a valid number"
        }

        // Check if model is selected
        if (currentState.selectedModel.isBlank()) {
            isValid = false
        }

        // Check if API key is required for non-local providers
        if (currentState.selectedProvider != ProviderType.LOCAL && currentState.apiKey.isBlank()) {
            isValid = false
        }

        // Validate through RustBridge if available
        try {
            val rustValidation = validateConfigWithRustBridge()
            if (!rustValidation) {
                isValid = false
            }
        } catch (e: Exception) {
            Log.w("ModelSetupViewModel", "RustBridge validation failed, using local validation", e)
        }

        _uiState.value = currentState.copy(
            maxTokensError = maxTokensError,
            temperatureError = temperatureError,
            isValid = isValid
        )

        return isValid
    }

    /**
     * Validate configuration using RustBridge
     * Placeholder for future Rust-based validation
     */
    private fun validateConfigWithRustBridge(): Boolean {
        // For now, return true as RustBridge doesn't have config validation yet
        // This is a placeholder for future integration
        return true
    }

    /**
     * Save the current configuration to SharedPreferences
     * API keys are stored securely using Keystore
     */
    fun saveConfig() {
        val currentState = _uiState.value

        // Validate before saving
        if (!validateInputs()) {
            Log.e("ModelSetupViewModel", "Cannot save invalid configuration")
            return
        }

        with(prefs.edit()) {
            // Save provider
            putString(KEY_PROVIDER, currentState.selectedProvider.name)
            
            // Save model
            putString(KEY_MODEL, currentState.selectedModel)
            
            // Save max tokens
            putString(KEY_MAX_TOKENS, currentState.maxTokens)
            
            // Save temperature
            putString(KEY_TEMPERATURE, currentState.temperature)
            
            // Mark config as validated
            putBoolean(KEY_CONFIG_VALIDATED, true)
            
            apply()
        }

        // Save API key securely if it's not empty
        if (currentState.apiKey.isNotBlank()) {
            keystoreManager.saveApiKey(KEY_API_KEY, currentState.apiKey)
        }

        _uiState.value = currentState.copy(configSaved = true)
        Log.d("ModelSetupViewModel", "Configuration saved successfully")
    }

    /**
     * Load configuration from SharedPreferences
     */
    fun loadConfig() {
        val providerName = prefs.getString(KEY_PROVIDER, ProviderType.OPENROUTER.name) ?: ProviderType.OPENROUTER.name
        val provider = try {
            ProviderType.valueOf(providerName)
        } catch (e: IllegalArgumentException) {
            ProviderType.OPENROUTER
        }

        val model = prefs.getString(KEY_MODEL, "") ?: ""
        val maxTokens = prefs.getString(KEY_MAX_TOKENS, "4096") ?: "4096"
        val temperature = prefs.getString(KEY_TEMPERATURE, "0.7") ?: "0.7"
        val configValidated = prefs.getBoolean(KEY_CONFIG_VALIDATED, false)

        // Load API key from secure storage
        val apiKey = keystoreManager.getApiKey(KEY_API_KEY) ?: ""

        // If model is empty, use the first model for the provider
        val finalModel = if (model.isBlank()) {
            providerModels[provider]?.firstOrNull() ?: ""
        } else {
            model
        }

        _uiState.value = ModelSetupState(
            selectedProvider = provider,
            selectedModel = finalModel,
            apiKey = apiKey,
            maxTokens = maxTokens,
            temperature = temperature,
            configSaved = configValidated,
            maxTokensError = null,
            temperatureError = null,
            isValid = configValidated
        )

        Log.d("ModelSetupViewModel", "Configuration loaded successfully")
    }

    /**
     * Clear all configuration
     */
    fun clearConfig() {
        with(prefs.edit()) {
            remove(KEY_PROVIDER)
            remove(KEY_MODEL)
            remove(KEY_MAX_TOKENS)
            remove(KEY_TEMPERATURE)
            remove(KEY_CONFIG_VALIDATED)
            apply()
        }

        // Clear API key from secure storage
        keystoreManager.saveApiKey(KEY_API_KEY, "")

        _uiState.value = ModelSetupState(
            selectedProvider = ProviderType.OPENROUTER,
            selectedModel = providerModels[ProviderType.OPENROUTER]?.firstOrNull() ?: "",
            apiKey = "",
            maxTokens = "4096",
            temperature = "0.7",
            configSaved = false,
            maxTokensError = null,
            temperatureError = null,
            isValid = false
        )

        Log.d("ModelSetupViewModel", "Configuration cleared")
    }

    /**
     * Check if configuration is complete and valid
     */
    fun isConfigComplete(): Boolean {
        return _uiState.value.configSaved && _uiState.value.isValid
    }

    /**
     * Get the current configuration as a ModelConfig object for easy access
     */
    fun getCurrentConfig(): ModelConfig {
        return ModelConfig(
            provider = _uiState.value.selectedProvider,
            model = _uiState.value.selectedModel,
            apiKey = _uiState.value.apiKey,
            maxTokens = _uiState.value.maxTokens.toIntOrNull() ?: 4096,
            temperature = _uiState.value.temperature.toDoubleOrNull() ?: 0.7
        )
    }
}

/**
 * State class for model setup UI
 */
data class ModelSetupState(
    val selectedProvider: ProviderType = ProviderType.OPENROUTER,
    val selectedModel: String = "",
    val apiKey: String = "",
    val maxTokens: String = "4096",
    val temperature: String = "0.7",
    val configSaved: Boolean = false,
    val maxTokensError: String? = null,
    val temperatureError: String? = null,
    val isValid: Boolean = false
)

/**
 * Data class representing model configuration
 */
data class ModelConfig(
    val provider: ProviderType,
    val model: String,
    val apiKey: String,
    val maxTokens: Int,
    val temperature: Double
)
