package com.kaanb.kasa_android.ui.util

/**
 * Mirrors the desktop UI's `formatDurationShort`: "MM:SS", or "HH:MM:SS" once it passes an hour.
 */
fun formatDurationShort(seconds: Double): String {
    val total = seconds.toLong().coerceAtLeast(0)
    val hours = total / 3600
    val minutes = (total % 3600) / 60
    val secs = total % 60

    val short = "%02d:%02d".format(minutes, secs)
    return if (hours > 0) "%02d:%s".format(hours, short) else short
}
