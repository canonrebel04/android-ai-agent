package com.yourdomain.agent

import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material.icons.filled.History
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.launch
import java.util.regex.Pattern

/**
 * ANSI color codes for terminal output
 */
object AnsiColors {
    private val ANSI_COLOR_MAP = mapOf(
        "30" to Color.Black,
        "31" to Color.Red,
        "32" to Color.Green,
        "33" to Color.Yellow,
        "34" to Color.Blue,
        "35" to Color.Magenta,
        "36" to Color.Cyan,
        "37" to Color.White,
        "90" to Color.Gray,
        "91" to Color(0xFFFF6B6B),  // Light Red
        "92" to Color(0xFF51CF66),  // Light Green
        "93" to Color(0xFFFFD43B),  // Light Yellow
        "94" to Color(0xFF64B5F6),  // Light Blue
        "95" to Color(0xFFBA68C8),  // Light Magenta
        "96" to Color(0xFF4FC3F7),  // Light Cyan
        "97" to Color(0xFFF5F5F5),  // Light White
    )

    private val ANSI_BACKGROUND_MAP = mapOf(
        "40" to Color.Black,
        "41" to Color.Red,
        "42" to Color.Green,
        "43" to Color.Yellow,
        "44" to Color.Blue,
        "45" to Color.Magenta,
        "46" to Color.Cyan,
        "47" to Color.White,
    )

    /**
     * Parse ANSI escape codes and return styled text
     */
    fun parseAnsi(text: String, baseColor: Color = MaterialTheme.colorScheme.onSurface): AnnotatedString {
        val pattern = Pattern.compile("\\u001B\[([0-9;]*)m")
        val matcher = pattern.matcher(text)
        
        val builder = AnnotatedString.Builder()
        
        var currentColor: Color? = null
        var currentBgColor: Color? = null
        var currentWeight: FontWeight? = null
        var currentStyle: FontStyle? = null
        var currentDecoration: TextDecoration? = null
        var lastIndex = 0
        
        while (matcher.find()) {
            // Add text before the ANSI code
            if (matcher.start() > lastIndex) {
                val plainText = text.substring(lastIndex, matcher.start())
                val style = SpanStyle(
                    color = currentColor ?: baseColor,
                    background = currentBgColor,
                    fontWeight = currentWeight,
                    fontStyle = currentStyle,
                    textDecoration = currentDecoration
                )
                builder.pushStyle(style)
                builder.append(plainText)
                builder.pop()
            }
            
            // Parse ANSI codes
            val codes = matcher.group(1).split(";").filter { it.isNotEmpty() }
            
            for (code in codes) {
                when (code) {
                    "0" -> {
                        // Reset all formatting
                        currentColor = null
                        currentBgColor = null
                        currentWeight = null
                        currentStyle = null
                        currentDecoration = null
                    }
                    "1" -> currentWeight = FontWeight.Bold
                    "2" -> currentWeight = FontWeight.Light
                    "3" -> currentStyle = FontStyle.Italic
                    "4" -> currentDecoration = TextDecoration.Underline
                    in ANSI_COLOR_MAP -> currentColor = ANSI_COLOR_MAP[code]
                    in ANSI_BACKGROUND_MAP -> currentBgColor = ANSI_BACKGROUND_MAP[code]
                }
            }
            
            lastIndex = matcher.end()
        }
        
        // Add remaining text
        if (lastIndex < text.length) {
            val plainText = text.substring(lastIndex)
            val style = SpanStyle(
                color = currentColor ?: baseColor,
                background = currentBgColor,
                fontWeight = currentWeight,
                fontStyle = currentStyle,
                textDecoration = currentDecoration
            )
            builder.pushStyle(style)
            builder.append(plainText)
            builder.pop()
        }
        
        return builder.toAnnotatedString()
    }
}

/**
 * ViewModel for TerminalScreen
 */
class TerminalViewModel : androidx.lifecycle.ViewModel() {
    private val _outputLines = mutableStateListOf<OutputLine>()
    val outputLines: List<OutputLine> get() = _outputLines
    
    private val _commandHistory = mutableStateListOf<String>()
    val commandHistory: List<String> get() = _commandHistory
    
    private val _currentCommand = mutableStateOf("")
    val currentCommand: String get() = _currentCommand.value
    
    private val _isExecuting = mutableStateOf(false)
    val isExecuting: Boolean get() = _isExecuting.value
    
    private val _sandboxInitialized = mutableStateOf(false)
    val sandboxInitialized: Boolean get() = _sandboxInitialized.value
    
