use std::{
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
};

use fs2::FileExt;
use sha2::{Digest, Sha256};

const SDK_VERSION: &str = "5.3.0";

struct SdkAsset {
    archive: &'static str,
    sha256: &'static str,
}

fn sdk_asset(target: &str) -> SdkAsset {
    match target {
        "x86_64-unknown-linux-gnu" => SdkAsset {
            archive: "mindrovesdk-5.3.0-linux-x64.zip",
            sha256: "73b0c916a6488074d53c370d1c61b841af158906bd8fcd864150f87e5155169d",
        },

        "aarch64-unknown-linux-gnu" => SdkAsset {
            archive: "mindrovesdk-5.3.0-linux-aarch64.zip",
            sha256: "4f1749103dfde726a2a788b3c6a2bdeec4867083508eef642f91e647130f970c",
        },

        "armv7-unknown-linux-gnueabihf" | "arm-unknown-linux-gnueabihf" => SdkAsset {
            archive: "mindrovesdk-5.3.0-linux-arm32.zip",
            sha256: "25dff711f3301eb13b06f438fecc356b8e5171705193f576b261ec6fdc496e29",
        },

        "x86_64-pc-windows-msvc" => SdkAsset {
            archive: "mindrovesdk-5.3.0-win-x64.zip",
            sha256: "2690d5697b535edcf33d8a5d178f87cfd6064f144a0360e9fc9d34b856851195",
        },

        "i686-pc-windows-msvc" => SdkAsset {
            archive: "mindrovesdk-5.3.0-win-x86.zip",
            sha256: "ff871593cc3f3c21edaa9dd867f45e4dfd5b16ce3710c49732e6ae92df2b2298",
        },

        "x86_64-apple-darwin" | "aarch64-apple-darwin" => SdkAsset {
            archive: "mindrovesdk-5.3.0-macos.zip",
            sha256: "0639ea99bd9104a89c454de19c823a58fbf567de2f023aca3b650abc4adbe217",
        },

        other => panic!("MindRove SDK does not support target {other}"),
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=MINDROVE_SDK_DIR");
    println!("cargo:rerun-if-changed=wrapper.h");

    let target = env::var("TARGET").expect("TARGET not set by Cargo");
    let host = env::var("HOST").expect("HOST not set by Cargo");

    let asset = sdk_asset(&target);

    let sdk_path = env::var_os("MINDROVE_SDK_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| get_sdk(&target, &host, &asset));

    let libdir_path = sdk_path
        .join("lib")
        .canonicalize()
        .expect("cannot canonicalize SDK lib path");

    let include_path = sdk_path
        .join("inc")
        .canonicalize()
        .expect("cannot canonicalize SDK include path");

    println!("cargo:rustc-link-search=native={}", libdir_path.display());

    println!("cargo:rustc-link-lib=dylib=BoardController");
    println!("cargo:rustc-link-lib=dylib=DataHandler");
    println!("cargo:rustc-link-lib=dylib=MLModule");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_path.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("bindings.rs");

    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}

fn get_sdk(target: &str, host: &str, asset: &SdkAsset) -> PathBuf {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // target[/<triple>]/debug/build/mindrove-sys-<hash>/out
    let target_profile_dir = out_dir
        .ancestors()
        .nth(3)
        .expect("unexpected OUT_DIR layout");

    let cache_dir = target_profile_dir.join("mindrove-sdk-cache").join(target);

    fs::create_dir_all(&cache_dir).expect("cannot create SDK cache");

    let sdk_dir = cache_dir.join(format!("mindrovesdk-{SDK_VERSION}"));
    let archive = cache_dir.join(asset.archive);
    let lock_path = cache_dir.join(format!("{SDK_VERSION}.lock"));

    if sdk_dir.exists() {
        return sdk_dir;
    }

    let lock = fs::File::create(lock_path).expect("cannot create SDK lock");

    lock.lock_exclusive().expect("cannot lock SDK cache");

    // Another Cargo process may have finished while we waited.
    if sdk_dir.exists() {
        return sdk_dir;
    }

    if archive.exists() && !sha256_matches(&archive, asset.sha256) {
        fs::remove_file(&archive).expect("cannot remove invalid cached SDK archive");
    }

    if !archive.exists() {
        let url = format!(
            "https://github.com/MindRove/MindRoveSDK/releases/download/{SDK_VERSION}/{}",
            asset.archive
        );

        download(&url, &archive);
    }

    verify_sha256(&archive, asset.sha256);

    let tmp_dir = sdk_dir.with_extension("tmp");

    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).expect("cannot remove stale SDK temp directory");
    }

    extract_zip(&archive, &tmp_dir);

    patch_sdk(&tmp_dir, target, host);

    fs::rename(&tmp_dir, &sdk_dir).expect("cannot finalize SDK extraction");

    sdk_dir
}

