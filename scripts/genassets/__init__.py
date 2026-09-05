"""Paket generator asset galaxy-idle (dipecah dari skrip tunggal; lihat CONVENTIONS.md §3).

Modul:
  numfx     — util numerik (noise, sphere, lighting).
  celestial — render planet/star/ship (luminance & RGB per-biome, prosedural).
  imgconv   — konversi PNG sumber (ComfyUI, M08) → ANSI half-block (M09; gantikan celestial
              prosedural mulai M10/M11).
  ansi      — encode RGB → ANSI half-block (depth tc/256/16).
  textart   — luminance ramp, portrait & banner (Pillow).
  catalog   — registry job, path, publish portrait.
  build     — orkestrasi `do_all`.
"""
