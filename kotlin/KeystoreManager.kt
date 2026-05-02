package com.yourdomain.agent.security

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import android.content.Context
import android.content.SharedPreferences
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Hardware-backed AES-256/GCM encrypt/decrypt for API keys.
 * Uses Android Keystore with StrongBox when available.
 */
class KeystoreManager(context: Context) {

    private val alias = "agent_api_key_store"
    private val prefs: SharedPreferences =
        context.getSharedPreferences("agent_keystore_prefs", Context.MODE_PRIVATE)

    private val keyStore: KeyStore = KeyStore.getInstance("AndroidKeyStore").apply {
        load(null)
    }

    // ── Key generation ─────────────────────────────────────────

    private fun generateKey() {
        val spec = KeyGenParameterSpec.Builder(
            alias,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setIsStrongBoxBacked(true)          // prefer hardware
            .build()

        KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").apply {
            init(spec)
            generateKey()
        }
    }

    private fun getOrCreateKey(): SecretKey {
        if (!keyStore.containsAlias(alias)) generateKey()
        return (keyStore.getEntry(alias, null) as KeyStore.SecretKeyEntry).secretKey
    }

    // ── Encrypt / Decrypt ──────────────────────────────────────

    /**
     * Encrypt [plainText] and return a Base64-encoded ciphertext.
     * Format: base64(iv) + ":" + base64(ciphertext)
     */
    fun encrypt(plainText: String): String {
        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.ENCRYPT_MODE, getOrCreateKey())

        val iv = cipher.iv
        val encrypted = cipher.doFinal(plainText.toByteArray(Charsets.UTF_8))

        val ivB64 = Base64.encodeToString(iv, Base64.NO_WRAP)
        val ctB64 = Base64.encodeToString(encrypted, Base64.NO_WRAP)
        return "$ivB64:$ctB64"
    }

    /**
     * Decrypt a Base64-encoded ciphertext produced by [encrypt].
     * Expects format: base64(iv) + ":" + base64(ciphertext)
     */
    fun decrypt(encryptedBase64: String): String {
        val parts = encryptedBase64.split(":")
        require(parts.size == 2) { "Invalid ciphertext format" }

        val iv = Base64.decode(parts[0], Base64.NO_WRAP)
        val ciphertext = Base64.decode(parts[1], Base64.NO_WRAP)

        val cipher = Cipher.getInstance("AES/GCM/NoPadding")
        cipher.init(Cipher.DECRYPT_MODE, getOrCreateKey(), GCMParameterSpec(128, iv))

        return String(cipher.doFinal(ciphertext), Charsets.UTF_8)
    }

    // ── API key helpers ────────────────────────────────────────

    /**
     * Encrypt and persist an API key under a string [key].
     */
    fun saveApiKey(key: String, value: String) {
        prefs.edit().putString(key, encrypt(value)).apply()
    }

    /**
     * Retrieve and decrypt an API key stored under [key].
     * Returns `null` if the key is not found.
     */
    fun getApiKey(key: String): String? {
        val encrypted = prefs.getString(key, null) ?: return null
        return decrypt(encrypted)
    }
}
