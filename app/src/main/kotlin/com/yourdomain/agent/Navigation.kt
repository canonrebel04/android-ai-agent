package com.yourdomain.agent

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController

import com.yourdomain.agent.ui.chat.UnifiedChatScreen
import com.yourdomain.agent.TerminalScreen

enum class Screen(val label: String, val icon: ImageVector) {
    Onboarding("Onboarding", Icons.Default.SmartToy),
    ModelSetup("Model Setup", Icons.Default.Settings),
    Chat("Chat", Icons.Default.Chat),
    Home("Logs", Icons.Default.List),
    Voice("Voice", Icons.Default.Mic),
    Models("Models", Icons.Default.Star),
    Skills("Skills", Icons.Default.Build),
    Channels("Channels", Icons.Default.Settings),
    Memory("Memory", Icons.Default.Info),
    Settings("Settings", Icons.Default.Settings),
    Terminal("Terminal", Icons.Default.Code),
}

@Composable
fun AgentNavigation() {
    var showOnboarding by rememberSaveable { mutableStateOf(true) }
    val navController = rememberNavController()

    // Handle onboarding completion
    val onOnboardingComplete = {
        showOnboarding = false
    }

    if (showOnboarding) {
        OnboardingScreen(onNavigateToModelSetup = onOnboardingComplete)
    } else {
        Scaffold(
            bottomBar = {
                NavigationBar {
                    listOf(Screen.Chat, Screen.Voice, Screen.Models, Screen.Settings, Screen.Terminal).forEach { screen ->
                        NavigationBarItem(
                            icon = { Icon(screen.icon, contentDescription = screen.label) },
                            label = { Text(screen.label) },
                            selected = navController.currentBackStackEntry?.destination?.route == screen.name,
                            onClick = { 
                                navController.navigate(screen.name) {
                                    popUpTo(navController.graph.findStartDestination().id) {
                                        saveState = true
                                    }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            }
                        )
                    }
                }
            }
        ) { innerPadding ->
            // Apply padding to avoid bottom bar overlap
            Box(modifier = Modifier.padding(innerPadding)) {
                val viewModel = viewModel<AgentViewModel>()
                val state by viewModel.state.collectAsState()

                NavHost(
                    navController = navController,
                    startDestination = Screen.ModelSetup.name
                ) {
                    composable(Screen.ModelSetup.name) {
                        val context = LocalContext.current
                        val viewModel: ModelSetupViewModel = viewModel(
                            factory = ModelSetupViewModelFactory(context)
                        )
                        ModelSetupScreen(
                            viewModel = viewModel,
                            onContinue = {
                                viewModel.saveConfig()
                                navController.navigate(Screen.Chat.name) {
                                    popUpTo(Screen.ModelSetup.name) {
                                        inclusive = true
                                    }
                                }
                            }
                        )
                    }
                    composable(Screen.Chat.name) {
                        UnifiedChatScreen(
                            messages = state.chatMessages,
                            onSendMessage = { viewModel.sendChatMessage(it) },
                            modelName = state.activeModel,
                            budgetUsd = state.monthlyCost
                        )
                    }
                    composable(Screen.Home.name) {
                        HomeScreen(viewModel = viewModel)
                    }
                    composable(Screen.Voice.name) { VoiceScreen() }
                    composable(Screen.Models.name) { ModelsScreen() }
                    composable(Screen.Skills.name) { SkillsScreen() }
                    composable(Screen.Channels.name) { ChannelsScreen() }
                    composable(Screen.Memory.name) { MemoryScreen() }
                    composable(Screen.Settings.name) { SettingsScreen() }
                    composable(Screen.Terminal.name) { TerminalScreen() }
                }
            }
        }
    }
}
