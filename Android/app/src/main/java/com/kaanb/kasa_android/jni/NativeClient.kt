package com.kaanb.kasa_android.jni

import android.content.Context
import android.os.Build
import androidx.annotation.RequiresApi
import uniffi.kasa_client.KasaClient
import uniffi.kasa_client.loadConfigFromRaw
import uniffi.kasa_client.newFromConfig
import uniffi.kasa_core.DateOrderCriteria
import uniffi.kasa_core.GlobalConfig
import uniffi.kasa_core.Media
import uniffi.kasa_core.ResolutionOrderCriteria
import uniffi.kasa_core.SearchCriteria
import java.nio.file.Files
import java.nio.file.Paths

class NativeClient {
    public var inner: KasaClient
    public var config: GlobalConfig
    external fun javaInit();

    @RequiresApi(Build.VERSION_CODES.O)
    constructor(context: Context) {
        System.loadLibrary("kasa_client")
        javaInit()

        var dataDir = context.dataDir;
        config = loadConfigFromRaw("""
            # Try to avoid using relative paths, they will cause problems, they should never be configured
            # from the GUI anyways

            [Database]
            # Path of the currently open database file
            db_path = "http://wetware-mainframe:8123"
            db_type = "remote"


            [Thumbnails]
            # Path of the db that stores the thumbnails
            thumbs_db_path = "${dataDir}/thumbs.db"

            # The max resolution for thumbnails, [width, height]
            resolution = [0, 0]

            # The file format for thumbnails
            thumbnail_format = "webp_lossy"


            [Downloader]
            # Path that gallery_dl will output the extracted media
            output_path = "${dataDir}/downloads/"

            # Optional: gallery_dl config path 
            gdl_config_path = ""


            [Layout]
            thumbnail_scale = 0.9
            show_filenames = false
        """.trimIndent())

        var downloadsDir = Paths.get("${dataDir}/downloads")

        if (!Files.exists(downloadsDir)) {
            Files.createDirectory(Paths.get("${dataDir}/downloads"))
        }


        inner = newFromConfig(config)
    }

    public suspend fun testSearch(): List<Media> {
        var criteria = SearchCriteria(
            containsTags = listOf(),
            containsTagsOrGroup = listOf(),
            excludesTags = listOf(),
            orderByDate = DateOrderCriteria.NEWEST_FIRST,
            orderByResolution = ResolutionOrderCriteria.NONE,
            dateRange = null,
            favoritesOnly = false
        )

        return inner.search(criteria)
    }

}