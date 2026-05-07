package com.yourdomain.agent

import android.content.Context
import android.content.Intent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.Button
import androidx.compose.material3.Divider
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics

// ── Screen ────────────────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen() {
    var maxSteps by remember { mutableStateOf("50") }
    var actionDelay by remember { mutableStateOf("500") }
    var stallTimeout by remember { mutableStateOf("300") }
    var visionMode by remember { mutableStateOf(true) }
    var sendConfirmation by remember { mutableStateOf(true) }
    var deleteConfirmation by remember { mutableStateOf(true) }

    // Telegram configuration state
    var telegramEnabled by remember { mutableStateOf(false) }
    var telegramToken by remember { mutableStateOf("") }
    val context = LocalContext.current

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("Settings")
                        Text(
                            text = "Configure agent behaviour",
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
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            // Execution
            SectionHeader("Execution")

            NumericSetting(
                label = "Max Steps",
                value = maxSteps,
                onValueChange = { maxSteps = it },
                helperText = "Maximum number of agent steps per task (1-200)",
            )

            NumericSetting(
                label = "Action Delay (ms)",
                value = actionDelay,
                onValueChange = { actionDelay = it },
                helperText = "Delay between consecutive actions (100-5000 ms)",
            )

            NumericSetting(
                label = "Stall Timeout (s)",
                value = stallTimeout,
                onValueChange = { stallTimeout = it },
                helperText = "Seconds before detecting a stalled task (30-600)",
            )

            Spacer(modifier = Modifier.height(8.dp))
            SectionDivider()
            Spacer(modifier = Modifier.height(8.dp))

            // Interaction
            SectionHeader("Interaction")

            ToggleSetting(
                label = "Vision Mode",
                description = "Allow the agent to capture and analyze screen content",
                checked = visionMode,
                onCheckedChange = { visionMode = it },
            )

            ToggleSetting(
                label = "Send Confirmation",
                description = "Ask for confirmation before sending messages",
                checked = sendConfirmation,
                onCheckedChange = { sendConfirmation = it },
            )

            ToggleSetting(
                label = "Delete Confirmation",
                description = "Ask for confirmation before deleting files or data",
                checked = deleteConfirmation,
                onCheckedChange = { deleteConfirmation = it },
            )

            Spacer(modifier = Modifier.height(8.dp))
            SectionDivider()
            Spacer(modifier = Modifier.height(8.dp))

            // Telegram Configuration
            SectionHeader("Telegram Bot")

            ToggleSetting(
                label = "Enable Telegram Bot",
                description = "Allow remote control via Telegram",
                checked = telegramEnabled,
                onCheckedChange = { telegramEnabled = it },
            )

            OutlinedTextField(
                value = telegramToken,
                onValueChange = { telegramToken = it },
                label = { Text("Bot Token") },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                visualTransformation = PasswordVisualTransformation(),
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Text(
                text = "Enter your Telegram bot token from @BotFather",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(start = 4.dp, top = 2.dp, bottom = 12.dp),
            )

            Button(
                onClick = {
                    if (telegramEnabled && telegramToken.isNotBlank()) {
                        val intent = Intent(context, TelegramBotService::class.java).apply {
                            putExtra(TelegramBotService.EXTRA_BOT_TOKEN, telegramToken)
                        }
                        val prefs = context.getSharedPreferences("AgentPrefs", Context.MODE_PRIVATE)
                        prefs.edit().putString("telegramToken", telegramToken).apply()
                        prefs.edit().putBoolean("telegramEnabled", true).apply()
                        context.startService(intent)
                    }
                },
                modifier = Modifier.fillMaxWidth(),
                enabled = telegramEnabled && telegramToken.isNotBlank()
            ) {
                Icon(Icons.Default.Send, contentDescription = "Start")
                Spacer(modifier = Modifier.width(8.dp))
                Text("Start Telegram Bot")
            }
        }
    }
}

// Reusable components

@Composable
private fun SectionHeader(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.titleSmall,
        fontWeight = FontWeight.Bold,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(vertical = 8.dp),
    )
}

@Composable
private fun SectionDivider() {
    Divider(
        modifier = Modifier.fillMaxWidth(),
        color = MaterialTheme.colorScheme.outlineVariant,
        thickness = 1.dp,
    )
}

@Composable
private fun NumericSetting(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    helperText: String,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        OutlinedTextField(
            value = value,
            onValueChange = { newValue ->
                if (newValue.all { it.isDigit() || it == '.' }) {
                    onValueChange(newValue)
                }
            },
            label = { Text(label) },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Text(
            text = helperText,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(start = 4.dp, top = 2.dp, bottom = 12.dp),
        )
    }
}

@Composable
private fun ToggleSetting(
    label: String,
    description: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 8.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = label,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium,
            )
            Text(
                text = description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Spacer(modifier = Modifier.width(12.dp))
        Switch(
            modifier = Modifier.semantics { contentDescription = label },
            checked = checked,
            onCheckedChange = onCheckedChange,
            colors = SwitchDefaults.colors(
                checkedThumbColor = MaterialTheme.colorScheme.primary,
                checkedTrackColor = MaterialTheme.colorScheme.primaryContainer,
                uncheckedThumbColor = MaterialTheme.colorScheme.outline,
                uncheckedTrackColor = MaterialTheme.colorScheme.surfaceVariant,
            ),
        )
    }
}
