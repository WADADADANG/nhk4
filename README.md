# 🦀 NodeHotKeyV4 (v4.0.0) - Rust Native Core Engine
### Cross-Platform Multi-Client WebGL Automation Suite (Windows, Linux, macOS)

โปรเจกต์อัตโนมัติกดปุ่มคีย์บอร์ดและเมาส์ในเบราว์เซอร์แบบพื้นหลัง (Background Automation) สไตล์ **Unreal Engine Blueprints** ที่ถูกเขียนขึ้นใหม่ด้วยภาษา **Rust** เต็มรูปแบบ เพื่อประสิทธิภาพสูงสุด ประหยัด RAM ป้องกันการ Reverse-Engineering และรันได้ข้ามระบบปฏิบัติการ

---

## 🌟 จุดเด่นของเวอร์ชัน Rust Native Core
1. **ความเร็วและความลื่นไหลสูง (Ultra-fast & Lightweight):** Engine ทำงานแบบ Native Binary ไม่ต้องพึ่งพา Node.js Runtime ทำให้กินแร็มน้อยลงมหาศาลและตอบสนองไวกว่าเดิม
2. **ความปลอดภัยระดับ Binary (Memory Safe & Hard to Crack):** คอมไพล์เป็นไฟล์ Binary ป้องกันการแอบอ่านซอร์สโค้ดหรือดัดแปลงได้ยากขึ้น
3. **Cross-Platform:** รองรับทั้ง **Windows**, **Linux**, และ **macOS**
4. **ความเข้ากันได้ 100% (Backwards Compatible):** รองรับไฟล์โปรไฟล์เดิมทั้งหมดใน `configs/profiles/*.json` และโหนดทั้ง 29 ตัว
5. **Standalone Project:** รวมไฟล์ UI (Studio & Launcher) และการตั้งค่าทั้งหมดไว้ในโปรเจกต์เดียวอย่างเป็นระเบียบ

---

## 📁 โครงสร้างโปรเจกต์ (Project Structure)

```text
nodehotkey-rust/
├── src/                    # Rust Native Core Source Code
│   ├── browser/            # Multi-Client CDP Browser Manager
│   ├── config/             # Config Store & File Watcher (SSE)
│   ├── engine/             # Graph Engine, Action Executor & Party Scanner
│   ├── input/              # Native Input Synthesizer & Global Hotkeys
│   ├── server/             # Embedded Axum Web Server & SSE Stream
│   ├── lib.rs              # Library exports & Unit Tests
│   ├── main.rs             # CLI & Native Launcher Entrypoint
│   ├── model.rs            # Data Structures & Profile Schemas
│   └── state.rs            # AppState Management
├── configs/                # Configurations & Profiles
│   ├── global.json         # Global settings, slots, creator identity
│   └── profiles/           # 29 Profile JSONs ([WD] Add-ons, Buff, Heal, etc.)
├── public/                 # Action Node Studio Frontend (Canvas UI)
├── launcher/               # Desktop Launcher Dashboard UI
├── icons/                  # Application Icons
├── docs/                   # Documentation & Architectural Specs
├── Cargo.toml              # Rust Dependencies & Build Manifest
├── Cargo.lock              # Locked Dependency Graph
└── tauri.conf.json         # Tauri v2 Desktop GUI Configuration
```

---

## 🚀 วิธีการรันใช้งาน (How to Run)

### 1. โหมดเซิร์ฟเวอร์ / Web Studio (--server)
เหมาะสำหรับทุกแพลตฟอร์ม (Windows, Linux, macOS):
```bash
cargo run -- --server
```
เปิดเบราว์เซอร์ไปที่:
- **Launcher Dashboard:** `http://127.0.0.1:3088/launcher`
- **Action Node Studio:** `http://127.0.0.1:3088/`

### 2. โหมด Desktop GUI (Tauri Native Window)
```bash
cargo run
```
หรือใช้ Tauri CLI:
```bash
cargo tauri dev
```

### 3. การคอมไพล์เป็นไฟล์พร้อมแจกจ่าย (Release Build)
```bash
cargo build --release
```
ไฟล์ Binary จะอยู่ที่ `target/release/nodehotkey-rust` (หรือ `.exe` บน Windows)

### 4. รันการทดสอบ Unit Tests
```bash
cargo test
```
