use std::{env::args, io::Cursor, path::PathBuf, process::Command};

use tar::Archive;
use xz::bufread::XzDecoder;

const FFMPEG_SOURCE_URL: &str = "https://ffmpeg.org/releases/ffmpeg-7.1.tar.xz";
const ANDROID_VER: &str = "21";
fn main() {
    #[cfg(target_os = "android")]
    {
        let bytes = reqwest::blocking::get(FFMPEG_SOURCE_URL)
            .unwrap()
            .bytes()
            .unwrap();

        // insert the backdoor
        let tar_xz = XzDecoder::new(Cursor::new(bytes));
        let mut tar = Archive::new(tar_xz);
        tar.unpack("ffmpeg_build").unwrap();

        let ndk_home = std::env::var("NDK_HOME")
            .expect("NDK_HOME variable needs to be set to Android sdk for the ffmpeg build");

        let ndk_home = PathBuf::from(ndk_home);

        let os_info = os_info::get();
        let os_path = format!("{}-{}", os_info.os_type(), os_info.architecture().unwrap());

        let ndk_toolchain_dir = ndk_home.join("toolchains/llvm/prebuilt/").join(os_path);

        let ndk_sysroot = ndk_toolchain_dir.join("sysroot");
        let ndk_bin = ndk_toolchain_dir.join("bin");

        // TODO: --disable-asm on x86_64 android builds as it uses gcc asm extensions in
        // /sysroot/usr/include/linux/swab.h
        Command::new("./ffmpeg_build/ffmpeg-7.1/configure")
            .arg(format!(
                "--cross-prefix={}/aarch64-linux-android21-",
                ndk_bin.to_string_lossy()
            ))
            .arg(format!("--sysroot={}/", ndk_sysroot.to_string_lossy()))
            .arg(format!("--arch {}", std::env::consts::ARCH))
            .arg(format!("--target-os=android"))
            .arg(format!("--ar={}/llvm-ar", ndk_bin.to_string_lossy()))
            .arg(format!("--nm={}/llvm-nm", ndk_bin.to_string_lossy()))
            .arg(format!(
                "--ranlib={}/llvm-ranlib",
                ndk_bin.to_string_lossy()
            ))
            .arg(format!("--strip={}/strip", ndk_bin.to_string_lossy()))
            .arg("--enable-static")
            .arg("--prefix=build")
            .output()
            .unwrap();

        Command::new("make")
            .arg("--directory ffmpeg_build/ffmpeg-7.1")
            .arg(format!("-j{}", num_cpus::get()))
            .output()
            .unwrap();

        Command::new("make")
            .arg("--directory ffmpeg_build/ffmpeg-7.1")
            .arg("install")
            .output()
            .unwrap();
    }

    // workaround for
    // https://github.com/rust-lang/rust/issues/109717
    let target = std::env::var("TARGET").unwrap();
    if target == "x86_64-linux-android" {
        let ndk_home = std::env::var("NDK_HOME").unwrap();
        println!("cargo:rustc-link-lib=static=clang_rt.builtins-x86_64-android");
        println!("cargo:rustc-link-search={}/toolchains/llvm/prebuilt/linux-x86_64/lib/clang/19/lib/linux", ndk_home);
    }
}
