package com.yourdomain.agent

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
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
        // Apply padding to avoid bottom bar overlap
        androidx.compose.foundation.layout.Box(modifier = androidx.compose.ui.Modifier.padding(padding)) {
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
}
