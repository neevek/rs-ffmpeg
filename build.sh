#!/bin/bash

function usage {
  echo "Usage: ./build.sh <native|arm64|arm> [--build-openssl --build-mp3lame --build-x264 --build-ffmpeg --build-all --debug-ffmpeg]"
  exit 1
}

arch=$1

case $arch in
  arm)
    ;;
  arm64)
    ;;
  native)
    ;;
  *)
    usage
    ;;
esac

shift

while [[ "$#" -gt 0 ]]; do
    case $1 in
        --build-openssl) build_openssl=1 ;;
        --build-mp3lame) build_mp3lame=1 ;;
        --build-x264) build_x264=1 ;;
        --build-ffmpeg) build_ffmpeg=1 ;;
        --build-all) build_all=1 ;;
        --debug-ffmpeg) debug_ffmpeg=1 ;;
        --*) echo "Unknown parameter: $1"; usage ;;
    esac
    shift
done

curl_proxy=
if [[ $http_proxy != "" ]]; then
  curl_proxy="-x $http_proxy"
fi

function do_make {
  check_error
  make -j$(nproc 2> /dev/null || echo 8)
  check_error
}

function check_ndk {
  if [[ "$ANDROID_NDK_ROOT" = "" ]]; then
    echo "set the ANDROID_NDK_ROOT environment variable first."
    exit
  fi
}

function check_error {
  if [[ $? -ne 0 ]]; then
    echo "==========================================="
    echo ">>>>>>>>>> failed! <<<<<<<<<<<<"
    echo "==========================================="
    exit 1
  fi
}

function define_host_triplet {
  host_triplet=$(ls -1 $ANDROID_NDK_ROOT/toolchains/llvm/prebuilt | tail -1)
  android_prebuilt=$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/$host_triplet
}

function clear_exports {
  unset CC
  unset STRIP
  unset STRINGS
  unset AR
  unset RANLIB
  unset host
  unset target_triplet
  unset conf
  unset basic_conf
}

function setup_android_toolchain {
  max_platform="android-21"
  for p in `ls $ANDROID_NDK_ROOT/platforms`; do
    if [[ "$p" > "$max_platform" ]]; then
      max_platform=$p
    fi
  done
  platform_level=$(echo $max_platform | cut -d- -f2)
  
  host_triplet=$(ls -1 $ANDROID_NDK_ROOT/toolchains/llvm/prebuilt | tail -1)
  android_prebuilt=$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/$host_triplet
  
  if [[ $arch == "arm64" ]]; then
    host=aarch64-linux
    target_triplet=aarch64-linux-android
    target_triplet2=aarch64-linux-android
    conf=--sysroot=$android_prebuilt/sysroot
    export CC=$android_prebuilt/bin/aarch64-linux-android${platform_level}-clang
  elif [[ $arch == "arm" ]]; then
    host=armv7-linux
    target_triplet=arm-linux-androideabi
    target_triplet2=armv7a-linux-androideabi
    conf=--sysroot=$android_prebuilt/sysroot
    export CC=$android_prebuilt/bin/armv7a-linux-androideabi${platform_level}-clang 
  fi

  export STRIP=$android_prebuilt/bin/llvm-strip
  export STRINGS=$android_prebuilt/bin/llvm-strings
  export AR=$android_prebuilt/bin/llvm-ar
  export RANLIB=$android_prebuilt/bin/llvm-ranlib
}

function fetch_source {
  branch=$1
  repo=$2
  dest_dir=$3
  
  echo "cloning into $dest_dir..."
  should_clone=1
  if [[ -d "$dest_dir" ]]; then
    echo "updating source code..."
    cd $dest_dir
    git pull --rebase origin $branch
    if [[ $? -eq 0 ]]; then
      should_clone=0
    fi
    cd -
  fi
  if [[ $should_clone -eq 1 ]]; then
    git clone --depth=1 -b $branch $repo $dest_dir
  fi
}

function get_filesize {
  filepath=$1
  if [[ "$filepath" == "" ]] || [[ ! -f "$filepath" ]]; then
    echo 0
  fi
  wc -c "$filepath" | awk '{print $1}'
}

