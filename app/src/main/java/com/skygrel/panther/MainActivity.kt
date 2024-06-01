package com.skygrel.panther

import android.Manifest
import android.app.NativeActivity
import android.content.Context
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat


class MainActivity : NativeActivity(), ActivityCompat.OnRequestPermissionsResultCallback {
    private val PERMISSION_REQUEST_CODE = 1

    var locationHelper: LocationHelper? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)



        this.locationHelper = LocationHelper(this)
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
            locationHelper?.onPermissionGranted()
            locationHelper?.startLocationUpdates();
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        if (requestCode == PERMISSION_REQUEST_CODE) {
            if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                // Permission granted
                Log.i("GPS", "Permission granted!")
                locationHelper?.onPermissionGranted()
                locationHelper?.startLocationUpdates();
            } else {
                // Permission denied
                Log.i("GPS", "Permission denied!")
                locationHelper?.onPermissionDenied();
            }
        }
    }


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