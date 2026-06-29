# 21 Prompt Flux (copy-paste)

Format: `<subject>, <style>`. Style suffix sama untuk semua (konsistensi + optimal ASCII).
Negatif tidak diperlukan Flux. Rasio 1:1 atau 4:5, steps ~28 (dev), guidance ~3.5, output PNG.
Simpan sebagai `char_<group>_NN.png` di `assets/source/characters/<group>/`.

**Style suffix:**
`sci-fi character portrait, head and shoulders bust, centered, facing forward, dramatic rim lighting, strong single key light, pure black background, high contrast, detailed face, painterly concept art, cinematic, sharp focus`

---

## Male

**char_male_01** — a stern human male starship captain, short groomed hair, defined jawline, futuristic uniform with shoulder insignia, confident commanding expression, *[+ style suffix]*

**char_male_02** — a human male engineer, messy hair, light stubble, protective goggles pushed up on forehead, grease-smudged jumpsuit, focused look, *[+ style suffix]*

**char_male_03** — an elderly human male scientist, bald head, neat white beard, thin-rimmed glasses, lab coat collar, wise calm expression, *[+ style suffix]*

**char_male_04** — a young human male fighter pilot, flight helmet with raised visor, communications headset, determined expression, *[+ style suffix]*

**char_male_05** — a rugged human male asteroid miner, hard hat with headlamp, dirt-streaked face, thick beard, heavy worksuit, *[+ style suffix]*

**char_male_06** — a charismatic human male merchant trader, slicked-back hair, neat goatee, ornate collar with hanging trinkets, sly confident smile, *[+ style suffix]*

**char_male_07** — a human male navigator, sleek neural visor over the eyes, short hair, tech earpiece, neutral focused expression, *[+ style suffix]*

## Female

**char_female_01** — a commanding human female starship captain, hair in a tight bun, sharp features, decorated uniform, authoritative gaze, *[+ style suffix]*

**char_female_02** — a human female engineer, ponytail, a smudge on her cheek, goggles on forehead, utility jumpsuit, determined look, *[+ style suffix]*

**char_female_03** — a human female scientist, shoulder-length hair, glasses, lab coat collar, intelligent thoughtful expression, *[+ style suffix]*

**char_female_04** — a human female pilot, flight helmet with visor up, short bob haircut, headset, brave confident smirk, *[+ style suffix]*

**char_female_05** — a tough human female miner, hard hat, braided hair, dust-streaked face, rugged worksuit, *[+ style suffix]*

**char_female_06** — an elegant human female merchant, long wavy hair, jeweled earrings, ornate high collar, knowing smile, *[+ style suffix]*

**char_female_07** — a human female navigator, glowing holographic visor, sleek straight hair, earpiece, calm precise expression, *[+ style suffix]*

## Alien

**char_alien_01** — a classic grey alien, smooth bald oversized head, huge black almond-shaped eyes, tiny nose and mouth, slender neck, *[+ style suffix]*

**char_alien_02** — an insectoid alien, chitinous exoskeleton head, large faceted compound eyes, twitching antennae, sharp mandibles, *[+ style suffix]*

**char_alien_03** — a reptilian alien, green scaled skin, vertical slit pupils, ridged brow crest, sharp angular features, *[+ style suffix]*

**char_alien_04** — a one-eyed cyclops alien, a single large central eye, smooth grey skin, broad cranium, calm curious expression, *[+ style suffix]*

**char_alien_05** — a cephalopod alien, bulbous head, large dark glossy eyes, writhing tentacles around the mouth, glistening skin, *[+ style suffix]*

**char_alien_06** — a crystalline alien, head formed of faceted translucent crystal shards, glowing geometric eyes, angular reflective form, *[+ style suffix]*

**char_alien_07** — an energy being alien, a semi-transparent glowing humanoid head made of swirling plasma and light, ethereal wisps, *[+ style suffix]*

---

> `generate_cloud.py` menyusun prompt final otomatis (`subject + ", " + style`) dari
> `comfyui/prompts/characters.json` — sumber tunggal, jadi prompt lokal & cloud selalu sinkron.
