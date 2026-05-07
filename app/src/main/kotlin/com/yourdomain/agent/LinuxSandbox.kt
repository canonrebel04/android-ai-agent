package com.yourdomain.agent

import android.content.Context
import android.content.res.AssetManager
import android.util.Log
import java.io.*
import java.nio.file.Files
import java.nio.file.StandardCopyOption

/**
 * LinuxSandbox - Manages a sandboxed Linux environment using PRoot
 * 
 * This class provides functionality to:
 * - Initialize a PRoot environment with Debian rootfs
 * - Execute shell commands in the sandbox
 * - Install packages using apt
 * - Handle file system operations
 */
class LinuxSandbox(private val context: Context) {
    companion object {
        private const val TAG = "LinuxSandbox"
        private const val ROOTFS_ASSET = "debian-rootfs.tar.gz"
        private const val ROOTFS_DIR = "linux-rootfs"
        private const val PROOT_LIB = "libproot.so"
        
        // Supported CPU architectures
        private val SUPPORTED_ARCHITECTURES = arrayOf(
            "arm64-v8a",
            "armeabi-v7a",
            "x86_64"
        )
    }
    
    private var rootfsPath: String = ""
    private var prootBinaryPath: String = ""
    private var isInitialized: Boolean = false
    private var tempDir: File? = null
    
    /**
     * Initialize the sandbox environment
     * 
     * @return true if initialization successful, false otherwise
     */
    fun start(): Boolean {
        return try {
            Log.d(TAG, "Starting LinuxSandbox initialization")
            
            // Create temporary directory for rootfs
            tempDir = File(context.cacheDir, ROOTFS_DIR)
            if (tempDir?.exists() == true) {
                Log.d(TAG, "Cleaning up existing temp directory")
                tempDir?.deleteRecursively()
            }
            tempDir?.mkdirs()
            
            // Extract rootfs from assets
            if (!extractRootfs()) {
                Log.e(TAG, "Failed to extract rootfs")
                return false
            }
            
            // Resolve proot binary path based on CPU architecture
            prootBinaryPath = resolveProotBinary()
            if (prootBinaryPath.isEmpty()) {
                Log.e(TAG, "Failed to resolve proot binary path")
                return false
            }
            
            Log.d(TAG, "PRoot binary path: $prootBinaryPath")
            Log.d(TAG, "Rootfs path: $rootfsPath")
            
            // Set executable permissions on proot binary
            if (!setProotExecutable()) {
                Log.e(TAG, "Failed to set executable permissions on proot")
                return false
            }
            
            isInitialized = true
            Log.d(TAG, "LinuxSandbox initialized successfully")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Error during initialization: ${e.message}")
            e.printStackTrace()
            false
        }
    }
    
    /**
     * Extract rootfs from assets to temporary directory
     * 
     * @return true if extraction successful, false otherwise
     */
    private fun extractRootfs(): Boolean {
        return try {
            Log.d(TAG, "Extracting rootfs from assets")
            
            val assetManager: AssetManager = context.assets
            val inputStream = assetManager.open(ROOTFS_ASSET)
            
            // Create target file
            val outputFile = File(tempDir, ROOTFS_ASSET)
            rootfsPath = outputFile.absolutePath
            
            // Copy asset to temp directory
            Files.copy(
                inputStream,
                outputFile.toPath(),
                StandardCopyOption.REPLACE_EXISTING
            )
            
            // Extract tar.gz
            val extractDir = File(tempDir, "rootfs")
            extractDir.mkdirs()
            
            val tarCommand = arrayOf(
                "tar", "-xzf", rootfsPath, "-C", extractDir.absolutePath
            )
            
            val process = ProcessBuilder(*tarCommand)
                .redirectErrorStream(true)
                .start()
            
            val exitCode = process.waitFor()
            if (exitCode != 0) {
                Log.e(TAG, "Failed to extract tar.gz, exit code: $exitCode")
                val errorOutput = process.inputStream.bufferedReader().readText()
                Log.e(TAG, "Error output: $errorOutput")
                return false
            }
            
            // Update rootfs path to extracted directory
            rootfsPath = extractDir.absolutePath
            Log.d(TAG, "Rootfs extracted successfully to: $rootfsPath")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Error extracting rootfs: ${e.message}")
            e.printStackTrace()
            false
        }
    }
    
    /**
     * Resolve the proot binary path based on CPU architecture
     * 
     * @return Path to proot binary, or empty string if not found
     */
    private fun resolveProotBinary(): String {
        val cpuAbi = getCpuArchitecture()
        Log.d(TAG, "Detected CPU architecture: $cpuAbi")
        
        if (cpuAbi.isEmpty()) {
            Log.e(TAG, "Unsupported CPU architecture")
            return ""
        }
        
        // Path to jniLibs directory
        val jniLibsDir = File(context.applicationInfo.nativeLibraryDir)
        
        // Check if proot binary exists in the architecture-specific directory
        val prootFile = File(jniLibsDir, PROOT_LIB)
        if (prootFile.exists()) {
            Log.d(TAG, "Found proot binary at: ${prootFile.absolutePath}")
            return prootFile.absolutePath
        }
        
        // Alternative: Check in assets or other locations
        Log.e(TAG, "PRoot binary not found in expected location: ${jniLibsDir.absolutePath}")
        return ""
    }
    
