package com.yourdomain.agent

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.PixelFormat
import android.hardware.display.DisplayManager
import android.hardware.display.VirtualDisplay
import android.media.ImageReader
import android.media.projection.MediaProjection
import android.media.projection.MediaProjectionManager
import android.os.Build
import android.os.IBinder
import android.util.DisplayMetrics
import java.io.ByteArrayOutputStream

/**
 * MediaProjectionService — captures screenshots for Vision Mode.
 * Started by SettingsScreen when visionMode is toggled on.
 * Requires FOREGROUND_SERVICE_MEDIA_PROJECTION permission (Android 14+).
 */
class MediaProjectionService : Service() {

    private var mediaProjection: MediaProjection? = null
    private var virtualDisplay: VirtualDisplay? = null
    private var imageReader: ImageReader? = null

    companion object {
        const val CHANNEL_ID = "media_projection_channel"
        const val NOTIFICATION_ID = 3001
        const val EXTRA_RESULT_CODE = "result_code"
        const val EXTRA_DATA = "data"
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val resultCode = intent?.getIntExtra(EXTRA_RESULT_CODE, -1) ?: -1
        val data = intent?.getParcelableExtra(EXTRA_DATA, Intent::class.java) ?: return START_NOT_STICKY

        if (resultCode != -1 && data != null) {
            startForeground(NOTIFICATION_ID, buildNotification())
            startCapture(resultCode, data)
        }

        return START_STICKY
    }

    private fun startCapture(resultCode: Int, data: Intent) {
        val projectionManager = getSystemService(Context.MEDIA_PROJECTION_SERVICE) as MediaProjectionManager
        mediaProjection = projectionManager.getMediaProjection(resultCode, data)

        val metrics = resources.displayMetrics
        val width = metrics.widthPixels
        val height = metrics.heightPixels
        val density = metrics.densityDpi

        imageReader = ImageReader.newInstance(
            width, height,
            PixelFormat.RGBA_8888, 2
        )

        virtualDisplay = mediaProjection?.createVirtualDisplay(
            "AgentVision",
            width, height, density,
            DisplayManager.VIRTUAL_DISPLAY_FLAG_AUTO_MIRROR,
            imageReader?.surface, null, null
        )
    }

    /**
     * Capture a single screenshot and return it as JPEG bytes.
     * Called by Rust core via JNI when vision mode is active.
     */
    fun captureScreenshot(): ByteArray? {
        val reader = imageReader ?: return null
        val image = reader.acquireLatestImage() ?: return null

        return try {
            val planes = image.planes
            val buffer = planes[0].buffer
            val pixelStride = planes[0].pixelStride
            val rowStride = planes[0].rowStride
            val rowPadding = rowStride - pixelStride * image.width

            val bitmap = Bitmap.createBitmap(
                image.width + rowPadding / pixelStride,
                image.height,
                Bitmap.Config.ARGB_8888
            )
            bitmap.copyPixelsFromBuffer(buffer)

            val cropped = Bitmap.createBitmap(bitmap, 0, 0, image.width, image.height)
            val stream = ByteArrayOutputStream()
            cropped.compress(Bitmap.CompressFormat.JPEG, 80, stream)

            bitmap.recycle()
            cropped.recycle()
            stream.toByteArray()
        } finally {
            image.close()
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        virtualDisplay?.release()
        imageReader?.close()
        mediaProjection?.stop()
        super.onDestroy()
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "Vision Mode",
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = "Screen capture for Vision Mode"
        }
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(channel)
    }

    private fun buildNotification(): Notification {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
                .setContentTitle("Agent Vision Mode")
                .setContentText("Screen capture active")
                .setSmallIcon(android.R.drawable.ic_menu_camera)
                .build()
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
                .setContentTitle("Agent Vision Mode")
                .setContentText("Screen capture active")
                .setSmallIcon(android.R.drawable.ic_menu_camera)
                .build()
        }
    }
}