function build_mp3lame {
  echo "================================================="
  echo "================ building mp3lame ==============="
  echo "================================================="
  clear_exports
  
  filename="lame-3.100.tar.gz"
  if [[ $(get_filesize $filename) -lt 1024000 ]]; then
    echo "downloading $filename ..."
    curl -v $curl_proxy https://phoenixnap.dl.sourceforge.net/project/lame/lame/3.100/lame-3.100.tar.gz > $filename
    if [[ $? -ne 0 ]]; then
      echo "downloading lamemp3 failed"
      exit 1
    fi
  fi

  cp ../lame-3.100-symbols.patch .
  rm -rf lame-3.100
  tar zxf $filename
  patch lame-3.100/include/libmp3lame.sym lame-3.100-symbols.patch
  
  lame_conf=--enable-shared=no
  host=
  if [[ $arch == "arm64" ]] || [[ $arch == "arm" ]]; then
    check_ndk
    setup_android_toolchain
  else
    lame_conf="$lame_conf --enable-lto"
  fi

  cd lame-3.100
  prefix=`pwd`/build
  make clean 2> /dev/null
  rm -rf $prefix
  
  ./configure --prefix=$prefix CC=$CC --host=$host $lame_conf
  do_make
  make install
  cd -
}

function build_x264 {
  echo "================================================="
  echo "================ building x264 =================="
  echo "================================================="
  clear_exports
  
  fetch_source "stable" "https://github.com/mirror/x264" "x264"

  cd x264
  prefix=`pwd`/build
  make clean 2> /dev/null
  rm -rf $prefix

  conf=
  if [[ $arch == "arm64" ]] || [[ $arch == "arm" ]]; then
    check_ndk
    setup_android_toolchain
    conf=--host=$host
  else
    conf="--enable-lto"
  fi

  ./configure \
    --prefix=$prefix \
    --enable-static \
    --enable-strip \
    --enable-pic $conf

  do_make
  make install

  cd -
}

function build_openssl {
  echo "================================================="
  echo "================ building openssl ==============="
  echo "================================================="
  clear_exports
  
  fetch_source "OpenSSL_1_1_1n" "https://github.com/openssl/openssl" "openssl"
  
  cd openssl
  
  prefix=`pwd`/build
  make clean 2> /dev/null
  rm -rf $prefix
  if [[ $arch = "arm64" ]] || [[ $arch = "arm" ]]; then
    check_ndk
    setup_android_toolchain
    
    conf=
    if [[ $arch = "arm64" ]]; then
      conf=android-arm64
      PATH=$android_prebuilt/bin:$ANDROID_NDK_ROOT/toolchains/aarch64-linux-android-4.9/prebuilt/$host_triplet/$target_triplet/bin:$PATH
    else
      conf=android-arm
      PATH=$android_prebuilt/bin:$ANDROID_NDK_ROOT/toolchains/arm-linux-androideabi-4.9/prebuilt/$host_triplet/$target_triplet/bin:$PATH
    fi
    
    ./Configure --prefix=$prefix --openssldir=$prefix $conf no-shared
  else
    ./config --prefix=$prefix --openssldir=$prefix no-shared '-Wl,-rpath,$(LIBRPATH)'
  fi

  do_make
  make install_sw
  cd -
}

# DOES NOT WORK CURRENTLY
function build_x265 {
  fetch_source "3.4" "https://github.com/videolan/x265" "x265"

  basic_conf=
  if [[ $arch == "arm64" ]] || [[ $arch == "arm" ]]; then
    check_ndk
    setup_android_toolchain
    basic_conf="-DCMAKE_SYSTEM_PROCESSOR=arm7l"
  fi
  
  out_dir=`pwd`/x265/out
  x265_build_dir=x265/source/build
  rm -rf $x265_build_dir $out_dir
  mkdir -p $x265_build_dir
  cd $x265_build_dir 
  
  cmake -DCMAKE_INSTALL_PREFIX=$out_dir -DCMAKE_C_COMPILER=$CC -DCMAKE_FIND_ROOT_PATH=$android_prebuilt/sysroot -DENABLE_CLI=False -DCROSS_COMPILE_ARM=1 -DCMAKE_SYSTEM_PROCESSOR=arm7l ..

  # do_make
  # make install
  cd -
}

