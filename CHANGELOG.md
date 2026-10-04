# Changelog

## 0.1.0 - 2026-10-05

### Added

- MIT `LICENSE` and `THIRD_PARTY_NOTICES.md` for the statically linked CharLS, OpenJPEG and IJG code.
- CI workflow and a release workflow that builds precompiled NIFs for macOS and Linux (glibc), arm64 and x86_64.

### Fixed

- The crate compiles: crate versions that exist (`openjp2` 0.6.1, `charls-sys` 2.4, `jpeg-encoder` 0.7, `rustler` 0.38) and their real APIs.
- Codecs return `{:ok, binary}` or `{:error, {reason, message}}` as `Dicom.Codec` requires; corrupt data and invalid metadata no longer raise.
- JPEG-LS: colour streams in any interleave mode decode to pixel-interleaved samples; noise no longer overflows CharLS's output estimate; truncated or corrupt streams fail at once instead of spinning for seconds inside CharLS.
- Lossless JPEG of 9–16 bits decodes to little-endian samples.

### Changed

- Every NIF runs on a dirty CPU scheduler.
- Decoded frames are checked against the metadata before allocating, which also covers CharLS advisory GHSA-mqrg-gfc8-73ff.
- JPEG 2000 encoding writes a raw J2K codestream and applies the reversible colour transform only for `YBR_RCT`.
- Precompiled NIFs for macOS and Linux (glibc), arm64 and x86_64; other targets build from source with `DICOM_CODECS_BUILD=1`.
