# PROMPTS — Subagent Siap-Pakai (Phase 2: UI/UX)

Prompt untuk delegasi ke subagent. Ganti `<...>`. Pakai hanya bila menghemat konteks; tugas kecil inline.

## Investigator (read-only)

> **subagent_type:** `cavecrew-investigator`
>
> Di `src/` galaxy-idle: <pertanyaan, mis. "di mana panel planet dirender & bagaimana draw_view
> dispatch per breakpoint" / "di mana ui::sprite & ui::portrait dipakai (atau tidak)">. Kembalikan
> tabel `file:line` + ringkas. Jangan sarankan fix.

## Builder (edit terfokus 1–2 file UI)

> **subagent_type:** `cavecrew-builder`
>
> Di galaxy-idle, implementasi <item UI/UX>. Spec: `abstraction/design/06-ui.md` bagian <x>. Sentuh
> hanya <file1[, file2]> di `src/ui/`. UI hanya membaca `App`/`GameState`. Ikuti layout modul
> `abstraction/design/07-architecture.md`. Jangan ubah mekanik/file lain. Kembalikan diff ringkas.

## Reviewer (sebelum centang)

> **subagent_type:** `cavecrew-reviewer`
>
> Review diff working-tree galaxy-idle untuk item <item>. Cek: sesuai `abstraction/design/06-ui.md`,
> tak ada overflow/clipping di panel, UI tak memutasi state, asset id valid di `manifest.ron`, tak
> langgar guardrail `agent/phase2/LOOP.md`. Satu baris per temuan, severity-tagged.

## Explore (sweep luas)

> **subagent_type:** `Explore`
>
> Petakan di galaxy-idle bagaimana <area, mis. "asset .ans dimuat & dirender via ansi-to-tui",
> "theme palette diterapkan ke panel"> bekerja. Breadth: medium. Kembalikan kesimpulan + file kunci.

## Catatan

- Output subagent kembali sebagai tool-result — relay yang penting saja.
- Lanjutkan subagent hidup via SendMessage (konteks utuh) daripada spawn baru.
