use jni::{
    objects::{JObject, JValue, JValueOwned},
    JNIEnv,
};
use std::os::fd::FromRawFd;
use wry::webview::JniHandle;

pub(crate) fn get_file<F>(jni_handle: JniHandle, uri: String, mode: &'static str, f: F)
where
    F: Fn(jni::errors::Result<std::fs::File>) + Send + 'static,
{
    jni_handle.exec(move |env: &mut JNIEnv, activity: &JObject, _web_view: &JObject| {
        let file = (|| {
            let mode_string = env.new_string(mode)?;
            let uri_obj = env.new_string(uri)
                .and_then(|s| env.call_static_method(
                    "android/net/Uri",
                    "parse",
                    "(Ljava/lang/String;)Landroid/net/Uri;",
                    &[(&s).into()],
                ))?;

            env.call_method(
                activity,
                "getContentResolver",
                "()Landroid/content/ContentResolver;",
                &[]
            )
                .and_method(
                    env,
                    "openAssetFileDescriptor",
                    "(Landroid/net/Uri;Ljava/lang/String;)Landroid/content/res/AssetFileDescriptor;",
                    &[(&uri_obj).into(), (&mode_string).into()]
                )
                .and_method(env, "getParcelFileDescriptor", "()Landroid/os/ParcelFileDescriptor;", &[])
                .and_method(env, "detachFd", "()I", &[])?
                .i()
        })()
            // Safety: This file descriptor comes directly from Java, from the detachFd function,
            // meaning that it should be open, and the responsibility to close it has now been
            // handed over exclusively to this File object.
            .map(|fd| unsafe { std::fs::File::from_raw_fd(fd) });
        f(file)
    });
}

// pub(crate) fn open_save_dialog(jni_handle: JniHandle) {
//     // let (tx, rx) = std::sync::mpsc::channel();
//     jni_handle.exec(move |env: &mut JNIEnv, activity: &JObject, web_view: &JObject| {
//         let result: jni::errors::Result<_> = (|| {
//             let intent = env.new_object(
//                 "android/content/Intent",
//                 "(Landroid/content/Intent;)V",
//                 &[(&get_intent_string_field(env, "ACTION_CREATE_DOCUMENT")?).into()]
//             )?;

//             env.call_method(
//                 intent,
//                 "addCategory",
//                 "(Ljava/lang/String;)Landroid/content/Intent;",
//                 &[(&get_intent_string_field(env, "CONTENT_OPENABLE")?).into()]
//             )?;

//             todo!()
//         })();
//         todo!()
//     });
// }

// fn get_intent_string_field<'a>(env: &mut JNIEnv<'a>, name: &str) -> jni::errors::Result<JValueOwned<'a>> {
//     env.get_static_field("android/content/Intent", name, "Ljava/lang/String;")
// }

trait MethodExt {
    fn and_method<'a>(
        self,
        env: &mut JNIEnv<'a>,
        name: &str,
        sig: &str,
        args: &[JValue],
    ) -> jni::errors::Result<JValueOwned<'a>>;
}

impl<'a> MethodExt for jni::errors::Result<JValueOwned<'a>> {
    fn and_method<'b>(
        self,
        env: &mut JNIEnv<'b>,
        name: &str,
        sig: &str,
        args: &[JValue],
    ) -> jni::errors::Result<JValueOwned<'b>> {
        self.and_then(|obj| env.call_method(obj.l().unwrap(), name, sig, args))
    }
}
