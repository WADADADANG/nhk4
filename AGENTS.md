# 🦀 NodeHotKeyV4 - AI Assistant & Project Memory Guide
*(Project Context, System Architecture & Engineering Principles for AI Agents)*

เอกสารนี้ทำหน้าที่เป็น **Project Memory (สมองส่วนจดจำ)** ของโปรเจกต์ **NodeHotKeyV4** เพื่อให้ AI ทุกตัวที่เปิดแชทใหม่เข้าใจบริบททั้งหมดทันที สามารถสานต่องานได้ตรงจุด และปฏิบัติตามกฎของผู้ใช้อย่างเคร่งครัด

---

## 📌 1. ภาพรวมโปรเจกต์และความเป็นมา (Overview & Evolution)
- **โปรเจกต์ต้นแบบ (Old Project):** `C:\Users\WADADADANG\Desktop\MyProjects\NodeHotkey`
  - ทำงานด้วย Node.js Runtime + Puppeteer + Electron
- **โปรเจกต์ใหม่นี้ (NodeHotKeyV4):** `C:\Users\WADADADANG\Desktop\MyProjects\NodeHotKeyV4`
  - ถูก **เขียนขึ้นใหม่ด้วยภาษา Rust เต็มรูปแบบ (Rust Native Core Engine)**
  - ประสิทธิภาพสูงขึ้นมาก กิน RAM ต่ำ ตอบสนองไวระดับ microseconds
  - ปลอดภัยในระดับ Binary ยากต่อการแอบเจาะหรือดัดแปลง (Memory-Safe & Binary Obfuscation)
  - รองรับ Cross-Platform (Windows, Linux, macOS)
  - ยังคงรักษาความเข้ากันได้ 100% (Backwards Compatible) กับไฟล์ Profile JSON เดิมทั้ง 29 Profile และโหนด Blueprint ทั้ง 29 ตัว

---

## 🛠️ 2. Tech Stack & เทคโนโลยีที่ใช้งาน
| องค์ประกอบ | เทคโนโลยีที่ใช้ | รายละเอียด |
| :--- | :--- | :--- |
| **Core Engine** | Rust 2021 + Tokio (Async Runtime) | ประมวลผล Action Graph, Variable Store และ Hotkey Events |
| **Web Server** | Axum 0.8 + Tower-HTTP | ให้บริการ REST API และ Server-Sent Events (SSE) พอร์ต `3088` |
| **Desktop GUI** | Tauri v2 (`tauri.conf.json`) | หน้าต่าง Desktop App ควบคุม Launcher Dashboard (ตั้งค่าเปิด `http://127.0.0.1:3088/launcher` เป็นหน้าแรก) |
| **Browser CDP** | Tokio-Tungstenite (WebSocket) | ยิงคำสั่งตรงผ่าน Chrome DevTools Protocol ควบคุมเกมในเว็บเบราว์เซอร์ |
| **Input Driver** | `rdev` | ดักจับและสังเคราะห์การกดปุ่มคีย์บอร์ดระดับ OS ข้ามแพลตฟอร์ม |
| **Computer Vision** | `image` (Rust crate) | สแกนแถบ Crimson Pixel (หลอดเลือด HP ในเกม Flyff Universe) |
| **Audio & TTS** | `rodio` + Edge TTS (`reqwest`) | เล่นเสียงแจ้งเตือนแบบ Native โดยไม่ต้องพึ่งคำสั่ง PowerShell |
| **Frontend UI** | Vanilla JS, Canvas API, CSS | Canvas Blueprint Node Editor (`public/`) และ Launcher UI (`launcher/`) |

---