    private val _sandbox = mutableStateOf<LinuxSandbox?>(null)
    
    private var historyIndex = -1
    private val maxHistorySize = 100
    
    init {
        // Initialize sandbox in background
        initializeSandbox()
    }
    
    private fun initializeSandbox() {
        try {
            val context = android.app.Application().applicationContext
            val sandbox = LinuxSandbox(context)
            _sandbox.value = sandbox
            _sandboxInitialized.value = sandbox.start()
        } catch (e: Exception) {
            Log.e("TerminalViewModel", "Error initializing sandbox", e)
            _sandboxInitialized.value = false
        }
    }
    
    fun setCommand(command: String) {
        _currentCommand.value = command
        historyIndex = -1
    }
    
    fun navigateHistory(up: Boolean) {
        if (_commandHistory.isEmpty()) return
        
        if (up) {
            if (historyIndex < _commandHistory.size - 1) {
                historyIndex++
                _currentCommand.value = if (historyIndex == 0) {
                    _commandHistory[0]
                } else {
                    _commandHistory[historyIndex]
                }
            }
        } else {
            if (historyIndex > 0) {
                historyIndex--
                _currentCommand.value = _commandHistory[historyIndex]
            } else if (historyIndex == 0) {
                historyIndex = -1
                _currentCommand.value = ""
            }
        }
    }
    
    fun addToOutput(text: String, isCommand: Boolean = false, isError: Boolean = false) {
        val line = OutputLine(
            text = text,
            isCommand = isCommand,
            isError = isError,
            timestamp = System.currentTimeMillis()
        )
        _outputLines.add(line)
    }
    
    fun clearOutput() {
        _outputLines.clear()
    }
    
    fun executeCommand(command: String) {
        if (command.isBlank() || _isExecuting.value || _sandbox.value == null) return
        
        val trimmedCommand = command.trim()
        if (trimmedCommand.isEmpty()) return
        
        // Add command to history
        if (_commandHistory.isEmpty() || _commandHistory.last() != trimmedCommand) {
            _commandHistory.add(trimmedCommand)
            if (_commandHistory.size > maxHistorySize) {
                _commandHistory.removeAt(0)
            }
        }
        
        // Display command in output
        addToOutput("\$ ${trimmedCommand}", isCommand = true)
        setCommand("")
        
        _isExecuting.value = true
        
        // Execute in background
        val sandbox = _sandbox.value!!
        
        android.os.Handler(android.os.Looper.getMainLooper()).post {
            object : Thread() {
                override fun run() {
                    try {
                        val result = sandbox.executeCommand(trimmedCommand, 30000)
                        if (result != null) {
                            val (exitCode, output) = result
                            if (exitCode == 0) {
                                // Split output by newlines and add each line
                                output.split("\n").forEach { line ->
                                    if (line.isNotBlank()) {
                                        addToOutput(line)
                                    }
                                }
                            } else {
                                addToOutput("Error: Command exited with code $exitCode", isError = true)
                                if (output.isNotBlank()) {
                                    output.split("\n").forEach { line ->
                                        if (line.isNotBlank()) {
                                            addToOutput(line, isError = true)
                                        }
                                    }
                                }
                            }
                        } else {
                            addToOutput("Error: Failed to execute command", isError = true)
                        }
                    } catch (e: Exception) {
                        addToOutput("Error: ${e.message}", isError = true)
                        Log.e("TerminalViewModel", "Error executing command", e)
                    } finally {
                        _isExecuting.value = false
                    }
                }
            }.start()
        }
    }
    
    fun getSuggestions(command: String): List<String> {
        if (command.isBlank()) return emptyList()
        
        val commonCommands = listOf(
            "ls", "cd", "pwd", "cat", "echo", "grep", "find", "mkdir", "rm", "cp", "mv",
            "chmod", "chown", "ps", "kill", "top", "htop", "df", "du", "free",
            "apt", "apt-get", "yum", "dnf", "apt-cache", "dpkg", "rpm",
            "python", "python3", "pip", "pip3", "node", "npm", "npx", "yarn",
            "git", "git clone", "git pull", "git push", "git status", "git log",
            "curl", "wget", "ssh", "scp", "rsync", "ping", "ifconfig", "ip",
            "netstat", "ss", "dig", "nslookup", "traceroute", "mtr",
            "man", "info", "which", "whereis", "type", "alias", "unalias",
            "source", "export", "unset", "env", "printenv", "set"
        )
        
        val prefix = command.lowercase()
        return commonCommands.filter { it.startsWith(prefix) }.take(5)
    }
    
