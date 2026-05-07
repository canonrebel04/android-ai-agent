package com.yourdomain.agent

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDropDown
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenu
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel


enum class ProviderType(val displayName: String) {
    OPENROUTER("OpenRouter"),
    ANTHROPIC("Anthropic"),
    GOOGLE("Google"),
    MISTRAL("Mistral"),
    DEEPSEEK("DeepSeek"),
    LOCAL("Local")
}

val providerModels = mapOf(
    ProviderType.OPENROUTER to listOf(
        "openai/gpt-4o",
        "openai/gpt-4o-mini",
        "anthropic/claude-3-7-sonnet",
        "anthropic/claude-3-5-sonnet",
        "anthropic/claude-3-haiku",
        "google/gemini-flash-1.5",
        "google/gemini-pro-1.5",
        "mistral/mistral-large",
        "mistral/mistral-small",
        "deepseek/deepseek-v4",
        "deepseek/deepseek-v4-pro",
        "meta-llama/llama-3.2-11b",
        "meta-llama/llama-3.2-3b",
    ),
    ProviderType.ANTHROPIC to listOf(
        "claude-3-7-sonnet-20250219",
        "claude-3-5-sonnet-20241022",
        "claude-3-opus-20240229",
        "claude-3-sonnet-20240229",
        "claude-3-haiku-20240307",
    ),
    ProviderType.GOOGLE to listOf(
        "gemini-1.5-flash",
        "gemini-1.5-pro",
        "gemini-2.0-flash",
        "gemini-2.0-pro",
    ),
    ProviderType.MISTRAL to listOf(
        "mistral-large",
        "mistral-small",
        "mistral-tiny",
        "codestral-latest",
        "mistral-medium",
    ),
    ProviderType.DEEPSEEK to listOf(
        "deepseek-v4",
        "deepseek-v4-pro",
        "deepseek-v4-ultra",
        "deepseek-coder",
        "deepseek-chat",
    ),
    ProviderType.LOCAL to listOf(
        "local-llama-3.2-3b",
        "local-mistral-7b",
        "local-phi-4-mini",
        "local-gemma-2b",
    )
)


