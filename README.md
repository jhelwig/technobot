# Build & Dependencies

## macOS

### Dependencies

```sh
brew install pkg-config openssl libsodium opus libopusenc ffmpeg youtube-dl
```

### Compiling

Unfortunately, the `openssl` crate requires some help in finding where OpenSSL
is installed, so we need to set both `OPENSSL_INCLUDE_DIR`, and
`DEP_OPENSSL_INCLUDE`.

```sh
OPENSSL_INCLUDE_DIR=$(brew --prefix openssl)/include DEP_OPENSSL_INCLUDE=$(brew --prefix openssl)/include cargo build
```
