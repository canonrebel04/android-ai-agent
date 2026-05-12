package com.yourdomain.agent.ui.chat

import androidx.compose.animation.*
import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.platform.LocalFocusManager
import com.yourdomain.agent.ChatMessage
import kotlinx.coroutines.launch

@Composable
fun UnifiedChatScreen(
    messages: List<ChatMessage>,
    onSendMessage: (String) -> Unit,
    modelName: String = "claude-3-5-sonnet",
    budgetUsd: String = "$0.42"
) {
    var inputText by remember { mutableStateOf("") }
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val focusManager = LocalFocusManager.current

    // Auto-scroll to bottom on new messages
    LaunchedEffect(messages.size) {
        if (messages.isNotEmpty()) {
            listState.animateScrollToItem(messages.size - 1)
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF0A0C10)) // Deep Terminal Dark
    ) {
        // Background Grid Pattern (Cyberpunk vibe)
        Box(
            modifier = Modifier
                .fillMaxSize()
                .drawBehind {
                    val gridSize = 40.dp.toPx()
                    val color = Color(0xFF1A1D23)
                    for (x in 0..size.width.toInt() step gridSize.toInt()) {
                        drawLine(color, Offset(x.toFloat(), 0f), Offset(x.toFloat(), size.height), strokeWidth = 1f)
                    }
                    for (y in 0..size.height.toInt() step gridSize.toInt()) {
                        drawLine(color, Offset(0f, y.toFloat()), Offset(size.width, y.toFloat()), strokeWidth = 1f)
                    }
                }
        )

        Column(modifier = Modifier.fillMaxSize()) {
            // Status Header
            StatusHeader(modelName, budgetUsd)

            // Message List
            LazyColumn(
                state = listState,
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp),
                verticalArrangement = Arrangement.spacedBy(16.dp),
                contentPadding = PaddingValues(top = 16.dp, bottom = 100.dp)
            ) {
                // Bolt ⚡ Optimization: Add key for Compose list rendering
                // Using a stable key prevents Compose from recomposing existing items
                // when new items are added, which significantly improves chat scroll performance.
                itemsIndexed(
                    items = messages,
                    key = { _, message -> message.id }
                ) { index, message ->
                    AnimatedMessageItem(message, index)
                }
            }
        }

        // Bottom Input Area
        Box(
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .background(Color(0xCC0A0C10)) // Translucent background
                .padding(16.dp)
        ) {
            ChatInput(
                value = inputText,
                onValueChange = { inputText = it },
                onSend = {
                    if (inputText.isNotBlank()) {
                        onSendMessage(inputText)
                        inputText = ""
                        focusManager.clearFocus()
                    }
                }
            )
        }
    }
}

@Composable
fun StatusHeader(modelName: String, budgetUsd: String) {
    Surface(
        color = Color(0xFF12151C),
        tonalElevation = 8.dp,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = 16.dp, vertical = 12.dp)
                .fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column {
                Text(
                    text = "ACTIVE_MODEL",
                    style = MaterialTheme.typography.labelSmall,
                    color = Color(0xFF5C6370),
                    letterSpacing = 1.sp
                )
                Text(
                    text = modelName.uppercase(),
                    style = MaterialTheme.typography.bodySmall,
                    color = Color(0xFF00E5FF), // Cyber Neon Cyan
                    fontWeight = FontWeight.Bold,
                    fontFamily = FontFamily.Monospace
                )
            }
            Column(horizontalAlignment = Alignment.End) {
                Text(
                    text = "SESSION_COST",
                    style = MaterialTheme.typography.labelSmall,
                    color = Color(0xFF5C6370),
                    letterSpacing = 1.sp
                )
                Text(
                    text = budgetUsd,
                    style = MaterialTheme.typography.bodySmall,
                    color = Color(0xFFFFB300), // Amber
                    fontWeight = FontWeight.Bold,
                    fontFamily = FontFamily.Monospace
                )
            }
        }
    }
}

