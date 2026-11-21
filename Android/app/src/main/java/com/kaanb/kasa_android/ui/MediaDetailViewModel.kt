package com.kaanb.kasa_android.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import kotlinx.coroutines.launch
import uniffi.kasa_client.KasaClient
import uniffi.kasa_core.DatabaseType
import uniffi.kasa_core.GlobalConfig
import uniffi.kasa_core.MediaInfo

/**
 * Per-destination state for one media item. Scoped to the detail screen's back stack entry, so it
 * is created when you navigate and discarded when you pop, and the loaded values survive
 * recomposition/rotation instead of being re-requested by produceState.
 */
class MediaDetailViewModel(
    private val client: KasaClient,
    private val config: GlobalConfig,
    private val hash: String,
) : ViewModel() {

    var source: String? by mutableStateOf(null)
        private set

    var mediaType: String? by mutableStateOf(null)
        private set

    var thumbnail: ByteArray? by mutableStateOf(null)
        private set

    var info: MediaInfo? by mutableStateOf(null)
        private set

    var error: String? by mutableStateOf(null)
        private set

    init {
        viewModelScope.launch {
            source = when (config.db.dbType) {
                DatabaseType.LOCAL -> runCatching { client.getValidPath(hash) }.getOrNull()
                DatabaseType.REMOTE -> "${config.db.dbPath}/media?hash=$hash"
                else -> null
            }?.let { raw ->
                // ExoPlayer and Coil both want a scheme
                if (raw.contains("://")) raw else "file://$raw"
            }

            mediaType = runCatching { client.getMediaType(hash) }.getOrNull()
            info = runCatching { client.getInfo(hash) }.getOrNull()
            thumbnail = runCatching { client.getThumbnail(hash) }.getOrNull()
        }
    }

    companion object {
        fun factory(client: KasaClient, config: GlobalConfig, hash: String) = viewModelFactory {
            initializer { MediaDetailViewModel(client, config, hash) }
        }
    }
}
