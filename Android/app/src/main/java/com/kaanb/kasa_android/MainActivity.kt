package com.kaanb.kasa_android

import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.annotation.RequiresApi
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.compose.viewModel
import com.kaanb.kasa_android.jni.NativeClient
import com.kaanb.kasa_android.ui.MediaViewModel
import com.kaanb.kasa_android.ui.navigation.KasaNavHost
import com.kaanb.kasa_android.ui.theme.KasaTheme

class MainActivity : ComponentActivity() {
    @RequiresApi(Build.VERSION_CODES.O)
    override fun onCreate(savedInstanceState: Bundle?) {
        val client = NativeClient(this)

        super.onCreate(savedInstanceState)

        enableEdgeToEdge()
        setContent {
            KasaTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    val mediaViewModel: MediaViewModel =
                        viewModel(factory = MediaViewModel.factory(client))
                    KasaNavHost(
                        viewModel = mediaViewModel,
                        modifier = Modifier.padding(innerPadding),
                    )
                }
            }
        }
    }
}
