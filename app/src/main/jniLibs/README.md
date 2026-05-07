# PRoot Binaries for Android

This directory contains architecture-specific PRoot binaries for Android.

## Directory Structure

```
app/src/main/jniLibs/
├── arm64-v8a/      # ARM 64-bit (most modern Android devices)
│   └── libproot.so
├── armeabi-v7a/    # ARM 32-bit (older devices)
│   └── libproot.so
└── x86_64/         # x86 64-bit (emulators, some tablets)
    └── libproot.so
```

## Current Status

The `libproot.so` files in each architecture directory are **placeholders**. You need to replace them with actual PRoot binaries.

## How to Obtain PRoot Binaries

### Option 1: Download Pre-built Binaries

The Termux project provides pre-built PRoot binaries for Android:

- **Termux PRoot Repository**: https://github.com/termux/proot
- **Termux Packages**: Check the Termux package repository for pre-built binaries

#### Direct Download URLs (if available):
```bash
# ARM64 (aarch64)
wget https://github.com/termux/proot/releases/download/<VERSION>/proot-aarch64 -O arm64-v8a/libproot.so

# ARMv7 (arm)
wget https://github.com/termux/proot/releases/download/<VERSION>/proot-arm -O armeabi-v7a/libproot.so

# x86_64
wget https://github.com/termux/proot/releases/download/<VERSION>/proot-x86_64 -O x86_64/libproot.so
```

Replace `<VERSION>` with the latest release tag from the Termux PRoot repository.

### Option 2: Build from Source

1. Clone the Termux PRoot repository:
   ```bash
   git clone https://github.com/termux/proot.git
   cd proot
   ```

2. Build for Android using the Android NDK:
   ```bash
   # Set up NDK environment
   export NDK_HOME=/path/to/android-ndk
   export TOOLCHAIN=$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64
   
   # Build for ARM64
   make CROSS_COMPILE=aarch64-linux-android- ANDROID_API=21
   
   # Build for ARMv7
   make CROSS_COMPILE=arm-linux-androideabi- ANDROID_API=16
   
   # Build for x86_64
   make CROSS_COMPILE=x86_64-linux-android- ANDROID_API=21
   ```

3. Copy the resulting binaries to the appropriate directories.

### Option 3: Use Termux Environment

If you have Termux installed on an Android device, you can copy the binaries from:
- `/data/data/com.termux/files/usr/libexec/proot`

## File Permissions

After placing the binaries, ensure they have execute permissions:

```bash
chmod +x arm64-v8a/libproot.so
chmod +x armeabi-v7a/libproot.so
chmod +x x86_64/libproot.so
```

## Verification

To verify the binaries are working:

```bash
# For each architecture
file arm64-v8a/libproot.so    # Should show "ELF 64-bit LSB shared object, ARM aarch64"
file armeabi-v7a/libproot.so  # Should show "ELF 32-bit LSB shared object, ARM"
file x86_64/libproot.so       # Should show "ELF 64-bit LSB shared object, x86-64"
```

## Notes

- PRoot is used to provide a Linux-like environment on Android
- The binaries must match the target device architecture
- For production use, consider using the official Termux PRoot builds
- These binaries are typically used with Android's JNI (Java Native Interface)

## References

- Termux PRoot: https://github.com/termux/proot
- PRoot Official: https://proot-me.github.io/
- Android NDK: https://developer.android.com/ndk
