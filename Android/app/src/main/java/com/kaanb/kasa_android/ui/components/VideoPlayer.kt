package com.kaanb.kasa_android.ui.components

import androidx.annotation.OptIn
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.media3.common.MediaItem
import androidx.media3.common.Player as Media3Player
import androidx.media3.common.util.ExperimentalApi
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.ui.compose.SURFACE_TYPE_TEXTURE_VIEW
import androidx.media3.ui.compose.material3.Player

/**
 * Plays [source] (http(s) URL or file path) with ExoPlayer and the Material3 Compose player UI.
 * The player is released when the composable leaves the composition, so audio stops on back.
 */
@OptIn(UnstableApi::class, ExperimentalApi::class)
@Composable
fun VideoPlayer(
    source: String,
    modifier: Modifier = Modifier,
    autoPlay: Boolean = true,
    onRenderedFirstFrame: () -> Unit = {},
) {
    val context = LocalContext.current

    val player = remember(source) { ExoPlayer.Builder(context).build() }

    // media3's compose Player takes showControls as a plain parameter and never toggles it on its
    // own, so the tap handling is up to us
    var showControls by remember { mutableStateOf(false) }

    LaunchedEffect(player, source) {
        player.setMediaItem(MediaItem.fromUri(source))
        player.prepare()
        player.playWhenReady = autoPlay
    }

    DisposableEffect(player) {
        val listener = object : Media3Player.Listener {
            override fun onRenderedFirstFrame() = onRenderedFirstFrame()
        }
        player.addListener(listener)
        onDispose {
            player.removeListener(listener)
            player.release()
        }
    }

    Player(
        player = player,
        modifier = modifier.clickable(
            interactionSource = remember { MutableInteractionSource() },
            indication = null,
        ) { showControls = !showControls },
        contentScale = ContentScale.Fit,
        // TextureView instead of SurfaceView: a SurfaceView is a separate window layer, so Compose
        // cannot clip/transform it and shared element transitions would not animate at all
        surfaceType = SURFACE_TYPE_TEXTURE_VIEW,
        // PlayerDefaults.BottomControls = position text + progress slider + duration text
        showControls = showControls,
    )
}