@Composable
fun AnimatedMessageItem(message: ChatMessage, index: Int) {
    var visible by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) {
        visible = true
    }

    AnimatedVisibility(
        visible = visible,
        enter = fadeIn(animationSpec = tween(300, delayMillis = 50)) +
                slideInVertically(initialOffsetY = { 20 }, animationSpec = spring(dampingRatio = Spring.DampingRatioLowBouncy))
    ) {
        ChatBubble(message)
    }
}

@Composable
fun ChatBubble(message: ChatMessage) {
    val isUser = message.role == "user"
    val accentColor = if (isUser) Color(0xFF00E5FF) else Color(0xFF9DFF00) // Cyan vs Neon Green
    val backgroundColor = if (isUser) Color(0xFF1E252D) else Color(0xFF161B22)

    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = if (isUser) Alignment.End else Alignment.Start
    ) {
        Row(
            verticalAlignment = Alignment.Top,
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = if (isUser) Arrangement.End else Arrangement.Start
        ) {
            if (!isUser) {
                // Agent Avatar Placeholder
                Box(
                    modifier = Modifier
                        .size(24.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(accentColor.copy(alpha = 0.2f))
                        .padding(4.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text("H", color = accentColor, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                }
                Spacer(modifier = Modifier.width(8.dp))
            }

            Card(
                shape = RoundedCornerShape(
                    topStart = 12.dp,
                    topEnd = 12.dp,
                    bottomStart = if (isUser) 12.dp else 2.dp,
                    bottomEnd = if (isUser) 2.dp else 12.dp
                ),
                colors = CardDefaults.cardColors(containerColor = backgroundColor),
                modifier = Modifier
                    .widthIn(max = 300.dp)
                    .graphicsLayer {
                        // Subtle shadow/depth effect
                        shadowElevation = 2f
                    }
            ) {
                Column(modifier = Modifier.padding(12.dp)) {
                    Text(
                        text = message.content,
                        color = Color(0xFFE1E4E8),
                        style = MaterialTheme.typography.bodyMedium.copy(
                            lineHeight = 20.sp,
                            fontFamily = FontFamily.Monospace
                        )
                    )
                }
            }

            if (isUser) {
                Spacer(modifier = Modifier.width(8.dp))
                // User Avatar Placeholder
                Box(
                    modifier = Modifier
                        .size(24.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(accentColor.copy(alpha = 0.2f))
                        .padding(4.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text("U", color = accentColor, fontSize = 12.sp, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}

@Composable
fun ChatInput(
    value: String,
    onValueChange: (String) -> Unit,
    onSend: () -> Unit
) {
    var isPressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(if (isPressed) 0.96f else 1f)

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(24.dp))
            .background(Color(0xFF1C2128))
            .padding(horizontal = 16.dp, vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        TextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.weight(1f),
            colors = TextFieldDefaults.colors(
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent,
                disabledContainerColor = Color.Transparent,
                focusedIndicatorColor = Color.Transparent,
                unfocusedIndicatorColor = Color.Transparent,
                cursorColor = Color(0xFF00E5FF),
                focusedTextColor = Color.White,
                unfocusedTextColor = Color.LightGray
            ),
            placeholder = {
                Text(
                    "TYPE_COMMAND...",
                    style = MaterialTheme.typography.bodyMedium,
                    color = Color(0xFF5C6370),
                    fontFamily = FontFamily.Monospace
                )
            },
            maxLines = 5,
            textStyle = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onSend = { onSend() })
        )

        IconButton(
            onClick = onSend,
            modifier = Modifier
                .graphicsLayer {
                    scaleX = scale
                    scaleY = scale
                }
        ) {
            Icon(
                imageVector = Icons.Default.Send,
                contentDescription = "Send",
                tint = if (value.isNotBlank()) Color(0xFF00E5FF) else Color(0xFF5C6370)
            )
        }
    }
}
