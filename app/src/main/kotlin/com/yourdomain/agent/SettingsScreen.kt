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
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.Dispatchers
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics

// ── Screen ────────────────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(viewModel: AgentViewModel) {
    var maxSteps by remember { mutableStateOf("50") }
    var actionDelay by remember { mutableStateOf("500") }
    var stallTimeout by remember { mutableStateOf("300") }
    var visionMode by remember { mutableStateOf(true) }
    var sendConfirmation by remember { mutableStateOf(true) }
    var deleteConfirmation by remember { mutableStateOf(true) }

    // Notification monitoring state
    var monitorWeChat by remember { mutableStateOf(true) }
    var monitorQQ by remember { mutableStateOf(true) }
    var monitorWeibo by remember { mutableStateOf(true) }
    var monitorDingTalk by remember { mutableStateOf(true) }
    var monitorWhatsApp by remember { mutableStateOf(true) }
    var monitorTelegram by remember { mutableStateOf(true) }
    var monitoredContacts by remember { mutableStateOf("") }

    // Telegram configuration state
    val context = LocalContext.current
    val keystore = remember { KeystoreManager(context) }
    var telegramEnabled by remember { mutableStateOf(false) }

    // Launch a coroutine to fetch the token off the main thread to avoid blocking the UI
    var telegramToken by remember { mutableStateOf("") }
    androidx.compose.runtime.LaunchedEffect(Unit) {
        withContext(Dispatchers.IO) {
            val token = keystore.getApiKey("telegramToken") ?: ""
            telegramToken = token
        }
    }

    // Message queue state
    var unprocessedCount by remember { mutableStateOf(GlobalMessageQueue.getUnprocessedCount()) }
    var totalCount by remember { mutableStateOf(GlobalMessageQueue.getTotalCount()) }
    var messages by remember { mutableStateOf(GlobalMessageQueue.getAllMessages()) }

    // Register listener for message queue changes
    DisposableEffect(Unit) {
        val listener = object : MessageQueueListener {
            override fun onMessageAdded(message: MessageItem) {
                unprocessedCount = GlobalMessageQueue.getUnprocessedCount()
                totalCount = GlobalMessageQueue.getTotalCount()
                messages = GlobalMessageQueue.getAllMessages()
            }

            override fun onMessageProcessed(message: MessageItem) {
                unprocessedCount = GlobalMessageQueue.getUnprocessedCount()
                totalCount = GlobalMessageQueue.getTotalCount()
                messages = GlobalMessageQueue.getAllMessages()
            }

            override fun onMessageCleared() {
                unprocessedCount = GlobalMessageQueue.getUnprocessedCount()
                totalCount = GlobalMessageQueue.getTotalCount()
                messages = GlobalMessageQueue.getAllMessages()
            }

            override fun onAllMessagesProcessed() {
                unprocessedCount = GlobalMessageQueue.getUnprocessedCount()
                totalCount = GlobalMessageQueue.getTotalCount()
                messages = GlobalMessageQueue.getAllMessages()
            }
        }
        GlobalMessageQueue.addListener(listener)
        onDispose {
            GlobalMessageQueue.removeListener(listener)
        }
    }

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

            // Notification Monitoring
            SectionHeader("Notification Monitoring")

            ToggleSetting(
                label = "Monitor WeChat",
                description = "Monitor notifications from WeChat",
                checked = monitorWeChat,
                onCheckedChange = { monitorWeChat = it },
            )

            ToggleSetting(
                label = "Monitor QQ",
                description = "Monitor notifications from QQ",
                checked = monitorQQ,
                onCheckedChange = { monitorQQ = it },
            )

            ToggleSetting(
                label = "Monitor Weibo",
                description = "Monitor notifications from Weibo",
                checked = monitorWeibo,
                onCheckedChange = { monitorWeibo = it },
            )

            ToggleSetting(
                label = "Monitor DingTalk",
                description = "Monitor notifications from DingTalk",
                checked = monitorDingTalk,
                onCheckedChange = { monitorDingTalk = it },
            )

            ToggleSetting(
                label = "Monitor WhatsApp",
                description = "Monitor notifications from WhatsApp",
                checked = monitorWhatsApp,
                onCheckedChange = { monitorWhatsApp = it },
            )

            ToggleSetting(
                label = "Monitor Telegram",
                description = "Monitor notifications from Telegram",
                checked = monitorTelegram,
                onCheckedChange = { monitorTelegram = it },
            )

            OutlinedTextField(
                value = monitoredContacts,
                onValueChange = { monitoredContacts = it },
                label = { Text("Monitored Contacts") },
                placeholder = { Text("Comma-separated list of contacts to monitor") },
                singleLine = false,
                modifier = Modifier.fillMaxWidth(),
            )
            Text(
                text = "Enter contact names to monitor (comma-separated)",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(start = 4.dp, top = 2.dp, bottom = 12.dp),
            )

            Button(
                onClick = {
                    val prefs = context.getSharedPreferences("AgentPrefs", Context.MODE_PRIVATE)
                    prefs.edit().apply {
                        putBoolean("monitorWeChat", monitorWeChat)
                        putBoolean("monitorQQ", monitorQQ)
                        putBoolean("monitorWeibo", monitorWeibo)
                        putBoolean("monitorDingTalk", monitorDingTalk)
                        putBoolean("monitorWhatsApp", monitorWhatsApp)
                        putBoolean("monitorTelegram", monitorTelegram)
                        val contactsSet = if (monitoredContacts.isBlank()) {
                            null
                        } else {
                            monitoredContacts.split(",").map { it.trim() }.filter { it.isNotBlank() }.toSet()
                        }
                        putStringSet("monitoredContacts", contactsSet)
                    }.apply()
                },
                modifier = Modifier.fillMaxWidth(),
            ) {
                Icon(Icons.Default.Send, contentDescription = "Save")
                Spacer(modifier = Modifier.width(8.dp))
                Text("Save Notification Settings")
            }

            Spacer(modifier = Modifier.height(8.dp))
            SectionDivider()
            Spacer(modifier = Modifier.height(8.dp))

            // Message Queue
            SectionHeader("Message Queue ($unprocessedCount/$totalCount)")

            if (messages.isEmpty()) {
                Text(
                    text = "No notifications in queue",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(vertical = 8.dp)
                )
            } else {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 4.dp),
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    for (message in messages.take(5)) { // Show first 5 messages
                        val statusText = if (message.isProcessed) "Processed" else "Pending"
                        val statusColor = if (message.isProcessed) {
                            MaterialTheme.colorScheme.onSurfaceVariant
                        } else {
                            MaterialTheme.colorScheme.primary
                        }
                        Text(
                            text = "${message.sender}: ${message.content.take(50)}${if (message.content.length > 50) "..." else ""} - $statusText",
                            style = MaterialTheme.typography.bodySmall,
                            color = statusColor,
                            modifier = Modifier.padding(vertical = 2.dp)
                        )
                    }
                    if (messages.size > 5) {
                        Text(
                            text = "... and ${messages.size - 5} more",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(vertical = 2.dp)
                        )
                    }
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Button(
                    onClick = { viewModel.processNextNotification() },
                    modifier = Modifier.weight(1f),
                    enabled = unprocessedCount > 0
                ) {
                    Text("Process Next")
                }
                Button(
                    onClick = { viewModel.processAllNotifications() },
                    modifier = Modifier.weight(1f),
                    enabled = unprocessedCount > 0
                ) {
                    Text("Process All")
                }
                Button(
                    onClick = { GlobalMessageQueue.clear() },
                    modifier = Modifier.weight(1f),
                    enabled = totalCount > 0
                ) {
                    Text("Clear All")
                }
            }
            SectionHeader("Floating Overlay")

            var floatingOverlayEnabled by remember { mutableStateOf(false) }

            ToggleSetting(
                label = "Show Floating Agent Overlay",
                description = "Display a floating pill showing agent status and current action",
                checked = floatingOverlayEnabled,
                onCheckedChange = { enabled ->
                    floatingOverlayEnabled = enabled
                    val intent = Intent(context, FloatingAgentOverlay::class.java)
                    if (enabled) {
                        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
                            context.startForegroundService(intent)
                        } else {
                            context.startService(intent)
                        }
                    } else {
                        context.stopService(intent)
                    }
                },
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

            val coroutineScope = androidx.compose.runtime.rememberCoroutineScope()
            Button(
                onClick = {
                    if (telegramEnabled && telegramToken.isNotBlank()) {
                        val intent = Intent(context, TelegramBotService::class.java).apply {
                            putExtra(TelegramBotService.EXTRA_BOT_TOKEN, telegramToken)
                        }
                        val prefs = context.getSharedPreferences("AgentPrefs", Context.MODE_PRIVATE)

                        coroutineScope.launch {
                            withContext(Dispatchers.IO) {
                                keystore.saveApiKey("telegramToken", telegramToken)
                            }
                            prefs.edit().putBoolean("telegramEnabled", true).apply()
                            context.startService(intent)
                        }
                    }
                },
                modifier = Modifier.fillMaxWidth(),
                enabled = telegramEnabled && telegramToken.isNotBlank()
            ) {
                Icon(Icons.Default.Send, contentDescription = "Start")
                Spacer(modifier = Modifier.width(8.dp))
                Text("Start Telegram Bot")
            }

            Spacer(modifier = Modifier.height(8.dp))
            SectionDivider()
            Spacer(modifier = Modifier.height(8.dp))

            // WebSocket Gateway
            SectionHeader("WebSocket Gateway")

            var wsEnabled by remember { mutableStateOf(false) }
            var wsUrl by remember { mutableStateOf("ws://192.168.1.100:8080/ws") }
            var wsConnected by remember { mutableStateOf(false) }

            ToggleSetting(
                label = "Enable WebSocket Gateway",
                description = "Allow remote control via WebSocket connection",
                checked = wsEnabled,
                onCheckedChange = { wsEnabled = it },
            )

            OutlinedTextField(
                value = wsUrl,
                onValueChange = { wsUrl = it },
                label = { Text("WebSocket URL") },
                placeholder = { Text("ws://host:port/ws") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            Text(
                text = "Enter WebSocket server URL (e.g., ws://192.168.1.100:8080/ws)",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(start = 4.dp, top = 2.dp, bottom = 12.dp),
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Button(
                    onClick = {
                        if (wsEnabled && wsUrl.isNotBlank()) {
                            val intent = Intent(context, WebSocketGateway::class.java).apply {
                                action = WebSocketGateway.ACTION_CONNECT
                                putExtra(WebSocketGateway.EXTRA_URL, wsUrl)
                            }
                            val prefs = context.getSharedPreferences("AgentPrefs", Context.MODE_PRIVATE)
                            prefs.edit().putString("wsUrl", wsUrl).apply()
                            prefs.edit().putBoolean("wsEnabled", true).apply()
                            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
                                context.startForegroundService(intent)
                            } else {
                                context.startService(intent)
                            }
                            wsConnected = true
                        }
                    },
                    modifier = Modifier.weight(1f),
                    enabled = wsEnabled && wsUrl.isNotBlank() && !wsConnected
                ) {
                    Text("Connect")
                }
                Button(
                    onClick = {
                        val intent = Intent(context, WebSocketGateway::class.java).apply {
                            action = WebSocketGateway.ACTION_DISCONNECT
                        }
                        context.startService(intent)
                        wsConnected = false
                    },
                    modifier = Modifier.weight(1f),
                    enabled = wsConnected
                ) {
                    Text("Disconnect")
                }
            }
            if (wsConnected) {
                Text(
                    text = "Connected to $wsUrl",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 8.dp)
                )
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
