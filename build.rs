use regex::Regex;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

fn run_command(cur_dir: &str, cmd: &str, args: &str) -> io::Result<()> {
    let re = Regex::new("\\s+").unwrap();
    let args = re.replace_all(args, " ");

    let mut cmd = Command::new(cmd);
    cmd.current_dir(cur_dir);
    fill_command_args(&mut cmd, &args);
    let status = cmd.status()?;

    if !status.success() {
        Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
    } else {
        Ok(())
    }
}

fn fill_command_args(cmd: &mut Command, args: &str) {
    for arg in args.split(" ").into_iter() {
        cmd.arg(arg);
    }
}

fn add_header_binding<Filter>(
    builder: bindgen::Builder,
    parent: &PathBuf,
    filter: &mut Filter,
) -> bindgen::Builder
where
    Filter: FnMut(&str) -> bool,
{
    let files = fs::read_dir(parent).unwrap();
    let mut builder = builder;
    for file in files {
        let file = file.unwrap();
        let file_type = file.file_type().unwrap();

        if file_type.is_file() {
            let path = file.path();
            let path = path.to_str().unwrap();
            if filter(path) {
                builder = builder.header(path);
            }
        } else if file_type.is_dir() {
            builder = add_header_binding(builder, &file.path(), filter);
        }
    }
    builder
}

fn build(build_type: &str) {
    let target = env::var("TARGET").unwrap();

    let target_arch = match target.as_str() {
        "aarch64-linux-android" => "arm64",
        "armv7-linux-androideabi" => "arm",
        _ => "native",
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let proj_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let ffmpeg_build_dir = format!("{}/thirdparty/ffmpeg/build", out_dir);
    let ffmpeg_include_dir = format!("{}/include", ffmpeg_build_dir);
    let build_script = format!("{}/build.sh", proj_dir);

    let args = format!("{} {}", target_arch, build_type);
    run_command(&out_dir, &build_script, &args).unwrap();

    let mut ignored_headers = vec![
        "libavutil/hwcontext_cuda.h",
        "libavutil/hwcontext_vdpau.h",
        "libavutil/hwcontext_vaapi.h",
        "libavutil/hwcontext_opencl.h",
        "libavutil/hwcontext_d3d11va.h",
        "libavcodec/d3d11va.h",
        "libavutil/hwcontext_dxva2.h",
        "libavcodec/dxva2.h",
        "libavutil/hwcontext_qsv.h",
        "libavcodec/qsv.h",
        "libavutil/hwcontext_vdpau.h",
        "libavcodec/vdpau.h",
        "libavcodec/xvmc.h",
    ];
    let mut bindings_builder =
        bindgen::Builder::default().clang_arg(format!("-I{}/include", ffmpeg_build_dir));

    if target_arch == "native" {
        ignored_headers.push("libavutil/hwcontext_vulkan.h");
    } else {
        ignored_headers.push("libavutil/hwcontext_videotoolbox.h");
        ignored_headers.push("libavcodec/videotoolbox.h");

        let ndk_root = env::var("ANDROID_NDK_ROOT").unwrap();
        // the following code only compiles on MacOS
        bindings_builder = bindings_builder.clang_arg(format!(
            "-I{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include",
            &ndk_root
        ));
        if target_arch == "arm64" {
            bindings_builder = bindings_builder
                .clang_arg(format!("-I{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include/aarch64-linux-android", &ndk_root));
            bindings_builder = bindings_builder
                .clang_arg(format!("-L{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/aarch64-linux-android/30", &ndk_root));
        } else {
            bindings_builder = bindings_builder
                .clang_arg(format!("-I{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/include/arm-linux-androideabi", &ndk_root));
            bindings_builder = bindings_builder
                .clang_arg(format!("-L{}/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/arm-linux-androideabi/30", &ndk_root));
        }
    }

    let mut filter = |path: &str| {
        for h in &ignored_headers {
            if path.ends_with(h) {
                return false;
            }
        }
        true
    };

    let ffmpeg_include_dir = PathBuf::from(&ffmpeg_include_dir);
    let bindings_builder = add_header_binding(bindings_builder, &ffmpeg_include_dir, &mut filter);

    let bindings = bindings_builder
        .allowlist_type("(av|AV).*")
        .allowlist_function("(av|AV).*")
        .allowlist_var("(av|AV).*")
        .generate()
        .expect("Unable to generate");

    bindings
        .write_to_file(format!("{}/bindings.rs", out_dir))
        .expect("Couldn't write bindings!");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build.sh");

    // build types for build.sh: [--build-openssl --build-mp3lame --build-x264 --build-ffmpeg --build-all --debug-ffmpeg]
    let build_type = env::var("RS_FFMPEG_BUILD_TYPE").unwrap_or("".into());
    if !build_type.is_empty() {
        build(&build_type);
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let ffmpeg_lib_dir = format!("{}/thirdparty/ffmpeg/build/lib", out_dir);
    let openssl_lib_dir = format!("{}/thirdparty/openssl/build/lib", out_dir);
    let x264_lib_dir = format!("{}/thirdparty/x264/build/lib", out_dir);
    let lame_lib_dir = format!("{}/thirdparty/lame-3.100/build/lib", out_dir);

    println!("cargo:rustc-link-search=all={}", &ffmpeg_lib_dir);
    println!("cargo:rustc-link-search=all={}", &openssl_lib_dir);
    println!("cargo:rustc-link-search=all={}", &x264_lib_dir);
    println!("cargo:rustc-link-search=all={}", &lame_lib_dir);
    println!("cargo:rustc-link-lib=static=avformat");
    println!("cargo:rustc-link-lib=static=avcodec");
    println!("cargo:rustc-link-lib=static=avfilter");
    println!("cargo:rustc-link-lib=static=swresample");
    println!("cargo:rustc-link-lib=static=swscale");
    println!("cargo:rustc-link-lib=static=avutil");
    println!("cargo:rustc-link-lib=static=avdevice");
    println!("cargo:rustc-link-lib=static=ssl");
    println!("cargo:rustc-link-lib=static=crypto");
    println!("cargo:rustc-link-lib=static=x264");
    println!("cargo:rustc-link-lib=static=mp3lame");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=m");

    let target = env::var("TARGET").unwrap();
    if target.ends_with("apple-darwin") {
        println!(
            "cargo:rustc-link-search=framework={}",
            "/System/Library/Frameworks"
        );
        println!("cargo:rustc-link-lib=framework=VideoToolbox");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
    }
}
