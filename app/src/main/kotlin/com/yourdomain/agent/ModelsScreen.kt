package com.yourdomain.agent

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

// ── Data model ────────────────────────────────────────────────────────────────

enum class ModelTier(val label: String, val emoji: String) {
    TRIVIAL("Trivial", "\uD83D\uDC22"),
    STANDARD("Standard", "\uD83D\uDEE0\uFE0F"),
    COMPLEX("Complex", "\u2699\uFE0F"),
    CRITICAL("Critical", "\uD83D\uDD12"),
}

data class ModelTierConfig(
    val tier: ModelTier,
    val primaryModel: String,
    val fallbackModels: List<String>,
    val costEstimate: String,   // e.g. "$0.002/1K tokens"
)

// ── Screen ────────────────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ModelsScreen() {
    val tiers = remember {
        listOf(
            ModelTierConfig(
                tier = ModelTier.TRIVIAL,
                primaryModel = "llama-3.2-3b",
                fallbackModels = listOf("gemini-flash-2.0", "phi-4-mini"),
                costEstimate = "~$0.0002/1K tokens",
            ),
            ModelTierConfig(
                tier = ModelTier.STANDARD,
                primaryModel = "deepseek-v4-pro",
                fallbackModels = listOf("claude-sonnet-4-20250514", "gpt-4o"),
                costEstimate = "~$0.0015/1K tokens",
            ),
            ModelTierConfig(
                tier = ModelTier.COMPLEX,
                primaryModel = "claude-sonnet-4-20250514",
                fallbackModels = listOf("gpt-4o", "deepseek-v4-ultra"),
                costEstimate = "~$0.0060/1K tokens",
            ),
            ModelTierConfig(
                tier = ModelTier.CRITICAL,
                primaryModel = "claude-opus-4-20250514",
                fallbackModels = listOf("gpt-4.1", "deepseek-v4-ultra"),
                costEstimate = "~$0.0150/1K tokens",
            ),
        )
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("🤖 Model Tiers")
                        Text(
                            text = "Select primary model and fallbacks per complexity tier",
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
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 16.dp, vertical = 8.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            items(tiers, key = { it.tier.name }) { config ->
                TierCard(config)
            }
        }
    }
}

// ── Tier Card ─────────────────────────────────────────────────────────────────

@Composable
private fun TierCard(config: ModelTierConfig) {
    var expanded by remember { mutableStateOf(false) }

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = when (config.tier) {
                ModelTier.TRIVIAL -> MaterialTheme.colorScheme.surfaceVariant
                ModelTier.STANDARD -> MaterialTheme.colorScheme.surface
                ModelTier.COMPLEX -> MaterialTheme.colorScheme.secondaryContainer
                ModelTier.CRITICAL -> MaterialTheme.colorScheme.primaryContainer
            },
        ),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            // ── Header row: tier label + primary model ──
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(
                        text = config.tier.emoji,
                        style = MaterialTheme.typography.titleLarge,
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Column {
                        Text(
                            text = config.tier.label,
                            style = MaterialTheme.typography.titleMedium,
                            fontWeight = FontWeight.Bold,
                        )
                        Text(
                            text = "primary: ${config.primaryModel}",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                Icon(
                    imageVector = Icons.Default.Star,
                    contentDescription = null,
                    modifier = Modifier.size(20.dp),
                    tint = when (config.tier) {
                        ModelTier.TRIVIAL -> MaterialTheme.colorScheme.outline
                        ModelTier.STANDARD -> MaterialTheme.colorScheme.primary.copy(alpha = 0.5f)
                        ModelTier.COMPLEX -> MaterialTheme.colorScheme.primary.copy(alpha = 0.7f)
                        ModelTier.CRITICAL -> MaterialTheme.colorScheme.primary
                    },
                )
            }

            Spacer(modifier = Modifier.height(8.dp))

            // ── Cost estimate ──
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "Cost: ",
                    style = MaterialTheme.typography.bodySmall,
                    fontWeight = FontWeight.SemiBold,
                )
                Text(
                    text = config.costEstimate,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            // ── Fallback chips ──
            if (config.fallbackModels.isNotEmpty()) {
                Spacer(modifier = Modifier.height(8.dp))
                Text(
                    text = "Fallbacks:",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Spacer(modifier = Modifier.height(4.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    config.fallbackModels.forEach { fallback ->
                        FilterChip(
                            selected = false,
                            onClick = { /* read-only display */ },
                            label = {
                                Text(
                                    text = fallback,
                                    style = MaterialTheme.typography.labelSmall,
                                )
                            },
                            leadingIcon = {
                                Icon(
                                    imageVector = Icons.Default.CheckCircle,
                                    contentDescription = null,
                                    modifier = Modifier.size(14.dp),
                                )
                            },
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            // ── Configure button ──
            Button(
                onClick = { expanded = !expanded },
                modifier = Modifier.fillMaxWidth(),
            ) {
                Text(if (expanded) "▲ Collapse" else "⚙ Configure")
            }

            // ── Expanded details ──
            if (expanded) {
                Spacer(modifier = Modifier.height(12.dp))
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text(
                        text = "DETAILS",
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    DetailRow("Tier", config.tier.label)
                    DetailRow("Primary", config.primaryModel)
                    DetailRow("Fallbacks", config.fallbackModels.joinToString(", "))
                    DetailRow("Estimated Cost", config.costEstimate)
                    DetailRow(
                        "Max Tokens",
                        when (config.tier) {
                            ModelTier.TRIVIAL -> "4K"
                            ModelTier.STANDARD -> "32K"
                            ModelTier.COMPLEX -> "128K"
                            ModelTier.CRITICAL -> "200K"
                        },
                    )
                    DetailRow(
                        "Timeout",
                        when (config.tier) {
                            ModelTier.TRIVIAL -> "30s"
                            ModelTier.STANDARD -> "120s"
                            ModelTier.COMPLEX -> "600s"
                            ModelTier.CRITICAL -> "1800s"
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun DetailRow(label: String, value: String) {
    Row(modifier = Modifier.fillMaxWidth()) {
        Text(
            text = "$label:",
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.SemiBold,
            modifier = Modifier.width(100.dp),
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
