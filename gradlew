#!/bin/sh
export ANDROID_HOME=/opt/android-sdk
export GRADLE_USER_HOME="$HOME/.gradle"
exec /root/.gradle/wrapper/dists/gradle-9.2.0-bin/11i5gvueggl8a5cioxuftxrik/gradle-9.2.0/bin/gradle "$@"
