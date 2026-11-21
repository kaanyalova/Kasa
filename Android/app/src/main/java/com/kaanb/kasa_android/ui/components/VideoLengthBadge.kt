package com.kaanb.kasa_android.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.produceState
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import com.kaanb.kasa_android.ui.util.formatDurationShort
import uniffi.kasa_client.KasaClient

/** Video length badge, same look as the desktop thumbnail overlay (black 60% pill, top-left). */
@Composable
fun VideoLengthBadge(
    client: KasaClient,
    hash: String,
    modifier: Modifier = Modifier,
) {
    val length by produceState<Double?>(initialValue = null, key1 = hash) {
        value = runCatching { client.getVideoLength(hash) }.getOrNull()
    }

    val text = length?.let { formatDurationShort(it) } ?: return

    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        color = Color.White,
        modifier = modifier
            .background(Color.Black.copy(alpha = 0.6f), RoundedCornerShape(4.dp))
            .padding(horizontal = 6.dp, vertical = 2.dp),
    )
}
