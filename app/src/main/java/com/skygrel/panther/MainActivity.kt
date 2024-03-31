package com.skygrel.panther

import android.Manifest
import android.app.NativeActivity
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat


class MainActivity : NativeActivity() {
    private val PERMISSION_REQUEST_CODE = 1

    var locationHelper: LocationHelper? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Example of a call to a native method
//        binding.sampleText.text = stringFromJNI()
        this.locationHelper = LocationHelper(this)
        checkAndRequestPermissions()
    }


    private fun checkAndRequestPermissions() {
        // Check if we already have the permission
        if (ContextCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION)
            != PackageManager.PERMISSION_GRANTED
        ) {
            // We don't have permission, so we request it
            ActivityCompat.requestPermissions(
                this, arrayOf(Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION),
                PERMISSION_REQUEST_CODE
            )
        } else {
            // We already have permission
            Log.i("GPS", "Permission already granted!")
//            onPermissionGranted()
        }
    }

//    fun onRequestPermissionsResult(
//        requestCode: Int,
//        permissions: Array<String?>?,
//        grantResults: IntArray
//    ) {
//        super.onRequestPermissionsResult(requestCode, permissions!!, grantResults)
//        if (requestCode == PERMISSION_REQUEST_CODE) {
//            if (grantResults.size > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
//                // Permission granted
//                onPermissionGranted()
//            } else {
//                // Permission denied
//                onPermissionDenied()
//            }
//        }
//    }


    /**
     * A native method that is implemented by the 'panther' native library,
     * which is packaged with this application.
     */
//    external fun stringFromJNI(): String

    companion object {
        // Used to load the 'panther' library on application startup.
        init {
            System.loadLibrary("panther")
        }
    }
}