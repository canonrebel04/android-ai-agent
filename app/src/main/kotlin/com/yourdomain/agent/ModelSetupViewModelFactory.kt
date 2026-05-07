package com.yourdomain.agent

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider

/**
 * Factory for creating ModelSetupViewModel with required dependencies
 */
class ModelSetupViewModelFactory(
    private val context: Context
) : ViewModelProvider.Factory {
    
    @Suppress("UNCHECKED_CAST")
    override fun <T : ViewModel> create(modelClass: Class<T>): T {
        if (modelClass.isAssignableFrom(ModelSetupViewModel::class.java)) {
            return ModelSetupViewModel(context) as T
        }
        throw IllegalArgumentException("Unknown ViewModel class")
    }
}
