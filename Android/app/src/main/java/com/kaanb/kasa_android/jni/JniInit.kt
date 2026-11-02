package com.kaanb.kasa_android.jni

import uniffi.kasa_client.KasaClient
import uniffi.kasa_client.newRemote
import uniffi.kasa_core.GlobalConfig
import uniffi.kasa_client.

class JniInit {
    external fun javaInit();

    constructor() {
        System.loadLibrary("kasa_client")
        javaInit()

        var config = loadConfigFromRaw("""
            
        """.trimIndent())

    }

}