fn download(url: &str, destination: &Path) {
    let tmp = destination.with_extension("zip.tmp");

    if tmp.exists() {
        fs::remove_file(&tmp).expect("cannot remove stale download");
    }

    let mut response = ureq::get(url)
        .call()
        .expect("failed to download MindRove SDK");

    let mut output = fs::File::create(&tmp).expect("cannot create SDK archive");

    io::copy(&mut response.body_mut().as_reader(), &mut output)
        .expect("failed writing SDK archive");

    fs::rename(&tmp, destination).expect("cannot finalize SDK download");
}

fn sha256(path: &Path) -> String {
    let mut file = fs::File::open(path).expect("cannot open SDK archive");

    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buf).expect("cannot read SDK archive");

        if n == 0 {
            break;
        }

        hasher.update(&buf[..n]);
    }

    format!("{:x}", hasher.finalize())
}

fn sha256_matches(path: &Path, expected: &str) -> bool {
    sha256(path) == expected
}

fn verify_sha256(path: &Path, expected: &str) {
    let actual = sha256(path);

    assert_eq!(actual, expected, "MindRove SDK SHA256 mismatch");
}

fn extract_zip(archive: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("cannot create extraction directory");

    let file = fs::File::open(archive).expect("cannot open SDK zip");

    let mut zip = zip::ZipArchive::new(file).expect("invalid SDK zip");

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).expect("cannot read ZIP entry");

        // MindRove uses '\' even in some non-Windows archives.
        let name = entry.name().replace('\\', "/");
        let output = destination.join(name);

        if entry.is_dir() {
            fs::create_dir_all(&output).expect("cannot create SDK directory");
            continue;
        }

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent).expect("cannot create SDK directory");
        }

        let mut file = fs::File::create(&output).expect("cannot create extracted SDK file");

        io::copy(&mut entry, &mut file).expect("cannot extract SDK file");
    }
}

fn patch_sdk(sdk: &Path, target: &str, host: &str) {
    match target {
        target if target.contains("-linux-") => {
            patch_linux_sdk(sdk, target, host);
        }

        // Nothing currently required.
        target if target.contains("-windows-") => {}

        // Nothing currently required.
        target if target.contains("-apple-darwin") => {}

        _ => {}
    }
}

fn patch_linux_sdk(sdk: &Path, target: &str, host: &str) {
    // The absolute Nix store RPATH only makes sense for native builds.
    if target != host {
        return;
    }

    let Some(libstdcpp) = find_libstdcpp() else {
        return;
    };

    // On ordinary FHS Linux, libstdc++ is already in the loader's
    // standard search path. Only perform this hack for Nix.
    if !libstdcpp.starts_with("/nix/store/") {
        return;
    }

    let cpp_lib_dir = libstdcpp
        .parent()
        .expect("libstdc++ has no parent directory");

    let rpath = format!("$ORIGIN:{}", cpp_lib_dir.display());

    for name in [
        "libBoardController.so",
        "libDataHandler.so",
        "libMLModule.so",
    ] {
        let library = sdk.join("lib").join(name);

        if !library.exists() {
            continue;
        }

        let status = Command::new("patchelf")
            .arg("--set-rpath")
            .arg(&rpath)
            .arg(&library)
            .status()
            .expect("MindRove requires patchelf when building natively under Nix");

        assert!(status.success(), "failed to patch {}", library.display());
    }
}

fn find_libstdcpp() -> Option<PathBuf> {
    let mut compilers = Vec::new();

    if let Ok(cxx) = env::var("CXX") {
        compilers.push(cxx);
    }

    compilers.push("g++".into());
    compilers.push("c++".into());

    for compiler in compilers {
        let Ok(output) = Command::new(&compiler)
            .arg("-print-file-name=libstdc++.so.6")
            .output()
        else {
            continue;
        };

        if !output.status.success() {
            continue;
        }

        let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_owned());

        // GCC prints the input filename unchanged when it
        // cannot locate the library.
        if path.is_absolute() && path.exists() {
            return Some(path);
        }
    }

    None
}
