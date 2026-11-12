package com.kaanb.kasa_android.ui.navigation

import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionLayout
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.navigation.toRoute
import com.kaanb.kasa_android.ui.MediaViewModel
import com.kaanb.kasa_android.ui.screens.MediaDetailScreen
import com.kaanb.kasa_android.ui.screens.MediaGridScreen

private const val FADE_MS = 250
private const val POP_MS = 200

// The shared element needs a real transition to drive its progress; a fade over a static
// destination is the smallest one that works.
private val detailEnter = fadeIn(tween(FADE_MS))
private val detailExit = fadeOut(tween(FADE_MS))

@OptIn(ExperimentalSharedTransitionApi::class)
@Composable
fun KasaNavHost(
    viewModel: MediaViewModel,
    modifier: Modifier = Modifier,
    navController: NavHostController = rememberNavController(),
) {
    SharedTransitionLayout(modifier = modifier) {
        NavHost(
            navController = navController,
            startDestination = MediaGrid,
        ) {
            // the grid is only the backdrop, it neither moves nor fades
            composable<MediaGrid>(
                enterTransition = { EnterTransition.None },
                exitTransition = { ExitTransition.None },
                popEnterTransition = { EnterTransition.None },
                popExitTransition = { ExitTransition.None },
            ) {
                MediaGridScreen(
                    viewModel = viewModel,
                    onMediaClick = { item ->
                        navController.navigate(
                            MediaDetail(
                                hash = item.hash,
                                width = item.thumbnailX,
                                height = item.thumbnailY,
                            )
                        )
                    },
                    sharedTransitionScope = this@SharedTransitionLayout,
                    animatedVisibilityScope = this,
                )
            }

            // fade only, so the shared element morph is what you actually see. The back gesture is
            // intercepted in MediaDetailScreen, so this transition runs once, on release, instead
            // of being scrubbed by the drag.
            composable<MediaDetail>(
                enterTransition = { detailEnter },
                exitTransition = { detailExit },
                popEnterTransition = { detailEnter },
                // A fade, not None: None means zero duration, which leaves the shared element
                // nothing to interpolate, so the shrink back to the thumbnail is skipped.
                popExitTransition = { fadeOut(tween(POP_MS)) },
            ) { backStackEntry ->
                val route: MediaDetail = backStackEntry.toRoute()
                MediaDetailScreen(
                    viewModel = viewModel,
                    hash = route.hash,
                    mediaWidth = route.width,
                    mediaHeight = route.height,
                    onBack = { navController.popBackStack() },
                    sharedTransitionScope = this@SharedTransitionLayout,
                    animatedVisibilityScope = this,
                )
            }
        }
    }
}
