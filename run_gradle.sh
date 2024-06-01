export ANDROID_NDK_HOME="/home/skygrel19/Android/Sdk/ndk"
export ANDROID_HOME="/home/skygrel19/Android/Sdk"


cargo ndk -t arm64-v8a -o app/src/main/jniLibs/ --platform 34 build
cargo ndk -t x86 -o app/src/main/jniLibs/ --platform 34 build
cargo ndk -t x86_64 -o app/src/main/jniLibs/ --platform 34 build
./gradlew build

./gradlew installDebug
adb shell am start -n com.skygrel.panther/.MainActivity