@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ModelSetupScreen(
    viewModel: ModelSetupViewModel = viewModel(),
    onContinue: () -> Unit
) {
    val uiState by viewModel.uiState.collectAsState()
    var showAdvanced by remember { mutableStateOf(false) }
    var providerExpanded by remember { mutableStateOf(false) }
    var modelExpanded by remember { mutableStateOf(false) }

    val providers = ProviderType.entries
    val models = providerModels[uiState.selectedProvider] ?: emptyList()

    // Initialize model if empty
    LaunchedEffect(uiState.selectedProvider) {
        if (uiState.selectedModel.isEmpty() && models.isNotEmpty()) {
            viewModel.setModel(models[0])
        }
    }

    val isValid = uiState.isValid && uiState.selectedModel.isNotEmpty() &&
        (uiState.selectedProvider == ProviderType.LOCAL || uiState.apiKey.isNotBlank()) &&
        uiState.maxTokensError == null && uiState.temperatureError == null

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("Model Setup")
                        Text(
                            text = "Configure your AI provider and model",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                ),
            )
        },
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 16.dp, vertical = 24.dp),
            verticalArrangement = Arrangement.spacedBy(20.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(
                text = "Select AI Provider",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.primary,
            )

            ProviderDropdown(
                selectedProvider = uiState.selectedProvider,
                onProviderSelected = { provider ->
                    viewModel.setProvider(provider)
                    providerExpanded = false
                },
                providerExpanded = providerExpanded,
                onExpandedChange = { providerExpanded = it },
                providers = providers,
                modifier = Modifier.fillMaxWidth(0.9f)
            )

            Spacer(modifier = Modifier.height(8.dp))

            Text(
                text = "Select Model",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.primary,
            )

            ModelDropdown(
                selectedModel = uiState.selectedModel,
                onModelSelected = { model ->
                    viewModel.setModel(model)
                    modelExpanded = false
                },
                modelExpanded = modelExpanded,
                onExpandedChange = { modelExpanded = it },
                models = models,
                modifier = Modifier.fillMaxWidth(0.9f)
            )

            Spacer(modifier = Modifier.height(8.dp))

            Text(
                text = if (uiState.selectedProvider == ProviderType.LOCAL) "API Key (Optional)" else "API Key *",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.primary,
            )

            OutlinedTextField(
                value = uiState.apiKey,
                onValueChange = { viewModel.setApiKey(it) },
                label = { Text("Enter your API key") },
                placeholder = { Text("sk-..." + if (uiState.selectedProvider == ProviderType.GOOGLE) " (Google AI Studio key)" else "") },
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(
                    keyboardType = KeyboardType.Password,
                    imeAction = ImeAction.Done
                ),
                singleLine = true,
                modifier = Modifier.fillMaxWidth(0.9f),
                enabled = uiState.selectedProvider != ProviderType.LOCAL,
            )
            
            // SECURITY NOTE: API keys are now stored securely using Android Keystore
            if (uiState.selectedProvider != ProviderType.LOCAL && uiState.apiKey.isNotBlank()) {
                Text(
                    text = "Note: API key is stored securely using Android Keystore.",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }
            
            if (uiState.selectedProvider != ProviderType.LOCAL && uiState.apiKey.isBlank()) {
                Text(
                    text = "API key is required for this provider",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.error,
                )
            }

            Spacer(modifier = Modifier.height(16.dp))

            AdvancedSettingsCard(
                showAdvanced = showAdvanced,
                onToggle = { showAdvanced = !showAdvanced },
                maxTokens = uiState.maxTokens,
                onMaxTokensChange = { viewModel.setMaxTokens(it) },
                temperature = uiState.temperature,
                onTemperatureChange = { viewModel.setTemperature(it) },
                modifier = Modifier.fillMaxWidth(0.9f),
                maxTokensError = uiState.maxTokensError,
                temperatureError = uiState.temperatureError,
            )

            Spacer(modifier = Modifier.height(24.dp))

            Button(
                onClick = {
                    if (viewModel.validateInputs()) {
                        onContinue()
                    }
                },
                enabled = isValid,
                modifier = Modifier
                    .fillMaxWidth(0.9f)
                    .height(56.dp),
            ) {
                Text(
                    text = "Continue",
                    style = MaterialTheme.typography.labelLarge,
                    fontWeight = FontWeight.Bold,
                )
            }
        }
    }
}


@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ProviderDropdown(
    selectedProvider: ProviderType,
    onProviderSelected: (ProviderType) -> Unit,
    providerExpanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    providers: List<ProviderType>,
    modifier: Modifier = Modifier,
) {
    ExposedDropdownMenuBox(
        expanded = providerExpanded,
        onExpandedChange = onExpandedChange,
        modifier = modifier,
    ) {
        OutlinedTextField(
            value = selectedProvider.displayName,
            onValueChange = {},
            readOnly = true,
            label = { Text("Provider") },
            trailingIcon = {
                Icon(
                    imageVector = Icons.Default.ArrowDropDown,
                    contentDescription = "Select provider",
                )
            },
            modifier = Modifier
                .fillMaxWidth()
                .menuAnchor(),
        )
        ExposedDropdownMenu(
            expanded = providerExpanded,
            onDismissRequest = { onExpandedChange(false) },
            modifier = Modifier.fillMaxWidth(),
        ) {
            providers.forEach { provider ->
                DropdownMenuItem(
                    text = { Text(provider.displayName) },
                    onClick = { onProviderSelected(provider) },
                    leadingIcon = {
                        if (provider == selectedProvider) {
                            Icon(
                                imageVector = Icons.Default.Check,
                                contentDescription = "Selected",
                            )
                        }
                    },
                )
            }
        }
    }
}


