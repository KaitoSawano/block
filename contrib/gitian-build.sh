#!/bin/bash
set -e

build=false
linux=true
windows=true
osx=true
signer=""
commit=""
proc=2
mem=2000

usage() {
    cat <<EOF
Usage: $0 [options] signer commit

Options:
  -b|--build       Build binaries
  -o|--os <os>     Operating systems: l (linux), w (windows), x (osx). Default lwx.
  -j <n>           Number of processes. Default 2.
  -m <n>           Memory in MiB. Default 2000.
  -h|--help        Show this help.
EOF
}

while :; do
    case "$1" in
        -b|--build) build=true; shift ;;
        -o|--os)
            os="$2"
            linux=false; windows=false; osx=false
            [[ "$os" == *l* ]] && linux=true
            [[ "$os" == *w* ]] && windows=true
            [[ "$os" == *x* ]] && osx=true
            shift 2
            ;;
        -j) proc="$2"; shift 2 ;;
        -m) mem="$2"; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) break ;;
    esac
done

signer="$1"
commit="$2"

if [ -z "$signer" ] || [ -z "$commit" ]; then
    usage
    exit 1
fi

if [ "$build" = true ]; then
    if [ "$linux" = true ]; then
        echo "Building Linux..."
        pushd gitian-builder
        ./bin/gbuild -j "$proc" -m "$mem" --commit zenachain="$commit" \
            ../zenachain/contrib/gitian-descriptors/gitian-linux.yml
        popd
    fi

    if [ "$windows" = true ]; then
        echo "Building Windows..."
        pushd gitian-builder
        ./bin/gbuild -j "$proc" -m "$mem" --commit zenachain="$commit" \
            ../zenachain/contrib/gitian-descriptors/gitian-win.yml
        popd
    fi

    if [ "$osx" = true ]; then
        echo "Building macOS..."
        pushd gitian-builder
        ./bin/gbuild -j "$proc" -m "$mem" --commit zenachain="$commit" \
            ../zenachain/contrib/gitian-descriptors/gitian-osx.yml
        popd
    fi
fi

echo "Selesai. Cek output di gitian-builder/build/out/"
