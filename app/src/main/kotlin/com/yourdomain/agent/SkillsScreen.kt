package com.yourdomain.agent

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

// ── Data model ────────────────────────────────────────────────────────────────

enum class Complexity {
    LOW, MEDIUM, HIGH;

    val label: String
        get() = when (this) {
            LOW -> "Low"
            MEDIUM -> "Medium"
            HIGH -> "High"
        }

    val emoji: String
        get() = when (this) {
            LOW -> "\uD83D\uDCA1"
            MEDIUM -> "\uD83D\uDD0D"
            HIGH -> "\uD83D\uDD25"
        }
}

data class SkillInfo(
    val name: String,
    val description: String,
    val complexity: Complexity,
    val usageCount: Int,
    val enabled: Boolean = true,
)

// ── Screen ────────────────────────────────────────────────────────────────────

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SkillsScreen() {
    val skills = remember {
        mutableStateOf(
            listOf(
                SkillInfo(
                    name = "File Reader",
                    description = "Read and analyze text files and documents from the local filesystem with content extraction.",
                    complexity = Complexity.LOW,
                    usageCount = 1_247,
                ),
                SkillInfo(
                    name = "Terminal Commander",
                    description = "Execute shell commands, manage processes, and run system utilities securely.",
                    complexity = Complexity.MEDIUM,
                    usageCount = 3_891,
                ),
                SkillInfo(
                    name = "Code Search",
                    description = "Search across codebases using regex patterns, glob matching, and semantic analysis.",
                    complexity = Complexity.MEDIUM,
                    usageCount = 892,
                ),
                SkillInfo(
                    name = "Vision Analyzer",
                    description = "Analyze images, screenshots, and visual data using multimodal vision models.",
                    complexity = Complexity.MEDIUM,
                    usageCount = 456,
                ),
                SkillInfo(
                    name = "Git Manager",
                    description = "Perform git operations including commits, branching, diff analysis, and history browsing.",
                    complexity = Complexity.MEDIUM,
                    usageCount = 2_103,
                ),
                SkillInfo(
                    name = "Self-Improvement Engine",
                    description = "Analyze execution history to optimize agent prompts, tool selection, and decision-making strategies.",
                    complexity = Complexity.HIGH,
                    usageCount = 67,
                ),
                SkillInfo(
                    name = "Network Orchestrator",
                    description = "Coordinate multi-agent workflows across remote nodes with load balancing and failover.",
                    complexity = Complexity.HIGH,
                    usageCount = 12,
                    enabled = false,
                ),
                SkillInfo(
                    name = "Database Query",
                    description = "Execute and optimize SQL queries with schema introspection across MySQL, PostgreSQL, and SQLite.",
                    complexity = Complexity.MEDIUM,
                    usageCount = 534,
                ),
            )
        )
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("\uD83E\uDDE0 Skills")
                        Text(
                            text = "${skills.value.size} installed  •  ${skills.value.count { it.enabled }} enabled",
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
            items(skills.value, key = { it.name }) { skill ->
                SkillCard(
                    skill = skill,
                    onToggle = { enabled ->
                        skills.value = skills.value.map {
                            if (it.name == skill.name) it.copy(enabled = enabled) else it
                        }
                    },
                )
            }
        }
    }
}

// ── Skill Card ────────────────────────────────────────────────────────────────

@Composable
private fun SkillCard(
    skill: SkillInfo,
    onToggle: (Boolean) -> Unit,
) {
    var checked by remember(skill.name) { mutableStateOf(skill.enabled) }

    val complexityColor = when (skill.complexity) {
        Complexity.LOW -> MaterialTheme.colorScheme.tertiary
        Complexity.MEDIUM -> MaterialTheme.colorScheme.secondary
        Complexity.HIGH -> MaterialTheme.colorScheme.error
    }

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(
            containerColor = if (checked)
                MaterialTheme.colorScheme.surface
            else
                MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.6f),
        ),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            // ── Header: name + switch ──
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = skill.name,
                    style = MaterialTheme.typography.titleSmall,
                    fontWeight = FontWeight.Bold,
                    modifier = Modifier.weight(1f),
                )
                Switch(
                    checked = checked,
                    onCheckedChange = { newValue ->
                        checked = newValue
                        onToggle(newValue)
                    },
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = MaterialTheme.colorScheme.primary,
                        checkedTrackColor = MaterialTheme.colorScheme.primaryContainer,
                        uncheckedThumbColor = MaterialTheme.colorScheme.outline,
                        uncheckedTrackColor = MaterialTheme.colorScheme.surfaceVariant,
                    ),
                )
            }

            Spacer(modifier = Modifier.height(6.dp))

            // ── Description ──
            Text(
                text = skill.description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )

            Spacer(modifier = Modifier.height(10.dp))

            // ── Bottom row: complexity badge + usage count ──
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                // Complexity badge
                Surface(
                    shape = RoundedCornerShape(12.dp),
                    color = complexityColor.copy(alpha = 0.15f),
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = skill.complexity.emoji,
                            style = MaterialTheme.typography.labelSmall,
                        )
                        Spacer(modifier = Modifier.width(4.dp))
                        Text(
                            text = skill.complexity.label,
                            style = MaterialTheme.typography.labelSmall,
                            fontWeight = FontWeight.SemiBold,
                            color = complexityColor,
                        )
                    }
                }

                // Usage count
                Text(
                    text = "used ${formatCount(skill.usageCount)}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }

            // ── Enabled/disabled indicator ──
            AnimatedVisibility(
                visible = !checked,
                enter = expandVertically(),
                exit = shrinkVertically(),
            ) {
                Text(
                    text = "⚠ Skill is disabled — it will not be loaded for tasks",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.error,
                    modifier = Modifier.padding(top = 8.dp),
                )
            }
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

private fun formatCount(count: Int): String = when {
    count >= 1_000_000 -> "${count / 1_000_000}.${(count % 1_000_000) / 100_000}M"
    count >= 1_000 -> "${count / 1_000}.${(count % 1_000) / 100}K"
    else -> count.toString()
}
