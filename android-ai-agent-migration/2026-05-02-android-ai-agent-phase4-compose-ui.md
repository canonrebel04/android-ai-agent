# Android AI Agent — Phase 4: Compose UI Screens

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Build the Jetpack Compose UI screens — HomeScreen (task input + live log), ModelsScreen (tier configuration), SkillsScreen, SettingsScreen, MemoryScreen, and ChannelsScreen. All screens wire to AgentViewModel via StateFlow.

**Architecture:** MVVM — each screen observes `AgentViewModel.state`. Navigation via Jetpack Compose Navigation. HomeScreen is the primary entry point with task input, live scrolling log, model picker dropdown, and run/stop controls.

**Tech Stack:** Jetpack Compose BOM 2026.04.01, Material3, lifecycle-viewmodel-compose 2.10.0, coroutines 1.10.2.

**Files:** All in `app/src/main/kotlin/com/yourdomain/agent/` with `package com.yourdomain.agent`.

---

### Task 1: HomeScreen — task input + live log + controls

**Objective:** The main screen: text field for task input, scrolling live log, model picker, run/stop buttons, voice input button. Binds to AgentViewModel.state.

**Files:**
- Create: `app/src/main/kotlin/com/yourdomain/agent/HomeScreen.kt`

**Code:**

```kotlin
package com.yourdomain.agent

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(viewModel: AgentViewModel = viewModel()) {
    val state by viewModel.state.collectAsState()
    var taskInput by remember { mutableStateOf("") }
    val listState = rememberLazyListState()

    LaunchedEffect(state.logLines.size) {
        if (state.logLines.isNotEmpty()) {
            listState.animateScrollToItem(state.logLines.size - 1)
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("◉ AGENT  [${state.status}]") },
                actions = {
                    Text(
                        text = state.activeModel.ifEmpty { "claude-sonnet" },
                        style = MaterialTheme.typography.labelMedium,
                        modifier = Modifier.padding(end = 16.dp)
                    )
                }
            )
        },
        bottomBar = {
            Surface(
                tonalElevation = 3.dp,
                modifier = Modifier.fillMaxWidth()
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    OutlinedTextField(
                        value = taskInput,
                        onValueChange = { taskInput = it },
                        placeholder = { Text("Enter task or use voice...") },
                        modifier = Modifier.weight(1f),
                        singleLine = true,
                        enabled = state.status != "running"
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    if (state.status == "running") {
                        FilledTonalButton(onClick = { viewModel.stopTask() }) {
                            Text("■")
                        }
                    } else {
                        Button(
                            onClick = {
                                if (taskInput.isNotBlank()) {
                                    viewModel.startTask(taskInput)
                                    taskInput = ""
                                }
                            }
                        ) {
                            Text("▶ Run")
                        }
                    }
                }
            }
        }
    ) { padding ->
        Column(modifier = Modifier.padding(padding)) {
            // Current task display
            if (state.currentTask.isNotBlank()) {
                Surface(
                    color = MaterialTheme.colorScheme.secondaryContainer,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text(
                        text = state.currentTask,
                        modifier = Modifier.padding(12.dp),
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
            }

            // Confirmation prompt
            state.pendingConfirmation?.let { pending ->
                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.errorContainer
                    ),
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(8.dp)
                ) {
                    Column(modifier = Modifier.padding(12.dp)) {
                        Text("Confirm: $pending", style = MaterialTheme.typography.bodyMedium)
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            Button(onClick = { viewModel.confirmAction(true) }) {
                                Text("Approve")
                            }
                            OutlinedButton(onClick = { viewModel.confirmAction(false) }) {
                                Text("Deny")
                            }
                        }
                    }
                }
            }

            // Live log
            LazyColumn(
                state = listState,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 8.dp)
            ) {
                items(state.logLines) { line ->
                    Text(
                        text = line,
                        fontFamily = FontFamily.Monospace,
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(vertical = 2.dp)
                    )
                }
            }
        }
    }
}
```

**Step 2: Commit**
```bash
git add -A && git commit -m "feat: add HomeScreen with task input, live log, and run/stop controls"
```

---

### Task 2: ModelsScreen — tier configuration

**Objective:** Display and configure the 4 model tiers (Trivial, Standard, Complex, Critical) with model name, fallback chain editor.