    /**
     * Get the CPU architecture
     * 
     * @return CPU architecture string (e.g., "arm64-v8a", "x86_64")
     */
    private fun getCpuArchitecture(): String {
        val cpuAbi = try {
            // Try to get ABI from Build.SUPPORTED_ABIS
            android.os.Build.SUPPORTED_ABIS.firstOrNull() ?: ""
        } catch (e: Exception) {
            ""
        }
        
        // Fallback: Check system properties
        if (cpuAbi.isEmpty()) {
            try {
                val abi = System.getProperty("os.arch")
                when (abi) {
                    "aarch64" -> return "arm64-v8a"
                    "armv7l", "armv8l" -> return "armeabi-v7a"
                    "amd64", "x86_64" -> return "x86_64"
                    else -> return ""
                }
            } catch (e: Exception) {
                return ""
            }
        }
        
        // Validate against supported architectures
        return if (SUPPORTED_ARCHITECTURES.contains(cpuAbi)) {
            cpuAbi
        } else {
            Log.w(TAG, "Detected ABI '$cpuAbi' not in supported list, trying fallback")
            // Try to find a matching architecture
            when {
                cpuAbi.contains("arm64") -> "arm64-v8a"
                cpuAbi.contains("arm") -> "armeabi-v7a"
                cpuAbi.contains("x86_64") || cpuAbi.contains("amd64") -> "x86_64"
                else -> ""
            }
        }
    }
    
    /**
     * Set executable permissions on the proot binary
     * 
     * @return true if successful, false otherwise
     */
    private fun setProotExecutable(): Boolean {
        return try {
            val prootFile = File(prootBinaryPath)
            if (!prootFile.exists()) {
                Log.e(TAG, "PRoot file does not exist: $prootBinaryPath")
                return false
            }
            
            // Set executable permissions (755)
            prootFile.setExecutable(true)
            prootFile.setReadable(true, false)
            prootFile.setWritable(true, false)
            
            Log.d(TAG, "Set executable permissions on proot binary")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Error setting executable permissions: ${e.message}")
            e.printStackTrace()
            false
        }
    }
    
    /**
     * Execute a shell command in the sandboxed environment
     * 
     * @param command The command to execute
     * @param timeoutMs Timeout in milliseconds (default: 30000)
     * @return Pair of (exitCode, output) or null on error
     */
    fun executeCommand(command: String, timeoutMs: Long = 30000): Pair<Int, String>? {
        if (!isInitialized) {
            Log.e(TAG, "Sandbox not initialized. Call start() first.")
            return null
        }
        
        if (prootBinaryPath.isEmpty() || rootfsPath.isEmpty()) {
            Log.e(TAG, "PRoot or rootfs path not configured")
            return null
        }
        
        return try {
            Log.d(TAG, "Executing command: $command")
            
            // Build the proot command
            val prootCommand = arrayOf(
                prootBinaryPath,
                "-S", rootfsPath,
                "/bin/sh", "-c", command
            )
            
            Log.d(TAG, "Full command: ${prootCommand.joinToString(" ")}")
            
            val process = ProcessBuilder(*prootCommand)
                .redirectErrorStream(true)
                .start()
            
            // Wait for process to complete with timeout
            val startTime = System.currentTimeMillis()
            var exitCode: Int? = null
            var output = ""
            
            val outputThread = Thread {
                val reader = process.inputStream.bufferedReader()
                val builder = StringBuilder()
                var line: String?
                while (reader.readLine().also { line = it } != null) {
                    builder.append(line).append("\n")
                    if (System.currentTimeMillis() - startTime > timeoutMs) {
                        break
                    }
                }
                output = builder.toString()
            }
            
            outputThread.start()
            
            // Wait for process with timeout
            val remainingTime = timeoutMs - (System.currentTimeMillis() - startTime)
            if (remainingTime > 0) {
                exitCode = if (process.waitFor(remainingTime, java.util.concurrent.TimeUnit.MILLISECONDS)) {
                    process.exitValue()
                } else {
                    process.destroyForcibly()
                    -1 // Timeout
                }
            } else {
                process.destroyForcibly()
                exitCode = -1
            }
            
            outputThread.join(1000) // Give thread time to finish
            
            if (exitCode == null) {
                exitCode = process.exitValue()
            }
            
            Log.d(TAG, "Command executed with exit code: $exitCode")
            if (output.isNotEmpty()) {
                Log.d(TAG, "Command output: $output")
            }
            
            Pair(exitCode, output)
        } catch (e: Exception) {
            Log.e(TAG, "Error executing command: ${e.message}")
            e.printStackTrace()
            null
        }
    }
    