function build_ffmpeg {
  echo "================================================="
  echo "================ building ffmpeg ================"
  echo "================================================="
  clear_exports
  
  fetch_source "release/4.3" "https://github.com/FFmpeg/FFmpeg" "ffmpeg"
  
  cd ffmpeg
  make clean 2> /dev/null
  out_dir=`pwd`/build 
  rm -rf $out_dir
  mkdir -p $out_dir
  
  basic_conf="
    --prefix=$out_dir
    --disable-programs
    --disable-doc
    --disable-ffplay
    --enable-encoders
    --enable-decoders
    --enable-shared
    --enable-static
    --enable-protocol=file
    --enable-protocol=crypto
    --enable-pic
    --enable-small
    --enable-gpl
    --enable-nonfree
    --enable-libmp3lame
    --enable-libx264
    --enable-openssl
    --pkg-config-flags=--static"

    extra_cflags="-I../x264/build/include -I../lame-3.100/build/include -I../openssl/build/include"
    extra_ldflags="-L../x264/build/lib -lx264 -L../lame-3.100/build/lib -lmp3lame -L../openssl/build/lib -lssl -lcrypto"
    
  if [[ $arch == "arm64" ]] || [[ $arch == "arm" ]]; then
    setup_android_toolchain
    
    sysroot=$ANDROID_NDK_ROOT/sysroot
    include_dir=$sysroot/usr/include/$target_triplet
    lib_dir1=$ANDROID_NDK_ROOT/platforms/android-${platform_level}/arch-${arch}/usr/lib
    lib_dir2=$android_prebuilt/lib/gcc/$target_triplet/4.9.x
    cc=$android_prebuilt/bin/${target_triplet2}${platform_level}-clang
    ld=$android_prebuilt/bin/${target_triplet2}${platform_level}-clang
    ar=$android_prebuilt/${target_triplet}/bin/ar
    as=$android_prebuilt/${target_triplet}/bin/as
    strip=$android_prebuilt/${target_triplet}/bin/strip
    pkg_config=$android_prebuilt/bin/llvm-config
    
    echo "ndk root: [$ANDROID_NDK_ROOT]"
    echo "arch: [$arch]"
    echo "platform: [$max_platform] [$platform_level]"
    echo "prebuilt dir: [$android_prebuilt]"
    echo "host_triplet: [$host_triplet]"
    echo "target_triplet: [$target_triplet]"
    echo "include_dir: [$include_dir]"
    echo "lib_dir1: [$lib_dir1]"
    echo "lib_dir2: [$lib_dir2]"
    
    if [[ $arch == "arm64" ]]; then
      basic_conf="$basic_conf --arch=aarch64"
    elif [[ $arch == "arm" ]]; then
      basic_conf="$basic_conf --arch=arm"
    fi

    basic_conf="$basic_conf --pkg-config=$pkg_config --ar=$ar --strip=$strip --ld=$ld --cc=$cc --as=$cc --target-os=android --enable-cross-compile"
    extra_cflags="$extra_cflags -I$include_dir"
    extra_ldflags="$extra_ldflags -L$lib_dir1 -L$lib_dir2 -lm -lz"
  fi

  if [[ "$debug_ffmpeg" -eq 1 ]]; then
    basic_conf="$basic_conf --disable-optimizations --disable-stripping"
  fi

  ./configure $basic_conf --extra-cflags="$extra_cflags" --extra-ldflags="$extra_ldflags"
  do_make
  make install
  cd -
}

mkdir -p thirdparty
cd thirdparty

if [[ "$build_all" -eq 1 ]] || [[ "$build_openssl" -eq 1 ]]; then
  build_openssl
fi

if [[ "$build_all" -eq 1 ]] || [[ "$build_x264" -eq 1 ]]; then
  build_x264
fi

if [[ "$build_all" -eq 1 ]] || [[ "$build_mp3lame" -eq 1 ]]; then
  build_mp3lame
fi

if [[ "$build_all" -eq 1 ]] || [[ "$build_ffmpeg" -eq 1 ]]; then
  build_ffmpeg
fi
