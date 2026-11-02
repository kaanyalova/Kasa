# build the android libraries and generate the kotlin bindings
# requires you to have cargo-ndk from git because of some ffmpeg issues


rm -rf Android/app/src/main/jniLibs
cargo ndk -t arm64-v8a -t x86_64 -o jniLibs_temp build --release -p kasa_client
mv jniLibs_temp/ Android/app/src/main/jniLibs

rm -rf Android/app/src/main/java/com/kaanb/kasa_android/jni/bindings/*
cargo run --bin uniffi_bindgen generate --library ./Android/app/src/main/jniLibs/x86_64/libkasa_client.so --language kotlin --out-dir ./Android/app/src/main/java/com/kaanb/kasa_android/jni/bindings/