**Files:**
- Create: `app/src/main/kotlin/com/yourdomain/agent/ModelsScreen.kt`

**Code:**

```kotlin
package com.yourdomain.agent

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

data class ModelTierUi(
    val name: String,
    val primaryModel: String,
    val fallbacks: List<String>,
    val cost: String,
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ModelsScreen() {
    val tiers = remember {
        listOf(
            ModelTierUi("Trivial", "gemini-flash-2.5", listOf("mistral-small-3.2"), "~$0.075/M"),
            ModelTierUi("Standard", "mistral-small-3.2", listOf("gemini-flash-2.5"), "~$0.10/M"),
            ModelTierUi("Complex", "claude-sonnet-4-6", listOf("mistral-small-3.2", "gemini-flash-2.5"), "~$3.00/M"),
            ModelTierUi("Critical", "claude-opus-4-6", listOf("claude-sonnet-4-6"), "~$15.00/M"),
        )
    }

    Scaffold(
        topBar = { TopAppBar(title = { Text("Models & Tiers") }) }
    ) { padding ->
        LazyColumn(
            modifier = Modifier.padding(padding),
            verticalArrangement = Arrangement.spacedBy(8.dp),
            contentPadding = PaddingValues(16.dp)
        ) {
            items(tiers) { tier ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween
                        ) {
                            Text(tier.name, style = MaterialTheme.typography.titleMedium)
                            Text(tier.cost, style = MaterialTheme.typography.labelMedium)
                        }
                        Spacer(modifier = Modifier.height(4.dp))
                        Text("Primary: ${tier.primaryModel}", style = MaterialTheme.typography.bodyMedium)
                        if (tier.fallbacks.isNotEmpty()) {
                            Text(
                                "Fallbacks: ${tier.fallbacks.joinToString(", ")}",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Spacer(modifier = Modifier.height(8.dp))
                        OutlinedButton(onClick = { /* Edit tier */ }) {
                            Text("Configure")
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**
```bash
git add -A && git commit -m "feat: add ModelsScreen with tier configuration"
```

---

### Task 3: SkillsScreen — skill browser

**Objective:** List installed skills with enable/disable toggles, install from URL, usage stats.

**Files:**
- Create: `app/src/main/kotlin/com/yourdomain/agent/SkillsScreen.kt`

**Code:**

```kotlin
package com.yourdomain.agent

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

