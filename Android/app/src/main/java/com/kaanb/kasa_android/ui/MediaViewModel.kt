package com.kaanb.kasa_android.ui

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.kaanb.kasa_android.jni.NativeClient
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import uniffi.kasa_client.KasaClient
import uniffi.kasa_core.DateOrderCriteria
import uniffi.kasa_core.Media
import uniffi.kasa_core.ResolutionOrderCriteria
import uniffi.kasa_core.SearchCriteria

/**
 * Owns the search result so it survives navigation and configuration changes. Everything else is
 * reachable directly through [client] - no wrapper calls here.
 */
class MediaViewModel(nativeClient: NativeClient) : ViewModel() {

    val client: KasaClient = nativeClient.inner
    val config = nativeClient.config

    var media: List<Media> by mutableStateOf(emptyList())
        private set

    var isLoading: Boolean by mutableStateOf(false)
        private set

    var error: String? by mutableStateOf(null)
        private set

    init {
        reload()
    }

    fun reload() {
        if (isLoading) return
        viewModelScope.launch {
            isLoading = true
            error = null

            // the rust client initializes on its own thread and reports InvalidDb until it is
            // ready, so the first query can legitimately fail for a few hundred milliseconds
            var last: Throwable? = null
            repeat(10) {
                try {
                    media = client.search(
                        SearchCriteria(
                            containsTags = emptyList(),
                            containsTagsOrGroup = emptyList(),
                            excludesTags = emptyList(),
                            orderByDate = DateOrderCriteria.NEWEST_FIRST,
                            orderByResolution = ResolutionOrderCriteria.NONE,
                            dateRange = null,
                            favoritesOnly = false,
                        )
                    )
                    isLoading = false
                    return@launch
                } catch (t: Throwable) {
                    last = t
                    delay(300)
                }
            }

            error = last?.message
            isLoading = false
        }
    }

    companion object {
        fun factory(nativeClient: NativeClient) = viewModelFactory {
            initializer { MediaViewModel(nativeClient) }
        }
    }
}
