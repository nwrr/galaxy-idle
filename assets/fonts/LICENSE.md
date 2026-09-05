# Lisensi Font

## DejaVuSansMono.ttf

Dipakai oleh rasterizer `src/ui/raster.rs` (Buffer→PNG, Phase 2) untuk menggambar glyph teks.
Box-drawing (`│─┌┐└┘├┤┬┴┼`) dan half-block (`▀▄█`) digambar programatik (bukan lewat glyph font),
jadi font ini hanya perlu cakupan Latin/simbol dasar.

- **Sumber:** DejaVu Fonts project (turunan Bitstream Vera Fonts), dipaketkan di `ttf-dejavu`
  (paket sistem Arch Linux, `/usr/share/fonts/TTF/DejaVuSansMono.ttf`).
- **Lisensi:** Bitstream Vera License + DejaVu changes — bebas dipakai, dimodifikasi, dan
  didistribusikan ulang (termasuk dalam produk komersial), dengan syarat nama "Bitstream" atau
  "DejaVu" tidak dipakai untuk endorse produk turunan tanpa izin. Teks lengkap:
  <https://dejavu-fonts.github.io/License.html>
- **Versi:** sesuai paket `ttf-dejavu` terpasang di lingkungan build (tidak ada perubahan berkas).
