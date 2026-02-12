# Refactoring checklist (pixli engine)

Stav po prvním kole: typy a konstanty rendereru jsou v `types.rs` / `constants.rs`, init logika v `init.rs`. Níže je, co dává smysl ještě předělat.

---

## Vysoká priorita

### 1. Rozdělit `render()` v `src/renderer/mod.rs` (~670 řádků)

Metoda `render()` (ř. 543–1208) dělá vše v jednom bloku: unlit batche, depth pre-pass, shadow pass, main pass (unlit + sky + lit), bloom, SSAO, post. Doporučení:

- **Soukromé helpery ve stejném souboru**, např.:
  - `build_unlit_batches(world, …) -> (batches, entity_order)` + zápis do scratch bufferů
  - `run_depth_prepass_ssao(&mut encoder, …)` – když je SSAO zapnuté
  - `run_shadow_pass(&mut encoder, …)` – když jsou shadows zapnuté
  - `run_main_pass(&mut encoder, …)` – depth + MSAA resolve / scene texture (unlit, sky, lit)
  - `run_bloom(&mut encoder, …)` – když je bloom zapnutý
  - `run_ssao_pass(&mut encoder, …)` – SSAO + blur
  - `run_post_pass(&mut encoder, …)` – composite na swapchain
- `render()` pak jen: early-return když chybí device/queue, příprava view/proj/light, volání helperů v pořadí, `queue.submit(...)`.

Tím se `mod.rs` zkrátí a každý krok renderu bude čitelný z jednoho místa.

---

## Střední priorita

### 2. Rozdělit `src/math.rs` (~957 řádků) na podmoduly

Jeden soubor obsahuje: Vec2, Vec3, Vec4, Quat, Mat4, Transform, Color. Běžná struktura:

```
src/math/
  mod.rs      // pub use vec2::Vec2; pub use vec3::Vec3; ...
  vec2.rs
  vec3.rs
  vec4.rs
  quat.rs
  mat4.rs
  transform.rs
  color.rs
```

- Každý typ do vlastního souboru, `math/mod.rs` jen re-exportuje (`pub use`).
- Ve zbytku kódu se změní jen `use crate::math::…` – názvy typů zůstanou stejné.

---

### 3. `recreate_pipelines()` v `src/renderer/mod.rs` (~220 řádků)

Po změně MSAA se znovu vytváří lit / unlit / sky pipeline. Možnosti:

- **A)** Přesunout do `init.rs` jako `recreate_pipelines(device, &self.lit_pipeline_layout, &self.…, format, msaa) -> (pipeline, unlit_pipeline, sky_pipeline)` a v `mod.rs` jen přiřadit do `self.*`.
- **B)** Nechat v `mod.rs`, ale případně vytáhnout společné kousky (např. vytvoření shader modulu) do malých helperů v `init.rs`.

---

## Nízká priorita / volitelné

### 4. Shooter example – meshe a konfigurace

- Konstanty a „config“ příkladu do `examples/shooter/config.rs` (nebo `constants.rs`).
- Vytváření meshů (kostky, level, …) do `examples/shooter/meshes.rs` (nebo podobný modul), `main.rs` jen skládá systém a volá setup.

### 5. `src/app.rs` (~407 řádků)

- Pokud event loop a obsluha událostí porostou, lze oddělit např. `app/event.rs` nebo `app/loop.rs` a v `app.rs` jen skládat dohromady.

### 6. Varování z buildu

- `ecs/world.rs`: `ComponentStorage` vs `get_storage` visibility, nepoužívaný `get_storage_mut`, lifetime syntax.
- `audio.rs`: nepoužívané pole `data`.
- `renderer/camera.rs`: nepoužívaný `FpsCameraController`.
- `time.rs`: nepoužívané `last_frame`.

Stačí průběžně opravit podle potřeby (využití nebo odstranění mrtvého kódu, úprava visibility/lifetime).

---

## Shrnutí

| Co | Odhad rozsahu | Dopad |
|----|----------------|--------|
| Rozdělení `render()` na helpery | střední | Velký – přehlednost a údržba renderu. |
| Rozdělení `math.rs` na moduly | střední | Lepší struktura, menší soubory. |
| `recreate_pipelines` do init / helperů | malý | Méně duplicity, konzistence s init. |
| Shooter config/meshes | malý | Čistší příklad. |
| App / warnings | malý | Čistší kód a build. |

Nejvíc „profi“ efekt dává **bod 1 (render())** a **bod 2 (math)**. Zbytek lze dělat postupně podle času a chuti.