## 📁 3. โครงสร้างโฟลเดอร์หลัก (Project Structure)
```text
NodeHotKeyV4/
├── src/                          # Rust Native Core Source Code
│   ├── audio/                    # Native Sound & Edge TTS service
│   ├── browser/                  # Multi-Client CDP Browser Manager & Anti-Detection
│   ├── config/                   # Config Store & File Watcher (Notify crate)
│   ├── engine/                   # Blueprint Graph Engine & 29 Action Node Executors
│   ├── input/                    # Global Hotkey Listener & Key Synthesizer
│   ├── security/                 # Hardware ID (HWID), Anti-Crack & Obfuscation
│   ├── server/                   # Axum Web Server, SSE stream, API Routes
│   ├── vision/                   # Party HP Scanner (Crimson Pixel Detection)
│   ├── lib.rs                    # Core Library & Unit Tests
│   ├── main.rs                   # Entrypoint (CLI / Desktop GUI)
│   ├── model.rs                  # Data Schemas (Profile, WorkflowNode, GlobalConfig)
│   └── state.rs                  # AppState (Arc-wrapped thread-safe state)
├── configs/                      # Settings & Profile Databases
│   ├── global.json               # Global settings, Dynamic client slots, Hotkeys
│   └── profiles/                 # ไฟล์โปรไฟล์เกม JSON ทั้งหมด ([WD], [SOLO], [CW], RM, etc.)
├── public/                       # Canvas Node Editor Frontend (Web UI)
├── launcher/                     # Desktop Launcher Dashboard (Matrix Cards, PiP, Overlay)
├── docs/                         # เอกสารสถาปัตยกรรม (System Specs, Security Standards, Dynamic Slots)
├── Cargo.toml                    # Rust Dependencies & Build Manifest
├── tauri.conf.json               # Tauri v2 Desktop Window Config
├── start.bat                     # สคริปต์ดับเบิลคลิกเปิดระบบ (ตรวจจับโค้ดอัตโนมัติ: ถ้าไม่แก้เปิดในเสี้ยววิ / ถ้าแก้คอมไพล์ให้เองทันที)
├── build.bat                     # สคริปต์ดับเบิลคลิกสำหรับคอมไพล์โปรเจกต์ (Debug / Release)
├── test.bat                      # สคริปต์ดับเบิลคลิกรัน Unit Tests บน Windows
└── .agents/                      # AI Customizations, Rules & Skills
    ├── rules/working_style.md    # กฎการทำงานร่วมกับผู้ใช้อย่างเคร่งครัด
    ├── skills/handoff/SKILL.md   # ทักษะสรุปงานและส่งมอบแชทใหม่ (/handoff)
    └── handoff_latest.md         # เอกสารสถานะล่าสุดของโปรเจกต์
```

---

## 🧩 4. รายชื่อโหนด Blueprint ทั้ง 29 ตัว (All 29 Modular Nodes)
ระบบรองรับโหนด 6 หมวดหมู่:
1. **Triggers:** `trigger` (Keyboard/Mouse Event)
2. **Actions & Input:** `key_press`, `key_hold`, `forwarder`, `buff_sequence`, `macro_group`
3. **Logic & Flow:** `branch`, `var_branch`, `condition_group`, `loop`, `loop_scheduler`, `delay`, `sequencer`, `reroute`
4. **Vision & Party:** `party_scanner`, `party_heal`, `party_buff`, `party_slot`, `screenshot`
5. **Variables & Data:** `variable`, `var_get`, `var_set`, `format_text`
6. **Safety & Utilities:** `step_log`, `sound`, `tts`, `emergency_stop`, `client_check`, `webhook_out`

---

## ⚡ 5. ระบบ Dynamic Client Slots (ขยายจออิสระ & Fixed Slot ID)
- หน้าจอเกมแต่ละจอจะผูกกับ **Slot Number คงที่ถาวร** (Fixed Slot ID) เช่น Client 1, Client 2, Client 3...
- **ห้ามขยับรหัสอัตโนมัติ (No Auto Re-index):** การลบจอตรงกลางแถว (เช่น จอ 2) จะแปลงสภาพเป็นช่องว่าง `[ ➕ สร้างจอ 2 ]` เพื่อไม่ให้กระทบโหนดใน Canvas ที่เล็งเป้าไว้ที่ `targetClient: "3"`

---

## 🚨 6. กฎเหล็กในการทำงานร่วมกับผู้ใช้ (Strict Working Style Rules)
*อ้างอิงจาก `.agents/rules/working_style.md`*