    /**
     * Install a package using apt in the sandboxed environment
     * 
     * @param packageName The package name to install
     * @param timeoutMs Timeout in milliseconds (default: 60000)
     * @return true if installation successful, false otherwise
     */
    fun installPackage(packageName: String, timeoutMs: Long = 60000): Boolean {
        if (!isInitialized) {
            Log.e(TAG, "Sandbox not initialized. Call start() first.")
            return false
        }
        
        Log.d(TAG, "Installing package: $packageName")
        
        // First, update apt package list
        val updateResult = executeCommand("apt-get update -y", timeoutMs)
        if (updateResult == null || updateResult.first != 0) {
            Log.e(TAG, "Failed to update apt package list")
            return false
        }
        
        // Install the package
        val installResult = executeCommand("apt-get install -y $packageName", timeoutMs)
        if (installResult == null || installResult.first != 0) {
            Log.e(TAG, "Failed to install package: $packageName")
            Log.e(TAG, "Install output: ${installResult?.second}")
            return false
        }
        
        Log.d(TAG, "Package installed successfully: $packageName")
        return true
    }
    
    /**
     * Check if a file exists in the sandboxed environment
     * 
     * @param path The path to check
     * @return true if file exists, false otherwise
     */
    fun fileExists(path: String): Boolean {
        if (!isInitialized) {
            Log.e(TAG, "Sandbox not initialized. Call start() first.")
            return false
        }
        
        val result = executeCommand("test -e '$path' && echo 'EXISTS' || echo 'NOT_EXISTS'")
        return result?.second?.trim() == "EXISTS"
    }
    
    /**
     * Read a file from the sandboxed environment
     * 
     * @param path The path to the file
     * @return File content as string, or null on error
     */
    fun readFile(path: String): String? {
        if (!isInitialized) {
            Log.e(TAG, "Sandbox not initialized. Call start() first.")
            return null
        }
        
        val result = executeCommand("cat '$path'")
        return if (result?.first == 0) {
            result.second
        } else {
            null
        }
    }
    
    /**
     * Write a file in the sandboxed environment
     * 
     * @param path The path to the file
     * @param content The content to write
     * @return true if successful, false otherwise
     */
    fun writeFile(path: String, content: String): Boolean {
        if (!isInitialized) {
            Log.e(TAG, "Sandbox not initialized. Call start() first.")
            return false
        }
        
        // Escape special characters in content
        val escapedContent = content
            .replace("\$", "\\\$")
            .replace("`", "\\`")
            .replace("\"", "\\\"")
            .replace("'", "'\\''")
        
        val result = executeCommand("echo '$escapedContent' > '$path'")
        return result?.first == 0
    }
    
    /**
     * Create a directory in the sandboxed environment
     * 
     * @param path The path to the directory
     * @return true if successful, false otherwise
     */
    fun createDirectory(path: String): Boolean {
        if (!isInitialized) {
            Log.e(TAG, "Sandbox not initialized. Call start() first.")
            return false
        }
        
        val result = executeCommand("mkdir -p '$path'")
        return result?.first == 0
    }
    
    /**
     * List files in a directory in the sandboxed environment
     * 
     * @param path The path to the directory
     * @return List of file names, or null on error
     */
    fun listFiles(path: String): List<String>? {
        if (!isInitialized) {
            Log.e(TAG, "Sandbox not initialized. Call start() first.")
            return null
        }
        
        val result = executeCommand("ls -1 '$path'")
        return if (result?.first == 0) {
            result.second.split("\n").filter { it.isNotEmpty() }
        } else {
            null
        }
    }
    
    /**
     * Clean up the sandbox environment
     */
    fun cleanup() {
        Log.d(TAG, "Cleaning up LinuxSandbox")
        
        try {
            // Kill any running proot processes
            executeCommand("pkill -f proot", 5000)
            
            // Remove temporary directory
            tempDir?.deleteRecursively()
            tempDir = null
            
            rootfsPath = ""
            prootBinaryPath = ""
            isInitialized = false
            
            Log.d(TAG, "LinuxSandbox cleaned up successfully")
        } catch (e: Exception) {
            Log.e(TAG, "Error during cleanup: ${e.message}")
            e.printStackTrace()
        }
    }
    
    /**
     * Check if the sandbox is initialized
     * 
     * @return true if initialized, false otherwise
     */
    fun isSandboxInitialized(): Boolean {
        return isInitialized
    }
    
    /**
     * Get the rootfs path
     * 
     * @return The rootfs path
     */
    fun getRootfsPath(): String {
        return rootfsPath
    }
    
    /**
     * Get the proot binary path
     * 
     * @return The proot binary path
     */
    fun getProotBinaryPath(): String {
        return prootBinaryPath
    }
}