    override fun onCleared() {
        super.onCleared()
        _sandbox.value?.cleanup()
    }
}

/**
 * Data class for output lines
 */
data class OutputLine(
    val id: String = java.util.UUID.randomUUID().toString(),
    val text: String,
    val isCommand: Boolean = false,
    val isError: Boolean = false,
    val timestamp: Long = System.currentTimeMillis()
)

/**
 * Terminal Screen Composable
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TerminalScreen(
    viewModel: TerminalViewModel = viewModel()
) {
    val outputLines = viewModel.outputLines
    val commandHistory = viewModel.commandHistory
    val isExecuting = viewModel.isExecuting
    val sandboxInitialized = viewModel.sandboxInitialized
    
    var currentCommand by rememberSaveable { mutableStateOf("") }
    var showSuggestions by remember { mutableStateOf(false) }
    var suggestions by remember { mutableStateOf<List<String>>(emptyList()) }
    var showHistoryDropdown by remember { mutableStateOf(false) }
    
    val listState = rememberLazyListState()
    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    
    // Auto-scroll to bottom when new output arrives
    LaunchedEffect(outputLines.size) {
        if (outputLines.isNotEmpty()) {
            listState.animateScrollToItem(outputLines.size - 1)
        }
    }
    
    // Update current command from viewModel
    LaunchedEffect(viewModel.currentCommand) {
        currentCommand = viewModel.currentCommand
    }
    
    // Get suggestions as user types
    LaunchedEffect(currentCommand) {
        suggestions = viewModel.getSuggestions(currentCommand)
        showSuggestions = suggestions.isNotEmpty() && currentCommand.isNotBlank()
    }
    
    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Text("Terminal")
                        if (isExecuting) {
                            CircularProgressIndicator(
                                modifier = Modifier
                                    .height(16.dp)
                                    .width(16.dp),
                                strokeWidth = 2.dp
                            )
                        }
                    }
                },
                actions = {
                    IconButton(
                        onClick = { viewModel.clearOutput() },
                        enabled = outputLines.isNotEmpty()
                    ) {
                        Icon(Icons.Default.Clear, contentDescription = "Clear")
                    }
                    IconButton(
                        onClick = { showHistoryDropdown = true }
                    ) {
                        Icon(Icons.Default.History, contentDescription = "History")
                    }
                    
                    // History dropdown
                    if (showHistoryDropdown) {
                        DropdownMenu(
                            expanded = showHistoryDropdown,
                            onDismissRequest = { showHistoryDropdown = false },
                            modifier = Modifier.width(200.dp)
                        ) {
                            if (commandHistory.isEmpty()) {
                                DropdownMenuItem(
                                    text = { Text("No history") },
                                    onClick = { showHistoryDropdown = false }
                                )
                            } else {
                                commandHistory.takeLast(10).reversed().forEach { cmd ->
                                    DropdownMenuItem(
                                        text = { Text(cmd, maxLines = 1) },
                                        onClick = {
                                            currentCommand = cmd
                                            showHistoryDropdown = false
                                        }
                                    )
                                }
                            }
                        }
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                    actionIconContentColor = MaterialTheme.colorScheme.onPrimaryContainer
                )
            )
        },
        snackbarHost = { SnackbarHost(snackbarHostState) },
        bottomBar = {
            Surface(
                tonalElevation = 3.dp,
                shadowElevation = 8.dp
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 8.dp, vertical = 4.dp)
                ) {
                    // Suggestions dropdown
                    if (showSuggestions && suggestions.isNotEmpty()) {
                        Surface(
                            shape = MaterialTheme.shapes.small,
                            shadowElevation = 4.dp,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            Column(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .padding(vertical = 4.dp)
                            ) {
                                suggestions.forEach { suggestion ->
                                    Text(
                                        text = suggestion,
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .clickable {
                                                currentCommand = suggestion + " "
                                                showSuggestions = false
                                            }
                                            .padding(horizontal = 16.dp, vertical = 8.dp),
                                        style = MaterialTheme.typography.bodySmall
                                    )
                                }
                            }
                        }
                    }
                    
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        IconButton(
                            onClick = { viewModel.navigateHistory(true) },
                            enabled = commandHistory.isNotEmpty()
                        ) {
                            Icon(Icons.Default.KeyboardArrowUp, contentDescription = "Previous command")
                        }
                        
                        OutlinedTextField(
                            value = currentCommand,
                            onValueChange = { 
                                currentCommand = it
                                viewModel.setCommand(it)
                            },
                            modifier = Modifier.weight(1f),
                            placeholder = { 
                                Text(
                                    "Enter command...",
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            },
                            singleLine = false,
                            maxLines = 3,
                            enabled = !isExecuting && sandboxInitialized,
                            keyboardOptions = KeyboardOptions(
                                capitalization = KeyboardCapitalization.None,
                                autoCorrect = false,
                                imeAction = ImeAction.Send
                            ),
                            keyboardActions = KeyboardActions(
                                onSend = {
                                    if (currentCommand.isNotBlank()) {
                                        viewModel.executeCommand(currentCommand)
                                    }
                                }
                            ),
                            leadingIcon = {
                                if (!sandboxInitialized) {
                                    CircularProgressIndicator(
                                        modifier = Modifier.size(20.dp),
                                        strokeWidth = 2.dp
                                    )
                                }
                            },
                            trailingIcon = {
                                if (currentCommand.isNotBlank()) {
                                    IconButton(
                                        onClick = {
                                            currentCommand = ""
                                            viewModel.setCommand("")
                                        }
                                    ) {
                                        Icon(Icons.Default.Clear, contentDescription = "Clear")
                                    }
                                }
                            }
                        )
                        
                        IconButton(
                            onClick = { viewModel.navigateHistory(false) },
                            enabled = commandHistory.isNotEmpty()
                        ) {
                            Icon(Icons.Default.KeyboardArrowDown, contentDescription = "Next command")
                        }
                        
                        Button(
                            onClick = {
                                if (currentCommand.isNotBlank()) {
                                    viewModel.executeCommand(currentCommand)
                                }
                            },
                            enabled = currentCommand.isNotBlank() && !isExecuting && sandboxInitialized,
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                                contentColor = MaterialTheme.colorScheme.onPrimary
                            )
                        ) {
                            Icon(Icons.Default.Send, contentDescription = "Execute")
                        }
                    }
                }
            }
        }
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            if (!sandboxInitialized) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(16.dp)
                    ) {
                        CircularProgressIndicator()
                        Text("Initializing sandbox environment...")
                        Text(
                            "This may take a moment on first launch",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            } else if (outputLines.isEmpty()) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Icon(
                            Icons.Default.Send,
                            contentDescription = null,
                            modifier = Modifier.size(48.dp),
                            tint = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Text(
                            "Terminal Ready",
                            style = MaterialTheme.typography.headlineSmall
                        )
                        Text(
                            "Enter commands below to execute in the sandbox",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Spacer(modifier = Modifier.height(16.dp))
                        Text(
                            "Try: ls, pwd, echo, apt list",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
            
            LazyColumn(
                state = listState,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 8.dp, vertical = 4.dp),
                verticalArrangement = Arrangement.spacedBy(2.dp)
            ) {
                // Bolt ⚡ Optimization: Add stable key for Compose list rendering
                // This prevents unnecessary recompositions of all previous terminal
                // output lines when a new line is appended to the terminal.
                items(
                    items = outputLines,
                    key = { it.id }
                ) { line ->
                    TerminalOutputLine(line = line)
                }
            }
        }
    }
}

/**
 * Composable for a single terminal output line with ANSI color support
 */
