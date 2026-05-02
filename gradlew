#!/bin/sh
export ANDROID_HOME=/opt/android-sdk
export GRADLE_USER_HOME="$HOME/.gradle"
exec /usr/bin/gradle "$@"
