import android.net.Uri;
import androidx.ActivityResultCallback;

class DialogCallback implements ActivityResultCallback<Uri> {
    @Override
    public void onActivityResult(Uri uri) {
        saveDialogUri(uri.toString());
    }

    private native void saveDialogUri(String uri);
}