package com.kaanb.kasa_android.ui.screens

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import com.kaanb.kasa_android.ui.MediaViewModel
import com.kaanb.kasa_android.ui.components.MediaGrid
import uniffi.kasa_core.Media

/** Thin screen wrapper: loading state from the view model, grid for the result. */
@OptIn(ExperimentalSharedTransitionApi::class)
@Composable
fun MediaGridScreen(
    viewModel: MediaViewModel,
    modifier: Modifier = Modifier,
    onMediaClick: (Media) -> Unit = {},
    sharedTransitionScope: SharedTransitionScope,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    if (viewModel.isLoading && viewModel.media.isEmpty()) {
        Box(modifier = modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        return
    }

    MediaGrid(
        viewModel = viewModel,
        media = viewModel.media,
        modifier = modifier,
        onMediaClick = onMediaClick,
        sharedTransitionScope = sharedTransitionScope,
        animatedVisibilityScope = animatedVisibilityScope,
    )
}
