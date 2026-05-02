package com.yourdomain.agent

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector

enum class Screen(val label: String, val icon: ImageVector) {
    Home("Home", Icons.Default.Home),
    Voice("Voice", Icons.Default.Mic),
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
                listOf(Screen.Home, Screen.Voice, Screen.Models, Screen.Skills).forEach { screen ->
                    NavigationBarItem(
                        icon = { Icon(screen.icon, contentDescription = screen.label) },
                        label = { Text(screen.label) },
                        selected = currentScreen == screen,
                        onClick = { currentScreen = screen }
                    )
                }
            }
        }
    ) { innerPadding ->
        // Apply padding to avoid bottom bar overlap
        Box(modifier = Modifier.padding(innerPadding)) {
            when (currentScreen) {
                Screen.Home -> {
                    val viewModel = viewModel<AgentViewModel>()
                    HomeScreen(viewModel = viewModel)
                }
                Screen.Voice -> VoiceScreen()
                Screen.Models -> ModelsScreen()
                Screen.Skills -> SkillsScreen()
                Screen.Channels -> ChannelsScreen()
                Screen.Memory -> MemoryScreen()
                Screen.Settings -> SettingsScreen()
            }
        }
    }
}
