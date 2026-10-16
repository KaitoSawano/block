# Contrib

Folder ini berisi skrip dan descriptor untuk Gitian build.

## Isi

- `gitian-build.sh` — script untuk menjalankan Gitian build.
- `gitian-descriptors/` — resep build untuk Linux, Windows, macOS.
- `gitian-keys/` — folder untuk menyimpan PGP key developer.

## Gitian

Gitian adalah proses build deterministik yang dipakai Bitcoin dan Dogecoin
untuk menghasilkan binary rilis. Beberapa developer independen membangun
binary dari source yang sama, menandatangani hasilnya, dan hanya jika hash
cocok, binary dianggap aman.

**Gitian tidak bisa dijalankan di GitHub Actions.** Build dilakukan di VM
terisolasi (KVM/LXC/Docker) di mesin lokal atau server.

### Setup

1. Clone repositori yang dibutuhkan:

```bash
mkdir -p ~/gitian-build && cd ~/gitian-build
git clone https://github.com/devrandom/gitian-builder.git
git clone https://github.com/<user>/zenachain.git
git clone https://github.com/<user>/zenachain.sigs.git
