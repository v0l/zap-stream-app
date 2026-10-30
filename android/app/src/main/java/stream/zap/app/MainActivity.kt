/**
 * lumina-video Demo Activity
 *
 * Main activity for the lumina-video demo application. This activity uses GameActivity
 * from the Android Game SDK to host the native Rust/egui application.
 *
 * # Architecture
 *
 * ```
 * MainActivity (Kotlin)
 *     |
 *     +--> LuminaVideo.init(this)  [one-time, stores applicationContext]
 *     |
 *     +--> GameActivity (androidx.games)
 *             |
 *             +--> libzap_stream_app.so (Rust + egui + wgpu)
 *                     |
 *                     +--> LuminaVideo.createPlayer(nativeHandle) [via JNI]
 *                             |
 *                             +--> ExoPlayerBridge + ExoPlayer [on HandlerThread]
 *                                     |
 *                                     +--> HardwareBuffer (zero-copy frames)
 * ```
 *
 * # Video Playback Flow
 *
 * 1. `LuminaVideo.init(this)` stores the application context
 * 2. Rust's `AndroidVideoDecoder::new()` calls `LuminaVideo.createPlayer()` via JNI
 * 3. `createPlayer()` creates ExoPlayer on a dedicated HandlerThread
 * 4. ExoPlayerBridge receives decoded frames as HardwareBuffers
 * 5. Bridge submits HardwareBuffers to Rust via nativeSubmitHardwareBuffer
 * 6. Rust imports HardwareBuffer into Vulkan via VK_ANDROID_external_memory
 * 7. egui displays the texture with zero CPU copies
 */
package stream.zap.app

import android.os.Bundle
import android.util.Log
import android.view.WindowManager
import android.view.View
import android.view.ViewGroup
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import com.google.androidgamesdk.GameActivity
import com.luminavideo.bridge.LuminaVideo

/**
 * Main activity hosting the lumina-video native application.
 *
 * Extends GameActivity which provides the native window surface
 * and input handling required by egui's android-game-activity feature.
 */
class MainActivity : GameActivity() {

    companion object {
        private const val TAG = "LuminaVideoDemo"

        init {
            try {
                System.loadLibrary("zap_stream_app")
                Log.i(TAG, "Loaded libzap_stream_app.so")
            } catch (e: UnsatisfiedLinkError) {
                Log.e(TAG, "Failed to load libzap_stream_app.so: ${e.message}")
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        LuminaVideo.init(this)
        setupInsets()
        super.onCreate(savedInstanceState)
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
    }

    private fun setupInsets() {
        val content = window.decorView.findViewById<View>(android.R.id.content)
        ViewCompat.setOnApplyWindowInsetsListener(content) { v, windowInsets ->
            val insets = windowInsets.getInsets(WindowInsetsCompat.Type.systemBars())
            val lp = v.layoutParams as ViewGroup.MarginLayoutParams
            lp.topMargin = insets.top
            lp.leftMargin = insets.left
            lp.bottomMargin = insets.bottom
            lp.rightMargin = insets.right
            v.layoutParams = lp
            windowInsets
        }
        WindowCompat.setDecorFitsSystemWindows(window, false)
    }
}