1. **ทำงานตรงไปตรงมา (Straightforward & Direct Execution):**
   - ทำเฉพาะสิ่งที่ผู้ใช้ระบุหรือถาม
   - ห้าม Over-engineer หรือคิดระบบใหญ่โตเกินจำเป็น
2. **ต้องสรุปก่อนแก้ไขเสมอ (Mandatory Summary First):**
   - **ห้ามแก้ไขโค้ดทันทีก่อนได้รับความเห็นชอบ**
   - ตรวจสอบสาเหตุที่แท้จริง (Root Cause) แล้วสรุปให้ผู้ใช้ฟังก่อนเสมอ:
     1. สาเหตุคืออะไร (เกิดที่ไฟล์ไหน บรรทัดไหน)
     2. ทำไมถึงเกิดพฤติกรรมนั้นขึ้น
     3. แนวทางและจุดที่จะแก้ไข
   - **รอคำยืนยัน:** รอให้ผู้ใช้ตอบรับหรือส่งสัญญาณอนุมัติ (เช่น คำว่า *"ลุย"* หรือ *"โอเค"*) จึงค่อยเริ่มลงมือแก้โค้ด
3. **ทำทีละสเต็ปอย่างชัดเจน (Step-by-Step, One Issue at a Time):**
   - แก้ทีละประเด็น ไม่แก้หลายเรื่องปนกัน
4. **ตรวจสอบด้วยการทดสอบเสมอ (Test & Verification 100%):**
   - หลังการแก้ไขโค้ดทุกครั้ง ต้องรัน `cargo check` หรือ `cargo test` เพื่อยืนยันว่าการทดสอบผ่าน 100% และไม่มีข้อผิดพลาด (Zero Regression)
5. **รูปแบบการสื่อสาร (Communication Style):**
   - สื่อสารด้วยภาษาไทยที่เป็นมิตร ชัดเจน ตรงไปตรงมา กระชับ และไม่อ้อมค้อม
6. **ข้อกำหนดไฟล์ Batch Script (.bat) บน Windows:**
   - ห้ามใส่ภาษาไทยและ Emoji ในไฟล์ `.bat` โดยเด็ดขาด เขียนเป็นภาษาอังกฤษ ASCII ล้วนเท่านั้น เพื่อป้องกันปัญหา `cmd.exe` ตีความไบต์ผิดพลาด
7. **กฎการออกแบบ UI: ห้ามใช้สไตล์ Neon เด็ดขาด (Strictly No Neon / Clean Dark Matte Only):**
   - ห้ามใส่สี Neon, แสงเรืองแสง (Glow / Box-Shadow Neon), แถบสีฉูดฉาด หรือ Neon Gradients ใน UI โดยเด็ดขาด
   - โทนสีและสไตล์ที่ใช้ต้องเป็น **Clean Dark Matte** เรียบหรู สะอาดตา สบายตา (เช่น สไตล์แบบ Steam, VS Code, หรือ Discord Dark Theme)
   - ใช้พื้นหลังมืดด้าน ขอบเส้นสีจางๆ (Subtle Border) และไอคอน/ตัวหนังสือที่อ่านง่าย ไม่แสบตา
8. **การนำเสนอทางเลือกมากกว่า 1 แผนการเสมอก่อนเริ่มออกแบบ/แก้ไขระบบ (Multi-Plan Proposal Rule):**
   - เมื่อมีโจทย์การออกแบบหรือแก้ไขเชิงสถาปัตยกรรม/ระบบ ต้องคิดและนำเสนอ **แผนการมากกว่า 1 แผนการเสมอ** เพื่อให้ผู้ใช้สามารถเปรียบเทียบและเลือกแผนการที่ดีและตรงไปตรงมาที่สุด
   - เกณฑ์ 4 เสาหลักที่ต้องใช้ในการวิเคราะห์และประเมินทุกแผนการ:
     1. **ความตรงไปตรงมาของระบบ (Straightforwardness):** เรียบง่าย ไม่ซับซ้อน ไม่ Over-engineer
     2. **เน้นลื่นมากที่สุด (Maximum Smoothness & Zero Lag):** กินทรัพยากรต่ำสุด ไร้การหน่วงหรือกระตุก 100%
     3. **ไม่ทำเป็นแบบลูปมากเกินไป (No Polling Loops):** หลีกเลี่ยงการวนลูปสลีปดึงข้อมูลเป็นระยะ เช่น วนเช็คทุก 10 วิ หรือลูปถี่ๆ
     4. **เน้นดักจับเหตุการณ์ (Event-Driven Architecture):** ทำงานทันทีเมื่อมี Event เกิดขึ้น เพื่อความรวดเร็วและประหยัดทรัพยากรสูงสุด

