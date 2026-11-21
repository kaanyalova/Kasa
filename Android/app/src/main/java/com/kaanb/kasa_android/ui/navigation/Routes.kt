package com.kaanb.kasa_android.ui.navigation

import kotlinx.serialization.Serializable

/** Type-safe navigation routes: https://developer.android.com/guide/navigation/design/type-safety */
@Serializable
data object MediaGrid

@Serializable
data class MediaDetail(val hash: String, val width: Long, val height: Long)
