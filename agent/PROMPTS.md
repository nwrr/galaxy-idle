# PROMPTS — Subagent Siap-Pakai

Prompt yang bisa dipakai looping agent saat mendelegasikan ke subagent. Ganti `<...>`. Pakai hanya
bila benar menghemat konteks/usaha; tugas kecil cukup inline.

## Investigator (read-only, "di mana / apa yang memanggil")

> **subagent_type:** `cavecrew-investigator`
>
> Cari di `src/` galaxy-idle: <pertanyaan, mis. "di mana `GameState` di-mutasi saat tick" / "apa
> yang memanggil `economy::extract`">. Kembalikan tabel `file:line` + ringkas. Jangan sarankan fix.

## Builder (edit terfokus 1–2 file)

> **subagent_type:** `cavecrew-builder`
>
> Di galaxy-idle, implementasi <item checklist>. Spec: `abstraction/<file>.md` bagian <x>. Sentuh
> hanya <file1[, file2]>. Ikuti layout modul `abstraction/design/07-architecture.md`. Jangan ubah file lain.
> Setelah edit, pastikan `cargo build` lokal mental-check ok. Kembalikan diff ringkas.

## Reviewer (sebelum centang checklist)

> **subagent_type:** `cavecrew-reviewer`
>
> Review diff working-tree galaxy-idle untuk item <item>. Cek: sesuai spec `abstraction/<file>`,
> tak ada panic-prone unwrap di path runtime, id konten valid, tak melanggar guardrail `agent/LOOP.md`.
> Satu baris per temuan, severity-tagged.

## Explore (sweep luas, scope tak pasti)

> **subagent_type:** `Explore`
>
> Petakan di galaxy-idle bagaimana <area, mis. "render panel UI di-dispatch per breakpoint">
> bekerja. Breadth: medium. Kembalikan kesimpulan + file kunci, bukan dump file.

## Catatan

- Output subagent kembali ke thread utama sebagai tool-result — relay yang penting saja.
- Lanjutkan subagent yang masih hidup via SendMessage (konteks utuh) daripada spawn baru.
