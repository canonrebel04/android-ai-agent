package com.yourdomain.agent

import android.content.Intent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics


@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VoiceScreen() {
    val context = LocalContext.current
    val specs = remember { DevicePerformanceMetrics.measure(context) }
    val modelManager = remember { VoiceModelManager(context) }
    val models by modelManager.models.collectAsState()

    val whisperModels = remember(models) { models.filter { it.id.startsWith("whisper") } }
    val piperModels = remember(models) { models.filter { it.id.startsWith("piper") } }

    var wakeWord by remember { mutableStateOf("hey agent") }
    var voiceEnabled by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) { modelManager.refreshStatus() }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("🎙 Voice Mode") },
                actions = {
                    Switch(
                        modifier = Modifier.semantics { contentDescription = "Voice Mode Toggle" },
                        checked = voiceEnabled,
                        onCheckedChange = {
                            voiceEnabled = it
                            if (it) {
                                context.startService(Intent(context, VoiceService::class.java).apply {
                                    action = VoiceService.ACTION_START_LISTENING
                                })
                            } else {
                                context.startService(Intent(context, VoiceService::class.java).apply {
                                    action = VoiceService.ACTION_STOP_LISTENING
                                })
                            }
                        }
                    )
                }
            )
        }
    ) { padding ->
        LazyColumn(
            modifier = Modifier
                .padding(padding)
                .fillMaxSize(),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            // ── Device Performance ──
            item {
                Card(
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer)
                ) {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("📊 Device Performance", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                        Spacer(Modifier.height(8.dp))

                        PerformanceRow("RAM", "${specs.totalRamMB} MB total (${specs.availableRamMB} MB free)")
                        PerformanceRow("CPU", "${specs.cpuCores} cores · ${specs.cpuArch}")
                        PerformanceRow("Storage", "${specs.freeStorageMB} MB free")
                        PerformanceRow("Android", "${specs.androidVersion} · ${specs.manufacturer} ${specs.model}")
                        PerformanceRow("Tier", specs.performanceTier().label)

                        Spacer(Modifier.height(8.dp))
                        Text(
                            "Recommended: ${specs.recommendWhisperModel().label}",
                            style = MaterialTheme.typography.bodyMedium,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.primary
                        )
                    }
                }
            }

            // ── Wake Word ──
            item {
                Card {
                    Column(modifier = Modifier.padding(16.dp)) {
                        Text("🔊 Wake Word", style = MaterialTheme.typography.titleSmall, fontWeight = FontWeight.Bold)
                        Spacer(Modifier.height(8.dp))
                        OutlinedTextField(
                            value = wakeWord,
                            onValueChange = { wakeWord = it },
                            label = { Text("Wake word phrase") },
                            singleLine = true,
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Text(
                            "Say \"$wakeWord\" to activate voice input",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }

            // ── STT Models (Whisper) ──
            item {
                Text("🗣 Speech-to-Text Models", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                Text(
                    "Whisper.cpp models — all run 100% offline after download",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            // Bolt ⚡ Optimization: Add stable key for list items
            // Model progress updates trigger frequent recompositions. Using a stable key
            // prevents Compose from unnecessarily recomposing all other unchanged items in the list.
            items(
                items = whisperModels,
                key = { it.id }
            ) { model ->
                ModelRow(
                    model = model,
                    isRecommended = when (model.id) {
                        "whisper-${specs.recommendWhisperModel().name.lowercase()}" -> true
                        else -> false
                    },
                    canRun = DevicePerformanceMetrics.canRunModel(
                        WhisperModel.valueOf(model.id.removePrefix("whisper-").uppercase()),
                        specs
                    ),
                    onDownload = { modelManager.downloadModel(model.id) },
                    onDelete = { modelManager.deleteModel(model.id) },
                )
            }

            // ── TTS Voices (Piper) ──
            item {
                Spacer(Modifier.height(8.dp))
                Text("🔊 Text-to-Speech Voices", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
                Text(
                    "Piper TTS voices — 100+ languages, ONNX format, fully offline",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            // Bolt ⚡ Optimization: Add stable key for list items
            items(
                items = piperModels,
                key = { it.id }
            ) { model ->
                ModelRow(
                    model = model,
                    isRecommended = when (model.id) {
                        "piper-${specs.recommendTtsVoice().name.lowercase()}" -> true
                        else -> false
                    },
                    canRun = true,
                    onDownload = { modelManager.downloadModel(model.id) },
                    onDelete = { modelManager.deleteModel(model.id) },
                )
            }

            // ── Storage ──
            item {
                Spacer(Modifier.height(8.dp))
                Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp),
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text("💾 Downloaded models: ${modelManager.getTotalDownloadedMB()} MB")
                    }
                }
            }
        }
    }
}

@Composable
private fun PerformanceRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(label, style = MaterialTheme.typography.bodySmall, fontWeight = FontWeight.Medium)
        Text(value, style = MaterialTheme.typography.bodySmall, fontFamily = FontFamily.Monospace)
    }
}

@Composable
private fun ModelRow(
    model: ModelDownload,
    isRecommended: Boolean,
    canRun: Boolean,
    onDownload: () -> Unit,
    onDelete: () -> Unit,
) {
    Card(
        colors = CardDefaults.cardColors(
            containerColor = if (isRecommended)
                MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.5f)
            else
                MaterialTheme.colorScheme.surface
        )
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(model.name, style = MaterialTheme.typography.bodyMedium, fontWeight = FontWeight.Medium)
                    if (isRecommended) {
                        Text(
                            " ★ Recommended",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.padding(start = 4.dp)
                        )
                    }
                }
                Text(
                    "${model.sizeMB} MB",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
                if (!canRun) {
                    Text(
                        "⚠ Insufficient RAM/storage",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.error
                    )
                }
                if (model.status == DownloadStatus.DOWNLOADING) {
                    LinearProgressIndicator(
                        progress = { model.progress },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 4.dp)
                    )
                }
            }

            Spacer(Modifier.width(8.dp))

            when (model.status) {
                DownloadStatus.NOT_DOWNLOADED, DownloadStatus.FAILED -> {
                    Button(
                        onClick = onDownload,
                        enabled = canRun,
                        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)
                    ) {
                        Text("Download")
                    }
                }
                DownloadStatus.DOWNLOADING -> {
                    Text(
                        "${(model.progress * 100).toInt()}%",
                        style = MaterialTheme.typography.labelMedium
                    )
                }
                DownloadStatus.DOWNLOADED -> {
                    OutlinedButton(
                        onClick = onDelete,
                        contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)
                    ) {
                        Text("Delete")
                    }
                }
            }
        }
    }
}
