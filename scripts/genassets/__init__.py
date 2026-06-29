"""Paket generator asset galaxy-idle (dipecah dari skrip tunggal; lihat CONVENTIONS.md §3).

Modul:
  numfx     — util numerik (noise, sphere, lighting).
  celestial — render planet/star/ship (luminance & RGB per-biome).
  ansi      — encode RGB → ANSI half-block (depth tc/256/16).
  textart   — luminance ramp, portrait & banner (Pillow).
  catalog   — registry job, path, publish portrait.
  build     — orkestrasi `do_all`.
"""