---

## 💻 7. คำสั่งพื้นฐานสำหรับทดสอบและรันโปรเจกต์
- **ตรวจสอบไวยากรณ์และคอมไพล์:**
  ```bash
  cargo check
  # หรือคอมไพล์ผ่าน: build.bat (เลือก Debug หรือ Release ได้)
  ```
- **รันชุดทดสอบ Unit Tests:**
  ```bash
  cargo test
  # หรือดับเบิลคลิกไฟล์: test.bat
  ```
- **รันในโหมด Web Server / Headless (Port 3088):**
  ```bash
  cargo run -- --server
  # หรือดับเบิลคลิกไฟล์: start.bat (เปิด Engine พร้อมเด้งเปิด Launcher อัตโนมัติ)
  ```
- **คำสั่งจัดการความปลอดภัย & License (CLI):**
  ```bash
  # ดูรหัส HWID ของเครื่องปัจจุบัน:
  cargo run -- --hwid
  # สุ่มสร้าง Serial Keys สำหรับส่งให้ลูกค้า (พร้อมคำสั่ง SQL D1 Insert):
  cargo run -- --gen-serial 10
  # สร้าง License Key ผูกกับ HWID ที่ระบุ (PRO Lifetime):
  cargo run -- --gen-key NHKV4-XXXX-XXXX-XXXX-XXXX
  # สร้าง Master Developer Key (ใช้ได้ทุกเครื่อง):
  cargo run -- --gen-key-universal
  ```

---

## 🔒 8. สถาปัตยกรรมความปลอดภัยและแผนป้องกันโปรแกรม (Security & Anti-Crack Roadmap)
ข้อตกลงและทิศทางการออกแบบระบบความปลอดภัยตามความต้องการของผู้ใช้:
1. **ข้อมูลที่ไม่ต้องป้องกัน (Open & Accessible):**
   - **ไฟล์ Profile JSON (`configs/profiles/*.json`):** ปล่อยเป็น Plaintext ธรรมดา ไม่ต้องเข้ารหัส เพื่อความสะดวกในการแก้ไข ปรับแต่ง และแชร์
   - **หน้ากาก UI (`launcher/`, `public/`):** ปล่อยเป็นไฟล์ปกติ ไม่ต้องเข้ารหัส เพราะไม่มีความลับทางธุรกิจ
2. **ข้อมูลที่ต้องป้องกันเข้มงวด 2 ส่วนหลัก:**
   - **A. ปกป้องฐานข้อมูล & กุญแจ API (Database Secrets & API Protection):**
     - ห้ามเก็บ Master DB Credentials ไว้ในไบนารีของ Client
     - ทุกคำสั่งต้องผ่าน API Gateway ตัวกลาง (เช่น Cloudflare Worker)
     - ซ่อน String ลับใน Binary ด้วย Compile-time Obfuscation (XOR / Encryption) ป้องกันการส่อง Hex/Strings
     - เข้ารหัสช่องทางสื่อสารด้วย HTTPS TLS 1.3
   - **B. ระบบ License & ป้องกันคนแฮก/ก๊อปโปรแกรมไปแจก (HWID & Anti-Crack):**
     - ผูกสิทธิ์ใช้งานกับรหัสประจำเครื่อง Hardware ID (HWID: CPU + Motherboard + Windows MachineGuid)
     - ตรวจสอบคีย์และลายเซ็นดิจิทัล (Signed Token) กับเซิร์ฟเวอร์
     - ใช้ความปลอดภัยระดับ Binary ของ Rust (`strip = true`, `lto = true`, `opt-level = 3`) เพื่อป้องกันการ Reverse-Engineering หรือ Patch โค้ด

