package com.kaanb.kasa_android.ui.components

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.staggeredgrid.LazyVerticalStaggeredGrid
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridCells
import androidx.compose.foundation.lazy.staggeredgrid.items
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import com.kaanb.kasa_android.ui.MediaViewModel
import uniffi.kasa_core.Media

/** Stateless thumbnail grid: [media] in, clicks out. */
@OptIn(ExperimentalSharedTransitionApi::class)
@Composable
fun MediaGrid(
    viewModel: MediaViewModel,
    media: List<Media>,
    modifier: Modifier = Modifier,
    onMediaClick: (Media) -> Unit = {},
    sharedTransitionScope: SharedTransitionScope,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    LazyVerticalStaggeredGrid(
        columns = StaggeredGridCells.Fixed(3),
        modifier = modifier,
        verticalItemSpacing = 4.dp,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        items(media, key = { it.hash }) { item ->
            val bytes: State<ByteArray?> = produceState<ByteArray?>(initialValue = null, key1 = item.hash) {
                value = viewModel.client.getThumbnail(item.hash)
            }

            val context = LocalContext.current
            val request = remember(bytes.value, item.hash) {
                ImageRequest.Builder(context)
                    // Coil bails out with NullRequestDataException *before* it looks at the memory
                    // cache, so the data must never be null or the cached thumbnail can't be
                    // reused while the bytes are being refetched. The hash is a dummy that only
                    // matters on a cache miss (where it fails, which is the same as blank).
                    .data(bytes.value ?: item.hash)
                    .memoryCacheKey(item.hash)
                    .placeholderMemoryCacheKey(item.hash)
                    .build()
            }

            Box {
                AsyncImage(
                    model = request,
                    contentDescription = null,
                    modifier = with(sharedTransitionScope) {
                        Modifier
                            .sharedElement(
                                rememberSharedContentState(key = item.hash),
                                animatedVisibilityScope = animatedVisibilityScope,
                            )
                            .fillMaxWidth()
                            .clickable { onMediaClick(item) }
                            // known thumbnail size -> correct cell shape before the decode finishes
                            .aspectRatio(
                                if (item.thumbnailY > 0) {
                                    item.thumbnailX.toFloat() / item.thumbnailY.toFloat()
                                } else {
                                    1f
                                }
                            )
                    },
                    contentScale = ContentScale.Crop,
                )

                if (item.mediaType == "Video") {
                    VideoLengthBadge(
                        client = viewModel.client,
                        hash = item.hash,
                        modifier = Modifier.align(Alignment.TopStart).padding(8.dp),
                    )
                }
            }
        }
    }
}