data class SkillUi(
    val name: String,
    val description: String,
    val enabled: Boolean,
    val complexity: String,
    val usageCount: Int,
    val requiresConfirmation: Boolean,
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SkillsScreen() {
    val skills = remember {
        listOf(
            SkillUi("screen_control", "Tap, swipe, type on any app", true, "Standard", 28, false),
            SkillUi("open_app", "Launch app by name or package", true, "Trivial", 12, false),
            SkillUi("web_search", "Search via local self-hosted engine", true, "Standard", 31, false),
            SkillUi("send_message", "Send SMS, Telegram, WhatsApp", true, "Critical", 47, true),
            SkillUi("phone_call", "Make calls", true, "Critical", 3, true),
            SkillUi("calendar", "Read/create calendar events", true, "Standard", 8, false),
            SkillUi("shell_cmd", "Run shell command (Termux IPC)", false, "Critical", 0, true),
            SkillUi("camera", "Take a photo", true, "Standard", 2, false),
        )
    }

    Scaffold(
        topBar = { TopAppBar(title = { Text("Skills") }) }
    ) { padding ->
        LazyColumn(
            modifier = Modifier.padding(padding),
            verticalArrangement = Arrangement.spacedBy(4.dp),
            contentPadding = PaddingValues(16.dp)
        ) {
            items(skills) { skill ->
                Card(modifier = Modifier.fillMaxWidth()) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(12.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                Text(skill.name, style = MaterialTheme.typography.titleSmall)
                                Text(
                                    skill.complexity,
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.primary
                                )
                            }
                            Text(skill.description, style = MaterialTheme.typography.bodySmall)
                            Text(
                                "Used ${skill.usageCount} times",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Switch(
                            checked = skill.enabled,
                            onCheckedChange = { /* Toggle skill */ }
                        )
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**
```bash
git add -A && git commit -m "feat: add SkillsScreen with enable/disable toggles and usage stats"
```

---

### Task 4: SettingsScreen + MemoryScreen + ChannelsScreen (combined)

**Objective:** Create the remaining 3 screens — Settings (agent behavior, budget alerts, permissions), Memory (view/edit MEMORY.md), Channels (Telegram, WhatsApp, Voice, Gateway setup).

**Files:**
- Create: `app/src/main/kotlin/com/yourdomain/agent/SettingsScreen.kt`
- Create: `app/src/main/kotlin/com/yourdomain/agent/MemoryScreen.kt`
- Create: `app/src/main/kotlin/com/yourdomain/agent/ChannelsScreen.kt`

**SettingsScreen.kt:**

```kotlin
package com.yourdomain.agent

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen() {
    var maxSteps by remember { mutableStateOf("50") }
    var actionDelay by remember { mutableStateOf("300") }
    var stallTimeout by remember { mutableStateOf("5") }
    var visionMode by remember { mutableStateOf(false) }
    var budgetAlert by remember { mutableStateOf("10.00") }
    var confirmSend by remember { mutableStateOf(true) }
    var confirmDelete by remember { mutableStateOf(true) }

    Scaffold(
        topBar = { TopAppBar(title = { Text("Settings") }) }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Agent Behavior
            Text("Agent Behavior", style = MaterialTheme.typography.titleMedium)
            OutlinedTextField(value = maxSteps, onValueChange = { maxSteps = it }, label = { Text("Max steps per task") })
            OutlinedTextField(value = actionDelay, onValueChange = { actionDelay = it }, label = { Text("Action delay (ms)") })
            OutlinedTextField(value = stallTimeout, onValueChange = { stallTimeout = it }, label = { Text("Stall timeout (s)") })
            Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                Text("Vision Mode")
                Switch(checked = visionMode, onCheckedChange = { visionMode = it })
            }

            Divider()

            // Confirmations
            Text("Require Confirmation", style = MaterialTheme.typography.titleMedium)
            Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                Text("Send messages")
                Switch(checked = confirmSend, onCheckedChange = { confirmSend = it })
            }
            Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                Text("Delete / Payment / Call")
                Switch(checked = confirmDelete, onCheckedChange = { confirmDelete = it })
            }

            Divider()

            // Budget
            Text("Usage & Cost", style = MaterialTheme.typography.titleMedium)
            OutlinedTextField(value = budgetAlert, onValueChange = { budgetAlert = it }, label = { Text("Budget alert threshold ($)") })
            Text("Token usage: today / week / month", style = MaterialTheme.typography.bodySmall)
        }
    }
}
```

**MemoryScreen.kt:**

```kotlin
package com.yourdomain.agent

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MemoryScreen() {
    var memoryContent by remember { mutableStateOf(
        """## User Profile
- Name: User
- Preferred model: claude-sonnet-4-6

## Persistent Facts
- Work email: user@example.com
- Home WiFi: MyNetwork

## Recent Context
- Last task: "Sent email to Alex" (2026-04-21)

## Learned Preferences
- Prefers Telegram for status updates
- Dislikes verbose responses""".trimIndent()
    ) }
    var isEditing by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Memory") },
                actions = {
                    TextButton(onClick = { isEditing = !isEditing }) {
                        Text(if (isEditing) "Save" else "Edit")
                    }
                }
            )
        }
    ) { padding ->
        if (isEditing) {
            OutlinedTextField(
                value = memoryContent,
                onValueChange = { memoryContent = it },
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .padding(16.dp),
            )
        } else {
            Text(
                text = memoryContent,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .padding(16.dp),
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}
```

**ChannelsScreen.kt:**

```kotlin
package com.yourdomain.agent

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChannelsScreen() {
    var telegramToken by remember { mutableStateOf("") }
    var telegramEnabled by remember { mutableStateOf(false) }
    var whatsappEnabled by remember { mutableStateOf(false) }
    var voiceWakeWord by remember { mutableStateOf("hey agent") }
    var voiceEnabled by remember { mutableStateOf(false) }
    var gatewayEnabled by remember { mutableStateOf(false) }

    Scaffold(
        topBar = { TopAppBar(title = { Text("Channels") }) }
    ) { padding ->
        Column(
            modifier = Modifier
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Telegram
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                        Text("Telegram Bot", style = MaterialTheme.typography.titleMedium)
                        Switch(checked = telegramEnabled, onCheckedChange = { telegramEnabled = it })
                    }
                    if (telegramEnabled) {
                        OutlinedTextField(value = telegramToken, onValueChange = { telegramToken = it }, label = { Text("Bot Token") })
                        OutlinedButton(onClick = { /* Test connection */ }) { Text("Test Connection") }
                    }
                }
            }

            // WhatsApp
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                        Text("WhatsApp (via Accessibility)", style = MaterialTheme.typography.titleMedium)
                        Switch(checked = whatsappEnabled, onCheckedChange = { whatsappEnabled = it })
                    }
                    if (whatsappEnabled) {
                        Text("Opt-in. Monitors WhatsApp notifications from designated contacts only.", style = MaterialTheme.typography.bodySmall)
                    }
                }
            }

            // Voice
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                        Text("Voice Mode", style = MaterialTheme.typography.titleMedium)
                        Switch(checked = voiceEnabled, onCheckedChange = { voiceEnabled = it })
                    }
                    if (voiceEnabled) {
                        OutlinedTextField(value = voiceWakeWord, onValueChange = { voiceWakeWord = it }, label = { Text("Wake Word") })
                    }
                }
            }

            // Gateway
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(horizontalArrangement = Arrangement.SpaceBetween, modifier = Modifier.fillMaxWidth()) {
                        Text("WebSocket Gateway", style = MaterialTheme.typography.titleMedium)
                        Switch(checked = gatewayEnabled, onCheckedChange = { gatewayEnabled = it })
                    }
                    if (gatewayEnabled) {
                        Text("Port: 8765", style = MaterialTheme.typography.bodySmall)
                        Text("Auth token: auto-generated", style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
        }
    }
}
```

**Step 2: Commit**
```bash
git add -A && git commit -m "feat: add Settings, Memory, and Channels screens"
```

---

### Task 5: Navigation — wire all screens together

**Objective:** Add Jetpack Compose Navigation with bottom nav bar to connect all 6 screens.

**Files:**
- Create: `app/src/main/kotlin/com/yourdomain/agent/MainActivity.kt`
- Create: `app/src/main/kotlin/com/yourdomain/agent/Navigation.kt`

**Navigation.kt:**

```kotlin
package com.yourdomain.agent

import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector

enum class Screen(val label: String, val icon: ImageVector) {
    Home("Home", Icons.Default.Home),
    Models("Models", Icons.Default.Star),
    Skills("Skills", Icons.Default.Build),
    Channels("Channels", Icons.Default.Chat),
    Memory("Memory", Icons.Default.Info),
    Settings("Settings", Icons.Default.Settings),
}

@Composable
fun AgentNavigation() {
    var currentScreen by remember { mutableStateOf(Screen.Home) }

    Scaffold(
        bottomBar = {
            NavigationBar {
                listOf(Screen.Home, Screen.Models, Screen.Skills, Screen.Channels).forEach { screen ->
                    NavigationBarItem(
                        icon = { Icon(screen.icon, contentDescription = screen.label) },
                        label = { Text(screen.label) },
                        selected = currentScreen == screen,
                        onClick = { currentScreen = screen }
                    )
                }
            }
        }
    ) { padding ->
        when (currentScreen) {
            Screen.Home -> HomeScreen()
            Screen.Models -> ModelsScreen()
            Screen.Skills -> SkillsScreen()
            Screen.Channels -> ChannelsScreen()
            Screen.Memory -> MemoryScreen()
            Screen.Settings -> SettingsScreen()
        }
    }
}
```

**MainActivity.kt:**

```kotlin
package com.yourdomain.agent

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            AgentNavigation()
        }
    }
}
```

**Step 2: Commit**
```bash
git add -A && git commit -m "feat: add navigation with bottom nav bar and MainActivity"
```

---

## Completion Checklist

- [ ] 6 screens: Home, Models, Skills, Settings, Memory, Channels
- [ ] Bottom nav bar with 4 primary tabs
- [ ] HomeScreen wired to AgentViewModel.state
- [ ] All screens compile in Android Studio

## After Phase 4

The Android app has a complete UI. Remaining:
1. Wire AgentViewModel to the Rust JNI bridge (cross-compile .so)
2. Build the AccessibilityService integration
3. Telegram bot service
4. Gateway WebSocket server
