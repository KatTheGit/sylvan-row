# compile
./android.sh

# compile the java
cd android
ANDROID_HOME=~/Android/Sdk/ ./gradlew build
cd ..

# run it on a phone
~/Android/Sdk/platform-tools/adb install "$PWD/android/app/build/outputs/apk/debug/app-debug.apk"
~/Android/Sdk/platform-tools/adb shell am start -n org.bevyengine.example/org.bevyengine.example.MainActivity