@Composable
fun TerminalOutputLine(line: OutputLine) {
    val baseColor = when {
        line.isCommand -> MaterialTheme.colorScheme.onPrimaryContainer
        line.isError -> MaterialTheme.colorScheme.onErrorContainer
        else -> MaterialTheme.colorScheme.onSurface
    }
    
    val containerColor = when {
        line.isCommand -> MaterialTheme.colorScheme.primaryContainer
        line.isError -> MaterialTheme.colorScheme.errorContainer
        else -> Color.Transparent
    }
    
    Surface(
        shape = MaterialTheme.shapes.extraSmall,
        color = containerColor,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 8.dp, vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            if (line.isCommand) {
                Text(
                    text = "\$ ",
                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                    fontFamily = FontFamily.Monospace,
                    fontSize = 14.sp
                )
            }
            
            Spacer(modifier = Modifier.width(4.dp))
            
            // Parse and render ANSI colors
            val annotatedText = remember(line.text, baseColor) {
                AnsiColors.parseAnsi(line.text, baseColor)
            }
            
            Text(
                text = annotatedText,
                fontFamily = FontFamily.Monospace,
                fontSize = 14.sp,
                lineHeight = 20.sp
            )
        }
    }
}

/**
 * Preview for TerminalScreen
 */
@Composable
fun TerminalScreenPreview() {
    MaterialTheme {
        TerminalScreen()
    }
}
