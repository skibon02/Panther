package com.skygrel.panther;

import static androidx.activity.result.ActivityResultCallerKt.registerForActivityResult;

import android.app.Activity;
import android.content.Context;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;


public class LocationHelper {
    private LocationManager locationManager;
    private LocationListener locationListener;

    public LocationHelper(Activity activity) {
        locationManager = (LocationManager) activity.getSystemService(Context.LOCATION_SERVICE);
        locationListener = new LocationListener() {
            @Override
            public void onLocationChanged(Location location) {
                // Call a static native method without nativePtr
                onLocationUpdate(location.getLatitude(), location.getLongitude());
                Log.i("GPS", "Location info: ACC: " + location.getAccuracy() + ", altitude: " +
                        location.getAltitude() + " +-" + location.getVerticalAccuracyMeters() +
                        "\nTime: " + location.getElapsedRealtimeNanos() / 1000000000 +
                        "\nExtras: " + location.getExtras() +
                        "\nSpeed: " + location.getSpeed() + " +-" + location.getSpeedAccuracyMetersPerSecond());
            }

            @Override
            public void onProviderEnabled(String provider) {
                Log.i("GPS", "GPS Provider enabled: " + provider);
            }

            @Override
            public void onProviderDisabled(String provider) {
                Log.i("GPS", "GPS Provider disabled: " + provider);
            }

            @Override
            public void onFlushComplete(int requestCode) {
                Log.i("GPS", "GPS onFlushComplete. Code: " + requestCode);
            }
        };
    }


    public void startLocationUpdates() {
        new Handler(Looper.getMainLooper()).post(new Runnable() {
            @Override
            public void run() {
                try {
                    locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER, 0, 0, locationListener);
                    Log.i("GPS", "requestLocationUpdates call success!");
                } catch (SecurityException e) {
                    Log.e("GPS", e.toString());
                    // Handle permission issue
                }
            }
        });
    }

    // Stop requesting location updates
    public void stopLocationUpdates() {
        locationManager.removeUpdates(locationListener);
    }

    // Modified to not use nativePtr
    private native void onLocationUpdate(double latitude, double longitude);
}