@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ModelDropdown(
    selectedModel: String,
    onModelSelected: (String) -> Unit,
    modelExpanded: Boolean,
    onExpandedChange: (Boolean) -> Unit,
    models: List<String>,
    modifier: Modifier = Modifier,
) {
    ExposedDropdownMenuBox(
        expanded = modelExpanded,
        onExpandedChange = onExpandedChange,
        modifier = modifier,
    ) {
        OutlinedTextField(
            value = selectedModel,
            onValueChange = {},
            readOnly = true,
            label = { Text("Model") },
            trailingIcon = {
                Icon(
                    imageVector = Icons.Default.ArrowDropDown,
                    contentDescription = "Select model",
                )
            },
            modifier = Modifier
                .fillMaxWidth()
                .menuAnchor(),
        )
        ExposedDropdownMenu(
            expanded = modelExpanded,
            onDismissRequest = { onExpandedChange(false) },
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(max = 300.dp),
        ) {
            if (models.isEmpty()) {
                DropdownMenuItem(
                    text = { Text("No models available for this provider") },
                    onClick = {},
                    enabled = false,
                )
            } else {
                models.forEach { model ->
                    DropdownMenuItem(
                        text = { Text(model) },
                        onClick = { onModelSelected(model) },
                        leadingIcon = {
                            if (model == selectedModel) {
                                Icon(
                                    imageVector = Icons.Default.Check,
                                    contentDescription = "Selected",
                                )
                            }
                        },
                    )
                }
            }
        }
    }
}


@Composable
private fun AdvancedSettingsCard(
    showAdvanced: Boolean,
    onToggle: () -> Unit,
    maxTokens: String,
    onMaxTokensChange: (String) -> Unit,
    temperature: String,
    onTemperatureChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    maxTokensError: String? = null,
    temperatureError: String? = null,
) {
    Card(
        onClick = onToggle,
        modifier = modifier,
        colors = CardDefaults.cardColors(
            containerColor = MaterialTheme.colorScheme.surfaceContainer,
        ),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = "Advanced Settings",
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                )
                Icon(
                    imageVector = if (showAdvanced) Icons.Default.ExpandLess else Icons.Default.ExpandMore,
                    contentDescription = if (showAdvanced) "Collapse" else "Expand",
                )
            }

            if (showAdvanced) {
                Spacer(modifier = Modifier.height(12.dp))

                OutlinedTextField(
                    value = maxTokens,
                    onValueChange = { newValue ->
                        if (newValue.all { it.isDigit() }) {
                            onMaxTokensChange(newValue)
                        }
                    },
                    label = { Text("Max Tokens") },
                    placeholder = { Text("4096") },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    isError = maxTokensError != null,
                )
                
                if (maxTokensError != null) {
                    Text(
                        text = maxTokensError!!,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.error,
                        modifier = Modifier.padding(start = 16.dp, top = 4.dp)
                    )
                }

                Spacer(modifier = Modifier.height(8.dp))

                OutlinedTextField(
                    value = temperature,
                    onValueChange = { newValue ->
                        if (newValue.all { it.isDigit() || it == '.' }) {
                            onTemperatureChange(newValue)
                        }
                    },
                    label = { Text("Temperature") },
                    placeholder = { Text("0.7") },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    isError = temperatureError != null,
                )
                
                if (temperatureError != null) {
                    Text(
                        text = temperatureError!!,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.error,
                        modifier = Modifier.padding(start = 16.dp, top = 4.dp)
                    )
                }

                Spacer(modifier = Modifier.height(4.dp))
                Text(
                    text = "Temperature controls randomness (0.0 = deterministic, 1.0 = creative)",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}


@Composable
fun ModelSetupScreenPreview() {
    MaterialTheme {
        ModelSetupScreen(onContinue = {})
    }
}