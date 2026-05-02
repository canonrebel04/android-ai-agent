package com.yourdomain.agent

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp

// ── Default memory content ────────────────────────────────────────────────────

private val DEFAULT_MEMORY = """
# MEMORY.md

## Agent Identity
I am Hermes Agent, an intelligent AI assistant created by Nous Research.
I am helpful, knowledgeable, and direct.

## Preferences
- Communicate clearly and concisely.
- Admit uncertainty when appropriate.
- Prioritize being genuinely useful over being verbose.

## Project Context
- Working directory: /home/miyabi
- Platform: Android

## Notes
<!-- Add your persistent notes and context below -->
""".trimStart()

// ── Screen ────────────────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MemoryScreen() {
    var memoryContent by remember { mutableStateOf(DEFAULT_MEMORY) }
    var isEditing by remember { mutableStateOf(false) }
    var editBuffer by remember { mutableStateOf(DEFAULT_MEMORY) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("\uD83E\uDDE0 Memory")
                        Text(
                            text = "MEMORY.md",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                },
                actions = {
                    if (isEditing) {
                        // Save button
                        TextButton(
                            onClick = {
                                memoryContent = editBuffer
                                isEditing = false
                            },
                        ) {
                            Text(
                                text = "\u2705 Save",
                                color = MaterialTheme.colorScheme.onPrimaryContainer,
                            )
                        }
                        // Cancel button
                        TextButton(
                            onClick = {
                                editBuffer = memoryContent
                                isEditing = false
                            },
                        ) {
                            Text(
                                text = "Cancel",
                                color = MaterialTheme.colorScheme.onPrimaryContainer,
                            )
                        }
                    } else {
                        // Edit button
                        Button(
                            onClick = {
                                editBuffer = memoryContent
                                isEditing = true
                            },
                            colors = ButtonDefaults.buttonColors(
                                containerColor = MaterialTheme.colorScheme.primary,
                                contentColor = MaterialTheme.colorScheme.onPrimary,
                            ),
                        ) {
                            Text("\u270F\uFE0F Edit")
                        }
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer,
                    titleContentColor = MaterialTheme.colorScheme.onPrimaryContainer,
                ),
            )
        },
    ) { padding ->
        if (isEditing) {
            // ── Edit mode ──
            OutlinedTextField(
                value = editBuffer,
                onValueChange = { editBuffer = it },
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .padding(horizontal = 16.dp, vertical = 12.dp),
                textStyle = MaterialTheme.typography.bodyMedium.copy(
                    fontFamily = FontFamily.Monospace,
                ),
                label = { Text("MEMORY.md (edit mode)") },
            )
        } else {
            // ── View mode ──
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 16.dp, vertical = 12.dp),
            ) {
                Text(
                    text = memoryContent,
                    style = MaterialTheme.typography.bodyMedium.copy(
                        fontFamily = FontFamily.Monospace,
                    ),
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
}
