package com.kaanb.kasa_android.ui.screens

import androidx.activity.compose.PredictiveBackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import coil3.SingletonImageLoader
import coil3.compose.AsyncImage
import coil3.compose.AsyncImagePainter
import coil3.compose.rememberAsyncImagePainter
import coil3.request.ImageRequest
import com.kaanb.kasa_android.ui.MediaDetailViewModel
import com.kaanb.kasa_android.ui.MediaViewModel
import com.kaanb.kasa_android.ui.components.VideoPlayer
import uniffi.kasa_core.DatabaseType
import uniffi.kasa_core.Media
import uniffi.kasa_core.MediaInfo

@OptIn(ExperimentalMaterial3Api::class, ExperimentalSharedTransitionApi::class)
@Composable
fun MediaDetailScreen(
    viewModel: MediaViewModel,
    hash: String,
    mediaWidth: Long,
    mediaHeight: Long,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    sharedTransitionScope: SharedTransitionScope,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    // The back gesture must not scrub a transition: swallow the progress, pop only on commit.
    // The pop itself still animates, so the shared element shrinks back into its grid cell.
    PredictiveBackHandler { progress ->
        progress.collect { }
        onBack()
    }

    val ratio =
        if (mediaWidth > 0 && mediaHeight > 0) {
            mediaWidth.toFloat() / mediaHeight.toFloat()
        } else {
            1f
        }

    val detail: MediaDetailViewModel = viewModel(
        factory = MediaDetailViewModel.factory(viewModel.client, viewModel.config, hash),
    )

    val source = detail.source
    val mediaType = detail.mediaType
    val info = detail.info
    // the grid's thumbnail, shown inside the growing shared element until the video has a frame
    val thumbnail = detail.thumbnail

    var firstFrameRendered by remember(hash) { mutableStateOf(false) }

    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = { Text(info?.fileName.orEmpty()) },
                navigationIcon = {
                    TextButton(onClick = onBack) { Text("Back") }
                },
            )
        },
    ) { innerPadding ->
        Box(
            modifier = Modifier.fillMaxSize().padding(innerPadding),
            contentAlignment = Alignment.Center,
        ) {
            val sharedModifier = with(sharedTransitionScope) {
                Modifier
                    .sharedElement(
                        rememberSharedContentState(key = hash),
                        animatedVisibilityScope = animatedVisibilityScope,
                    )
                    .fillMaxWidth()
                    .aspectRatio(ratio)
            }

            when {
                mediaType == "Video"-> {
                    Box(modifier = sharedModifier) {
                        VideoPlayer(
                            source = source!!,
                            modifier = Modifier.matchParentSize(),
                            onRenderedFirstFrame = { firstFrameRendered = true },
                        )

                        // the thumbnail travels with the shared element, then crossfades into
                        // the first rendered video frame
                        AnimatedVisibility(
                            visible = !firstFrameRendered,
                            enter = fadeIn(tween(120)),
                            exit = fadeOut(tween(220)),
                            modifier = Modifier.matchParentSize(),
                        ) {
                            AsyncImage(
                                model = thumbnail,
                                contentDescription = null,
                                modifier = Modifier.matchParentSize(),
                                contentScale = ContentScale.Crop,
                            )
                        }
                    }
                }

                mediaType == "Image" -> {
                    val fullPainter = rememberAsyncImagePainter(
                        model = ImageRequest.Builder(LocalContext.current)
                            .data(source)
                            .build(),
                        imageLoader = SingletonImageLoader.get(LocalContext.current),
                        contentScale = ContentScale.Crop,
                        filterQuality = FilterQuality.High,
                    )
                    val fullState by fullPainter.state.collectAsState()
                    val fullAlpha by animateFloatAsState(
                        targetValue = if (fullState is AsyncImagePainter.State.Success) 1f else 0f,
                        animationSpec = tween(180),
                        label = "fullImageAlpha",
                    )

                    // This draws the thumbnail on top of the actual image which hasn't loaded yet
                    // then whenever the image loads it fades to the full size image
                    Box(modifier = sharedModifier) {
                        AsyncImage(
                            model = ImageRequest.Builder(LocalContext.current)
                                .data(thumbnail ?: hash)
                                .memoryCacheKey(hash)
                                .placeholderMemoryCacheKey(hash)
                                .build(),
                            contentDescription = null,
                            modifier = Modifier.matchParentSize(),
                            contentScale = ContentScale.Crop,
                        )

                        Image(
                            painter = fullPainter,
                            contentDescription = null,
                            modifier = Modifier.matchParentSize().alpha(fullAlpha),
                            contentScale = ContentScale.Crop,
                        )
                    }
                }
            }
        }
    }
}
