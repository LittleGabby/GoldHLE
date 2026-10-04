/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 *
 * Parts of this file are derived from SDL 2's Android project template, which
 * has a different license. Please see vendor/SDL/LICENSE.txt for details.
 */
package org.touchhle.android;

import android.app.Activity;
import android.content.ContentResolver;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.provider.OpenableColumns;
import android.util.Log;

import org.libsdl.app.SDLActivity;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * A wrapper class over SDLActivity
 */
public class MainActivity extends SDLActivity {
    /**
     * The request code for the "add game" system file picker. This must be
     * non-negative and may be any value not used for other request types.
     */
    private static final int ADD_GAME_REQUEST = 1;

    /**
     * A reference to the activity instance, so that the static method called
     * from Rust can reach the activity.
     */
    private static MainActivity instance = null;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        instance = this;
    }

    @Override
    protected void onDestroy() {
        if (instance == this) {
            instance = null;
        }
        super.onDestroy();
    }

    /**
     * Called from Rust (via JNI) when the user taps the "Add game" button in
     * the app picker. Launches the system file picker on the UI thread.
     *
     * Returns true if the picker was started, or false if the activity wasn't
     * ready or starting it threw an exception.
     */
    public static boolean addGamePicker() {
        final Activity activity = instance;
        if (activity == null) {
            Log.e("touchHLE", "Add game: activity is null");
            return false;
        }
        final boolean[] started = new boolean[]{false};
        final Throwable[] failure = new Throwable[]{null};
        final Object lock = new Object();
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                try {
                    Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
                    intent.addCategory(Intent.CATEGORY_OPENABLE);
                    intent.setType("*/*");
                    activity.startActivityForResult(intent, ADD_GAME_REQUEST);
                    started[0] = true;
                } catch (Throwable t) {
                    Log.e("touchHLE", "Add game: couldn't start the file picker", t);
                    failure[0] = t;
                } finally {
                    synchronized (lock) {
                        lock.notifyAll();
                    }
                }
            }
        });
        try {
            synchronized (lock) {
                // Wait a bounded amount of time for the UI thread to run the
                // above. If it times out, something is badly wrong.
                lock.wait(5000);
            }
        } catch (InterruptedException e) {
            Log.e("touchHLE", "Add game: interrupted while waiting for the picker", e);
            return false;
        }
        if (failure[0] != null) {
            Log.e("touchHLE", "Add game: picker failed", failure[0]);
            return false;
        }
        return started[0];
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (requestCode != ADD_GAME_REQUEST || resultCode != RESULT_OK || data == null) {
            return;
        }
        final Uri uri = data.getData();
        if (uri == null) {
            return;
        }
        // Copy the file in the background, so that the UI thread isn't
        // blocked while it happens.
        new Thread(new Runnable() {
            @Override
            public void run() {
                addGameCopy(uri);
            }
        }).start();
    }

    /**
     * Copy the file picked via the "Add game" file picker into the apps
     * directory, then leave a marker file so that the running app picker
     * notices it and adds it to the app list without a restart.
     */
    private void addGameCopy(Uri uri) {
        try {
            String name = getDisplayName(uri);
            // The app picker only lists files ending in ".ipa", so make sure
            // the copy does too.
            if (name == null) {
                name = "game.ipa";
            } else if (!name.toLowerCase().endsWith(".ipa")) {
                name = name + ".ipa";
            }

            File appsDir = new File(getExternalFilesDir(null), "touchHLE_apps");
            appsDir.mkdirs();
            File outFile = new File(appsDir, name);

            ContentResolver contentResolver = getContentResolver();
            InputStream inputStream = contentResolver.openInputStream(uri);
            if (inputStream == null) {
                Log.e("touchHLE", "Couldn't open the picked file");
                return;
            }
            OutputStream outputStream = new FileOutputStream(outFile);
            byte[] buffer = new byte[64 * 1024];
            int read;
            while ((read = inputStream.read(buffer)) != -1) {
                outputStream.write(buffer, 0, read);
            }
            outputStream.flush();
            outputStream.close();
            inputStream.close();

            Log.i("touchHLE", "Added game: " + outFile.getPath());
            // Leave a marker file so that the running app picker notices the
            // new game and shows it without needing a restart.
            try {
                new File(getExternalFilesDir(null), ".touchHLE_import_done")
                        .createNewFile();
            } catch (Exception e) {
                Log.e("touchHLE", "Couldn't write the import marker", e);
            }
        } catch (Exception e) {
            Log.e("touchHLE", "Couldn't add the picked game", e);
        }
    }

    /**
     * Get the display name of the picked file, using the content resolver's
     * query API.
     */
    private String getDisplayName(Uri uri) {
        try {
            Cursor cursor = getContentResolver().query(uri, null, null, null, null);
            if (cursor == null) {
                return null;
            }
            int nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME);
            String name = null;
            if (nameIndex != -1 && cursor.moveToFirst()) {
                name = cursor.getString(nameIndex);
            }
            cursor.close();
            return name;
        } catch (Exception e) {
            Log.e("touchHLE", "Couldn't query the picked file's name", e);
            return null;
        }
    }

    @Override
    protected String[] getLibraries() {
        return new String[]{
            "SDL2",
            "touchHLE"
        };
    }
}
