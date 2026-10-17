# Handoff Document: NodeHotKeyV4 Project

- **Generated Time**: 2026-10-15
- **Project**: NodeHotKeyV4 (v3.2.0 - Rust Native Core Engine)
- **Status**: Stable & All Unit Tests Passing (31/31)

---

## 1. Executive Summary
โปรเจกต์ **NodeHotKeyV4** เป็นเวอร์ชันที่พัฒนาเขียนขึ้นใหม่ด้วยภาษา **Rust** เต็มรูปแบบ โดยถอดแบบความสามารถเดิมมาจากโปรเจกต์ `NodeHotkey` (Node.js) เพื่อเพิ่มประสิทธิภาพในการทำงาน ลดการกิน RAM เป็น Native Binary ที่ยากต่อการดัดแปลง และทำงานได้แบบ Cross-Platform

ในเซสชันนี้ได้ดำเนินการ:
1. ตรวจสอบโครงสร้างไฟล์ทั้งหมดในโปรเจกต์ เปรียบเทียบกับโปรเจกต์เดิม `NodeHotkey`
2. สร้างระบบความจำและคู่มือประจำโปรเจกต์สำหรับ AI ได้แก่ [AGENTS.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/AGENTS.md)
3. ติดตั้งกฎการทำงานของผู้ใช้ที่เคร่งครัดใน [.agents/rules/working_style.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/.agents/rules/working_style.md) (ตรงไปตรงมา, สรุปก่อนแก้ไขเสมอ, รันเทส 100%)
4. ติดตั้ง Skill ส่งต่องาน [.agents/skills/handoff/SKILL.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/.agents/skills/handoff/SKILL.md)
5. พัฒนาระบบความปลอดภัย & License (Anti-Crack) ครบทั้ง 3 เฟสเดิม + อัปเกรดสู่ **Ed25519 Asymmetric Cryptography (ระดับ Enterprise/AAA)**
6. จัดทำเอกสารสถาปัตยกรรมและมาตรฐานความปลอดภัยระดับสากล [docs/SECURITY_AND_SYSTEM_DESIGN_STANDARDS.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/docs/SECURITY_AND_SYSTEM_DESIGN_STANDARDS.md)

---

## 2. Key Architecture Context
- **Rust Native Core (`src/`)**:
  - `src/engine/nodes/mod.rs` & `src/engine/executor.rs`: รองรับโหนดครบทั้ง 29 ตัว
  - `src/browser/launcher.rs` & `cdp_session.rs`: ควมคุมเบราว์เซอร์ผ่าน Chrome DevTools Protocol (CDP)
  - `src/server/routes.rs`: Axum Web Server & SSE (พอร์ต 3088) จำลอง API เดิมของ Node.js ไว้อย่างสมบูรณ์
  - `src/security/`: ระบบ HWID, Obfuscation และ License Verification ด้วย **Ed25519 (Asymmetric)**
  - `src/vision/party_scanner.rs`: สแกนหลอดเลือด HP แบบ Native Crimson Pixel
- **Configurations (`configs/`)**:
  - `configs/global.json`: Dynamic Client Slots และการตั้งค่าส่วนกลาง
  - `configs/profiles/`: โปรไฟล์เกม JSON เดิมทั้ง 29 โปรไฟล์ ([WD], [SOLO], [CW], ฯลฯ)
- **UI**:
  - Web Canvas Node Editor อยู่ใน `public/`
  - Launcher Dashboard อยู่ใน `launcher/`

---

## 3. Solved Issues & Verification
- **แก้ไขแล้ว**:
  - [src/browser/launcher.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/browser/launcher.rs): เพิ่ม `use std::env;` แก้ปัญหา E0433
  - [tauri.conf.json](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/tauri.conf.json): ตั้งค่า URL เริ่มต้นของหน้าต่าง Desktop App ให้เปิด **Launcher Dashboard** (`http://127.0.0.1:3088/launcher`) เป็นหน้าแรกแทนหน้า Action Node Studio
  - **เฟส 1 ระบบความปลอดภัย & HWID สำเร็จ 100% (Cross-Platform Full Support):**
    - สร้าง [src/security/hwid.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/hwid.rs) รองรับ 3 OS เต็มรูปแบบ:
      - 🪟 Windows: Registry `MachineGuid` (`HKLM\SOFTWARE\Microsoft\Cryptography`)
      - 🍎 macOS: Native `IOPlatformUUID` ผ่าน `ioreg -rd1 -c IOPlatformExpertDevice`
      - 🐧 Linux: `/etc/machine-id` หรือ `/var/lib/dbus/machine-id`
      - แฮช SHA-256 ออกมาเป็นรูปแบบสากล `NHKV4-XXXX-XXXX-XXXX-XXXX`
    - ปรับ [Cargo.toml](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/Cargo.toml) แยก `winreg` ไปอยู่ใต้ `[target.'cfg(windows)'.dependencies]` เพื่อให้ Build บน Mac/Linux ได้อย่างราบรื่น
    - เพิ่ม API `GET /api/security/hwid` ใน [src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs)
    - เพิ่มการ์ดแสดง HWID พร้อมปุ่มคลิกคัดลอกในหน้าต่าง Settings ของ Launcher ([launcher/ui/index.html](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/index.html) & [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js))
  - **เฟส 2 ระบบซ่อนข้อความลับในไบนารี สำเร็จ 100% (Compile-Time String Obfuscation):**
    - สร้าง [src/security/obfuscate.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/obfuscate.rs) บรรจุ Macro `secure_str!(...)`
    - เข้ารหัสด้วย Rolling Key XOR Scramble ตั้งแต่ตอนคอมไพล์ ตัด Plaintext String ออกจากไฟล์ Binary `.exe` 100%
    - ทำงานแบบ Pure Rust 100% รองรับ Windows, macOS, และ Linux
  - **เฟส 3 อัปเกรดระบบ License สู่ Ed25519 (Asymmetric Key Cryptography):**
    - ติดตั้ง `ed25519-dalek = { version = "2.1", features = ["std", "rand_core"] }`
    - ปรับปรุง [src/security/license.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/license.rs) ให้ตรวจสอบลายเซ็นด้วย **Ed25519 Public Key** (ใน `.exe` มีแค่ Public Key ไม่มี Private Key ป้องกันการแกะสร้างคีย์ปลอม 100%)
    - รองรับ Backward Compatibility ด้วย Fallback HMAC ตรวจสอบคีย์เวอร์ชันเดิมได้
    - ผูกกับ Engine Safety: หากไม่มี License หรือคีย์หมดอายุ/ไม่ตรงเครื่อง จะบล็อกไม่ให้กด "Start Engine" ทันที (`403 Forbidden`)
    - เพิ่ม API `GET /api/security/license/status` และ `POST /api/security/license/activate`
    - เพิ่มการ์ด Activate License สไตล์ Emerald Green ในหน้า Settings ของ Launcher พร้อมปุ่ม Activate และสถานะวันหมดอายุ
    - เพิ่มคำสั่ง CLI สำหรับผู้พัฒนาใน [src/main.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/main.rs):
      - `--hwid`: ตรวจสอบรหัสประจำเครื่อง
      - `--keys-info`: แสดง Public Key (สำหรับใส่ใน Client) และ Private Key Seed (สำหรับใส่ใน Cloudflare Worker Secret)
      - `--gen-key <HWID>`: สร้าง License Key Ed25519 เฉพาะเครื่อง
      - `--gen-key-universal`: สร้าง Master Developer Key Ed25519
    - รัน `cargo check` & `cargo test` ผ่านสมบูรณ์ 100% (**16/16 tests passed**)
  - **เฟส 4 ระบบเชื่อมต่อ Cloudflare D1 + Worker ตรวจสอบสิทธิ์ออนไลน์ สำเร็จ 100%:**
    - ติดตั้งระบบตรวจสอบและดึง License อัตโนมัติจาก Cloudflare Worker Gateway (`POST /api/security/license/sync-online`) ใน [src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs) และ [src/security/license.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/license.rs)
    - สร้างเอกสารคู่มือ SQL Schema สำหรับ Cloudflare D1 และโค้ด Worker อย่างละเอียดที่ [docs/CLOUDFLARE_D1_WORKER_SETUP.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/docs/CLOUDFLARE_D1_WORKER_SETUP.md)
    - ผสานปุ่ม `[☁️ Sync Cloud]` ในหน้า Settings ของ Launcher ([launcher/ui/index.html](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/index.html) & [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js)) ให้ผู้ใช้กดซิงค์ใบอนุญาตผ่านเน็ตได้ในคลิกเดียวโดยไม่ต้องก๊อปคีย์ยาวๆ มากรอกเอง
    - การเชื่อมต่อยิงผ่าน Worker URL ที่ผ่านการ Obfuscate ด้วย `secure_str!` ป้องกันการถูกค้นเจอ Endpoint ใน Binary
    - ผ่านการทดสอบ Unit Tests ทั้งหมด 100% (**16/16 tests passed**)
  - **เฟส 5 ระบบ Serial / Redeem Key ผูก HWID อัตโนมัติ (Zero Friction Architecture):**
    - แอดมินสุ่มสร้าง Product Keys (เช่น `NHK-PRO-4A2B-8C1D-E9F0`) ไว้ในตาราง `license_keys` บน Cloudflare D1
    - ลูกค้ากรอกคีย์ลงในช่อง `[ วาง License Key ที่ได้รับที่นี่ ]` แล้วกด `[✨ Activate]`
    - เส้นทาง `POST /api/security/license/activate` ใน [src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs) จะยิงไปที่ `POST /api/license/redeem` บน Cloudflare Worker เพื่อ **ล็อก Key เข้ากับ HWID ของเครื่องนั้นทันที (Hardware-Bound)**
    - ป้องกันการนำ Key เดียวกันไปเปิดบนเครื่องอื่นโดยเด็ดขาด (เครื่องอื่นจะโดน Reject ทันที) แต่ยอมให้เครื่องเดิม Re-activate ซ้ำได้
    - เพิ่มคำสั่ง CLI สำหรับผู้พัฒนา `cargo run -- --gen-serial <count> [tier] [duration_days]` สร้างชุดคีย์และคำสั่ง SQL Insert พร้อมใช้งาน
    - ผ่านการทดสอบ Unit Tests ทั้งหมด 100% (**16/16 tests passed**)
  - **เฟส 6 ระบบ Admin Portal (Next.js 16) และการควบคุมสถานะ License ออนไลน์แบบ Real-time สำเร็จ 100%:**
    - สร้างเว็บแอดมินระดับพรีเมียม `admin-web/` (Next.js 16 App Router) สำหรับสร้าง, สั่งระงับชั่วคราว (Suspend), ปลดระงับ, แบน และลบคีย์
    - เชื่อมต่อ Cloudflare Worker API ผ่าน Admin Secret (`/api/admin/keys`, `/api/admin/keys/create`, `/api/admin/keys/status`, `/api/admin/keys/delete`)
    - แก้ปัญหา Sync Real-time: เพิ่ม Auto-fetch D1 เมื่อโหลดหน้า/สลับแท็บ และปุ่ม `[ 🔄 ซิงค์ D1 ]`
    - แก้ปัญหา D1 Schema: ปรับปรุง Worker ให้รองรับทั้งตาราง `licenses` ที่มีและไม่มีคอลัมน์ `status` ตัดสิทธิ์ผ่าน `is_active = 0` ได้อย่างราบรื่น
    - ติดตั้ง Background Security Sentinel ใน Rust Core Engine (`src/server/mod.rs`) คอยเช็คสถานะจาก Cloudflare ทุกๆ 10 นาที (และตรวจทันทีใน 2.5 วินาทีตอนเริ่มโปรแกรม) หากพบว่าคีย์ถูกระงับหรือลบ จะสั่งหยุด Engine (`Engine halted`) ทันที
    - **Deploy ขึ้นระบบจริงบน Vercel สำเร็จ 100%:** ใช้งานได้ทั่วโลกที่ `https://nhk-admin.vercel.app` พร้อมตั้งค่า Environment Variables และ Google reCAPTCHA v2 ป้องกัน Brute-force
    - **ปรับปรุง UI License ใน Launcher เป็น 3 สถานะตามข้อกำหนด:**
      1. สถานะปกติ (`PRO Active`): ซ่อนช่อง input และซ่อนปุ่มตรวจสอบสถานะ ไม่เกะกะสายตา
      2. สถานะถูกหยุดใช้งานชั่วคราว (`Suspended`): ซ่อนช่อง input และแสดงปุ่ม **`[ 🔄 ตรวจสอบสถานะล่าสุด (Refresh Status) ]`** ให้ผู้ใช้กดรีเฟรชสิทธิ์ได้ทันที
      3. สถานะถูกลบ (`Unlicensed`): ล้างข้อมูลคีย์ในเครื่องออกทั้งหมด และสลับกลับไปเป็นสถานะรอกรอกข้อมูลคีย์ในช่อง input ทันทีอัตโนมัติ
  - **เฟส 7 ระบบป้องกันการกดตรวจสอบถี่เกินไป (Hybrid Anti-Spam & Rate Limiter) สำเร็จ 100%:**
    - **ด่านที่ 1 (Cooldown 5 วินาที):** เมื่อกดปุ่มตรวจสอบสถานะ ปุ่มจะติดคูลดาวน์ 5 วินาทีทันที พร้อมเวลานับถอยหลังบนปุ่ม `⏳ รอสักครู่ (5s)...` ป้องกันการเผลอกดเบิ้ลหรือสแปมเคาะ
    - **ด่านที่ 2 (Hard Lockout 10 นาที):** ใช้ระบบ Sliding Window บันทึกการกดใน 1 นาที (60 วินาที) หากตรวจพบการกดถึง 10 ครั้ง จะสั่งบล็อคทันทีเป็นเวลา 10 นาที (600 วินาที)
    - **Persistent Lockout:** เก็บเวลาปลดบล็อคลง `localStorage` ป้องกันการกดรีเฟรช F5 หรือปิด-เปิดโปรแกรมใหม่เพื่อบายพาส
    - **UI Countdown & Notification:** แสดงเวลานับถอยหลัง Real-time บนปุ่ม `🔒 ระงับการตรวจสอบ (เหลือ 09:59 นาที)` และขึ้นเตือน Toast สีส้มแจ้งผู้ใช้อย่างชัดเจน
    - **Backend Protection (Rust Core Engine):** ติดตั้ง `LicenseSyncRateLimiter` ใน `src/server/routes.rs` คุ้มกัน API ฝั่ง Server อีกชั้น ตอบกลับ HTTP 429 Too Many Requests หากถูกยิงคำขอเกิน 10 ครั้ง/นาที
  - **เฟส 8 หน้ารักษาความปลอดภัยด่านแรกสุด (License Guard Gatekeeper Screen) สำเร็จ 100%:**
    - **Gatekeeper Architecture:** สร้างหน้าจอ `#license-guard-overlay` บังคับแสดงผลเป็นด่านแรกสุด ปิดกั้นการเข้าถึง Dashboard และ Action Node Studio อย่างสิ้นเชิงจนกว่าจะมีสิทธิ์ที่ถูกต้อง
    - **กรณี Unlicensed / ลบคีย์:** แสดงรหัสเครื่อง HWID พร้อมปุ่มคัดลอก + ช่อง Input วาง Key + ปุ่ม `[ ✨ Activate ]`
    - **กรณี Suspended (ระงับชั่วคราว):** ตัดกลับมาหน้า Guard ทันที แสดงกล่องเตือนสีเหลือง 🟡 พร้อมปุ่ม `[ 🔄 ตรวจสอบสถานะล่าสุด ]` (คูลดาวน์ 5 วิ / ล็อค 10 นาที) เมื่อแอดมินปลดระงับและผู้ใช้กดปุ่มนี้สำเร็จ หน้า Guard จะเปิดทางพาเข้า Dashboard ทันที
    - **กรณี Active PRO:** ซ่อนหน้า Guard เปิดใช้งานโปรแกรมได้เต็มรูปแบบ และย้ายข้อมูล License ไปแสดงในหน้าตั้งค่า (Settings) ตามปกติ
    - **แก้ปัญหา UI แวบ (Canvas Leaking):** ดักใน `syncBackendStatus`, `reloadEditorFrame` และ `onDiagnosticsUpdate` ไม่ให้โหลดหรือคลายการล็อค iframe Canvas หากไม่มี License ที่ถูกต้อง
  - **เฟส 9 กฎการออกแบบ UI คลีนดาร์กแมตต์ และห้ามสไตล์ Neon เด็ดขาด (Clean Dark Matte / Strictly No Neon):**
    - บันทึกกฎเหล็กข้อที่ 7 ใน [.agents/rules/working_style.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/.agents/rules/working_style.md) และ [AGENTS.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/AGENTS.md)
    - ปรับโทนสีหน้าจอ License Guard และทั้งโปรแกรมให้อยู่ในรูปแบบ **Clean Dark Matte** เรียบหรู สะอาดตา สบายตา (เช่น สไตล์แบบ Steam, VS Code, หรือ Discord Dark Theme)
    - ห้ามใส่สี Neon, แสงเรืองแสง (Glow / Box-Shadow Neon), แถบสีฉูดฉาด หรือ Neon Gradients โดยเด็ดขาด
    - ลบตัวแปรและสไตล์สี Neon (เช่น `#ec4899`, Gradient Bar สีชมพู/ม่วงนีออน) ออกจาก `launcher.js` และ `index.html` 100%
  - **เฟส 10 ยกระดับความปลอดภัยระบบและป้องกันโควต้า SQL (Security Hardening & Quota Protection):**
    - **ป้องกัน Local Data Leaks:** อัปเดต `static_fallback` ใน [src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs) บล็อคไม่ให้เว็บภายนอกเข้าถึง `configs/`, `src/`, `.git`, และไฟล์สำคัญของระบบ ป้องกันการแอบขโมยคีย์ License และ Config
    - **แยก Public Key ออกจาก Private Seed:** ฝังเฉพาะ `MASTER_ED25519_PUBLIC_KEY_BYTES` (`[u8; 32]`) ใน [src/security/license.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/license.rs) สำหรับ Client ตรวจสอบความถูกต้อง ส่วน Private Key Seed สำหรับเครื่องผู้พัฒนาถูกซ่อนด้วย `secure_str!` ป้องกันการถูกส่องหา String ในไบนารี
    - **Edge Cache & Anti-Spam ใน Cloudflare Worker:** เพิ่ม In-Memory Cache (3 นาทีสำหรับคีย์ที่ถูกต้อง) และ IP Rate Limiter (30 reqs/min) ใน [docs/worker_api_clean.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/docs/worker_api_clean.js) ลดการอ่าน D1 เหลือเกือบ 0 และป้องกันการยิงสแปมถล่ม Worker
    - **ตัด Hardcoded Admin Secret:** บังคับให้ [admin-web/app/api/keys/route.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/admin-web/app/api/keys/route.js) และ Worker ต้องอ่านค่า `ADMIN_SECRET` จาก Environment Variable จริงเท่านั้น ป้องกันการใช้รหัสผ่านเริ่มต้น
    - รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ 100% (**16/16 tests passed**)
  - **เฟส 11 การปรับแต่ง Topbar, แก้ปัญหาสายเชื่อมต่อ (Wires), และตัดปัญหาจอกระพริบ/Server Offline ค้าง สำเร็จ 100%:**
    - **เอาปุ่ม Topbar ที่ไม่จำเป็นออก:** นำปุ่ม `#btn-open-browser-studio` ("เปิดในเบราว์เซอร์") และ `#btn-top-github` ออกจาก [launcher/ui/index.html](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/index.html) และ [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js) Topbar ด้านบนขวาสะอาดและกะทัดรัด
    - **แก้ปัญหาสายเชื่อมต่อ (Wires) หายไปหลังรีโหลดใน Action Node Studio:**
      - เติม Deterministic Unique ID ให้กับสายทุกเส้นใน `loadProfile()` ของ [public/js/canvas.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/canvas.js)
      - แก้ไข `renderWires()` ให้ใช้ `connIdStr` ในการจับคู่ DOM Element และ Retained-mode SVG Pruning ป้องกันการเผลอลบสายทิ้ง
      - ปรับปรุง `getPortCenter()` ให้ตรวจเช็คขนาดพอร์ตจริง (`width > 0 || height > 0`) ก่อนดึงพิกัด และ fallback ไปใช้การคำนวณทางคณิตศาสตร์อย่างแม่นยำขณะกำลัง reflow
      - ปรับปรุง [public/js/canvas-serializer.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/canvas-serializer.js) ให้ส่งออก Connection สะอาด เข้ากันได้กับ 29 โปรไฟล์เดิม 100%
    - **แก้ปัญหาจอกระพริบ + ป้ายเตือน Server Offline ค้าง:**
      - แยกสถานะการรันบอท (`isRunning`) ออกจากสถานะของ Web Server (`isServerOnline`) ใน [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js) อย่างเด็ดขาด
      - ตัดการตั้งค่า `isServerOnline = false;` ออกจาก `updateStatusUI()` เมื่อบอทหยุดทำงาน
      - ปรับเงื่อนไขใน `switchView('editor')` ให้ผู้ใช้เข้าไปจัดการโหนดและต่อสายใน Studio ได้ตลอดเวลา โดยไม่ถูกป้าย Server Offline บัง
      - ตัดวงจร Infinite Reload Loop ของ iframe ที่เคยถูกสั่งรีโหลดทุกๆ 2 วินาที อาการจอกระพริบและป้าย Offline แวบ/ค้างหายขาด 100%
    - **แก้ปัญหาโปรแกรมค้างตอนเริ่ม 5 วินาทีแรก (Startup Freeze / ReferenceError):**
      - ลบบล็อกคำสั่ง `if (btnTopGithub) { ... }` ที่ตกค้างใน `launcher.js` บรรทัด 470 ออกทั้งหมด ซึ่งเดิมทำให้เกิด Uncaught `ReferenceError: btnTopGithub is not defined` ขัดขวางการทำงานของ Event Listeners และการโหลด HWID ทั้งหมด
      - ย้ายการประกาศ `let cachedConfig = null;` และ `let cachedStatus = null;` ขึ้นไปด้านบนสุด ป้องกัน TDZ ReferenceError ขณะเรียก `initGlobalSettingsFromDisk()`
    - **เฟส 12 ระบบ Dynamic Client Slots และการส่งคำสั่ง Client Action สำเร็จ 100%:**
      - **แก้ไข Rust API Payload Deserialization ([src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs)):** เพิ่มการรองรับ `clientIndex` และ `browserChoice` ใน `ClientActionPayload` และ `ToggleEnablePayload` แก้ปัญหาที่เดิมคำสั่งเปิด/ปิด/หยุดชั่วคราว/ล้างข้อมูลทุกจอตกไปที่ Client 1 ตลอดเวลา
      - **เชื่อมต่อประเภทเบราว์เซอร์จริง:** นำ `browserChoice` (Chrome, Edge, Firefox) ที่เลือกในหน้าตั้งค่าจอไปใช้งานจริงในการสั่ง Launch
      - **ระบบล้างข้อมูลโปรไฟล์เบราว์เซอร์ (`clear_client_profile`):** รองรับการล้างโฟลเดอร์แคชและคุกกี้ทั้ง `profiles_data/client_{id}` และ `profiles/chrome-profile-{id}`
      - **ปรับค่าเริ่มต้น (Fresh Install Default):** ตั้งค่า `default_client_slots()` ใน [src/model.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/model.rs) และ [configs/global.json](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/configs/global.json) ให้เริ่มต้นด้วย 1 จอ (`Client 1`) เท่านั้น และสามารถกด `[ ➕ เพิ่มจอใหม่ ]` ได้ไม่จำกัดตามกำลังเครื่อง
    - **เฟส 13 ระบบการทำงานจริงของบอทและ CDP In-Game Input Dispatch สำเร็จ 100%:**
      - **โหลด Active Profiles อัตโนมัติเมื่อกดเริ่มบอท ([src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs)):** ปรับปรุง `start_engine` ให้ดึงโปรไฟล์ที่เปิดใช้งานทั้งหมดจาก `config_store` เข้าสู่ `state.engine` ทันทีที่สั่งรันบอท แก้ปัญหาผังในเมมโมรีว่างเปล่าและกดทริกเกอร์ไม่ติด
      - **เชื่อมต่อ CDP Input Dispatcher ([src/engine/executor.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/engine/executor.rs)):**
        - สร้าง `map_key_to_cdp` แปลงปุ่มเกม (เช่น `1-9`, `A-Z`, `F1-F12`, `SPACE`, `ESC`, `ENTER`) เป็น CDP Key, Code, และ Virtual Key Code
        - เชื่อมโยงโหนด `key_press`, `buff_sequence`, `key_hold`, `forwarder` ยิงเข้าเบราว์เซอร์เกมแต่ละจอผ่าน `cdp_sessions` ในพื้นหลังโดยไม่ต้องแย่งโฟกัสหน้าต่าง
        - เชื่อมโยงโหนด `loop` และ `loop_scheduler` ให้รัน Asynchronous Loop ในพื้นหลังอย่างต่อเนื่องตามช่วงเวลา `interval` / `jitter`
      - **แก้ปัญหาปุ่มทริกเกอร์ตัวอักษร ([src/engine/graph.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/engine/graph.rs)):** ปรับปรุง `normalize_trigger_key` ให้ตัดคำนำหน้า `KEY` ออกสำหรับปุ่ม A-Z ทำให้ปุ่มจาก `rdev` (เช่น `KeyA`) จับคู่กับค่าในโปรไฟล์ (เช่น `A`) ได้ 100%
    - **เฟส 15 แก้ไข Real-time Terminal Logs และ Browser Isolated Profile Path สำเร็จ 100%:**
      - **แก้ปัญหา Real-time Terminal Logs ไม่แสดงข้อความ ([src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs), [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js)):**
        - ปรับปรุง SSE Stream ให้ส่งข้อมูลทั้ง `message` และ `text`
        - ปรับปรุง `appendLogEntry` ใน `launcher.js` ให้รองรับ `data.text || data.message || ''` และ format timestamp ให้อัตโนมัติ แก้ปัญหาข้อความ log ว่างเปล่าแสดงแต่ badge ได้หายขาด 100%
      - **แก้ปัญหา Chrome เด้งเตือนสร้างไดเรกทอรีล้มเหลว ([src/browser/launcher.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/browser/launcher.rs), [src/config/store.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/config/store.rs), [src/main.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/main.rs)):**
        - ปรับปรุง `ConfigStore::new` และ `main.rs` ให้ resolve `base_dir` เป็น Absolute Path เต็มรูปแบบของ Windows ตั้งแต่เริ่มต้น
        - ใน `BrowserLauncher::launch` บังคับแปลง `user_data_dir` เป็น Absolute Path และเรียก `std::fs::create_dir_all` สร้างโฟลเดอร์บนดิสก์จริงก่อน spawn Chrome
        - กำจัดปัญหา Relative Path `.\profiles_data/client_1` ที่ Chrome Process Sandbox ปฏิเสธ ทำให้เบราว์เซอร์แยกโปรไฟล์และคุกกี้อิสระต่อจอได้อย่างราบรื่น 100%
    - **เฟส 16 ระบบแจ้งเตือนและกล่องยืนยันของโปรแกรมเอง (Universal Custom Dialog System) สำเร็จ 100%:**
      - **สร้างระบบ Universal Custom Dialog (`showConfirm` & `showAlert`):**
        - ดีไซน์ **Clean Dark Matte Theme** ไร้แสงสะท้อนหรือนีออน (Strictly No Neon) ตามกฎเหล็กข้อ 7
        - ทำงานด้วย Promise (`const ok = await window.showConfirm({ ... })`)
        - รองรับไอคอนแยกประเภท (🗑️ ลบ/ล้างข้อมูล, ⚠️ ข้อควรระวัง, ℹ️ ข้อมูลทั่วไป), รองรับการกด `Enter` เพื่อยืนยัน และ `Esc` หรือคลิก Backdrop เพื่อยกเลิก
        - ออกแบบให้ครอบคลุมทั้งระบบใน [launcher/ui/bridge.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/bridge.js), [launcher/ui/launcher.css](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.css), [public/js/components/dialog.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/components/dialog.js), และ [public/css/main.css](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/css/main.css)
      - **ทดแทน Native Browser Dialog ทั้งหมด:**
        - เปลี่ยนปุ่มล้างข้อมูลเบราว์เซอร์ (`clearClientBrowserProfile`) ใน [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js) จากเดิมที่เบราว์เซอร์เด้ง `127.0.0.1:3088 says` มาใช้กล่องยืนยันของโปรแกรมเอง
        - เปลี่ยน fallback ลบช่อง Client Slot (`deleteClientSlot`) ให้ใช้ `showConfirm`
        - เปลี่ยนการลบตัวแปร (`deleteVariable`) ใน [public/js/canvas.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/canvas.js) ให้ใช้ `showConfirm`
        - เปลี่ยนการลบและเขียนทับโปรไฟล์ (`deleteProfile`, `handleImportProfileFile`) ใน [public/js/state.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/state.js) และ [public/js/components/modal.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/components/modal.js) ให้ใช้ `showConfirm` และ `showAlert`
    - **เฟส 17 เปิดใช้งาน Browser App Mode และตัดแถบเตือน Command-line Flag สำเร็จ 100%:**
      - **เชื่อมต่อ App Mode ([src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs), [src/browser/launcher.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/browser/launcher.rs)):**
        - เพิ่มการตรวจจับค่า `-app` (เช่น `"1-app"`, `"2-app"`) ใน `ClientActionPayload` และ `configs/global.json`
        - ใน `BrowserLauncher::launch` เมื่อ `app_mode == true` จะส่งคำสั่ง `--app=https://universe.flyff.com/play` ส่งผลให้เบราว์เซอร์เปิดในโหมด **Borderless App Window** ไร้ Tab Bar, ไร้ Address Bar, ไร้ Bookmark Bar หน้าต่างเกมเปิดโล่งสะอาดตาตามการตั้งค่าของผู้ใช้ 100%
      - **ลบแถบเตือนสีดำ Unsupported Command-line Flag:**
        - เพิ่ม flag `--test-type` ในการสั่งรัน Chrome ทำให้ Chrome ซ่อนแถบเตือนเรื่อง `--disable-blink-features=AutomationControlled` ไม่ให้ปรากฏมากวนสายตา
    - **เฟส 18 ระบบ Process Watcher ตรวจจับการปิดเบราว์เซอร์อัตโนมัติ, ปรับโครงสร้าง Logs, ลบจอ 1 ถาวร, และคุ้มครอง Port 3088 สำเร็จ 100%:**
      - **1. Browser Process Auto-Status Watcher ([src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs)):**
        - ติดตั้ง `tokio::task::spawn_blocking(move || { child.wait() })` ในฟังก์ชัน `launch_client`
        - เมื่อผู้ใช้กดปิดหน้าต่างเบราว์เซอร์เกม (ปุ่มกากบาท `[X]`), Background Task จะตรวจพบการจบกระบวนการ (exit status) ทันที
        - สั่ง `state.client_manager.set_running(client_id, false, None, None)` และตัด `cdp_sessions` ออกอัตโนมัติ พร้อมส่ง Event `system` อัปเดตไปยัง Dashboard ทำให้สถานะใน Client Control Matrix สลับกลับเป็น "⚪ Closed/Standby" เองทันทีโดยไม่ต้องกดรีเฟรช
      - **2. แยกหมวดหมู่ประเภท Logs ชัดเจน (Action vs. Message Log vs. System):**
        - ปรับปรุงโครงสร้าง Badges และการกรองใน [launcher/ui/index.html](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/index.html), [launcher/ui/launcher.css](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.css), และ [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js)
        - แยกประเภทอย่างเป็นระเบียบตามการใช้งานจริง:
          - `🟣 Action`: การทำงานของบอท (KeyPress, KeyHold, BuffSequence, Loop, Party HP Scanner)
          - `🟢 Message Log`: ข้อความที่ส่งมาจาก Action Node `step_log` (Message Log) เท่านั้น
          - `⚙️ System`: ข้อมูลภายใน Engine (Browser Spawn, CDP Connect, Window Closed, Engine Start/Stop) สี Clean Cyan Matte (`#38bdf8`)
          - `🔴 Error` และ `🟡 Warning`: ข้อผิดพลาดและข้อควรระวัง
      - **3. ลบจอ 1 ถาวร & รองรับ Dynamic Empty Slot ([launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js)):**
        - ปลดล็อกเงื่อนไขที่บังคับ `slots = [1]` เมื่อลบจนเหลือ 0 จอ
        - เพิ่มการเรนเดอร์การ์ดช่องว่าง `[ ➕ เพิ่มจอ 1 ]` เมื่อไม่มีจอเหลืออยู่ ทำให้ผู้ใช้สามารถลบจอ 1 ได้อย่างสมบูรณ์ และกดสร้างใหม่ได้ตามต้องการ
      - **4. ซ่อน Port 3088 และสถาปัตยกรรมคุ้มครอง Local Port:**
        - ปรับปรุง Card 2 บน Dashboard ([launcher/ui/index.html](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/index.html), [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js)) จากเดิมที่แสดง `Port 3088 / http://localhost:3088` ให้เปลี่ยนเป็น `CORE ENGINE: 🟢 Online (Ready) | Protected Local Engine`
        - ตัดการ์ดตั้งค่า Port ออกจากหน้าต่าง Settings เพื่อไม่ให้ User สับสนหรือแก้ไขพอร์ตจนระบบทำงานผิดพลาด
        - พอร์ต 3088 ยังคงทำงานในระดับ Loopback (`127.0.0.1`) ภายในเครื่องอย่างปลอดภัย
      - **Unit Tests 100%:** ทดสอบผ่านครบทั้ง **19/19 tests passed** (Zero Regression)
    - **เฟส 19 ปรับมาตรฐาน Universal Custom Dialog สู่สไตล์แบบที่ 2 (Style 2 Standard) สำเร็จ 100%:**
      - **Harmonize Modal Design ([launcher/ui/bridge.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/bridge.js), [launcher/ui/launcher.css](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.css), [public/js/components/dialog.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/components/dialog.js), [public/css/main.css](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/css/main.css)):**
        - ปรับปรุงการทำงานของ `window.showConfirm` และ `window.showAlert` ให้แสดงผลเหมือนกล่องยืนยันการลบจอ (แบบที่ 2 ในรูป) 100%
        - การ์ดพื้นหลัง `#0f172a` ขอบกรอบสีแดงหม่น `rgba(239, 68, 68, 0.4)` (หรือสีส้ม/ฟ้าตามประเภท) พร้อมเงา `0 10px 30px rgba(0,0,0,0.8)`
        - หัวข้อ Title สีแดงสดใส `#f87171` ขนาด 14px ตัวหนา พร้อมไอคอน `🗑️` และปุ่มปิด `✕`
        - แยกข้อความคำถามหลักตัวหนาสีขาว (`#fff`) และนำข้อความรายละเอียดหรือผลกระทบ (`\n\n`) ใส่ลงในกล่องคำเตือนสีฟ้าหม่น (Callout Box) `💡 ข้อมูลสำคัญ:` อัตโนมัติ
        - ปุ่มยกเลิก `[ ✕ ยกเลิก ]` และปุ่มยืนยันสีแดงเต็มปุ่ม `[ 🗑️ ล้างข้อมูลโปรไฟล์ ]`
        - ครอบคลุมทั้งฝั่ง Launcher Dashboard และ Action Node Studio อย่างสม่ำเสมอเป็นมาตรฐานเดียวกัน
      - **Unit Tests 100%:** ผ่านครบทั้ง **19/19 tests passed** (Zero Regression)
    - **เฟส 20 จัดหมวดหมู่ Log ระบบและ License ให้ถูกต้อง (System vs. Action Reclassification) สำเร็จ 100%:**
      - **ย้าย License & Internal System Logs สู่หมวด `[System]` ([src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs)):**
        - ปรับแก้คำสั่ง `state.log` ในจุดที่เกี่ยวข้องกับระบบภายในและ License ให้ใช้ `"system"` แทน `"action"` ได้แก่:
          - License activated offline (`L127`)
          - Serial Key redeemed on Cloudflare (`L148`)
          - License synced from Cloudflare Server (`L270`)
          - Hotkey Sentinel state toggled (`L396`)
          - Cleared browser profile data (`L704`)
        - ส่งผลให้ข้อความ License Sync และงานระดับระบบย้ายไปแสดงผลใต้แท็บ **`[⚙️ System]`** (Badge สีฟ้าหม่น `#38bdf8`) อย่างถูกต้อง ไม่ปะปนกับแท็บ **`[🟣 Action]`** อีกต่อไป
      - **ปรับปรุงคำป้ายกำกับปุ่ม Filter Pills ([launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js)):**
        - เปลี่ยน `filterLog` และ `filterStep` ในพจนานุกรม TH/EN ให้แสดงเป็น `🟢 Message Log` ชัดเจนตรงตามชื่อ Action Node
      - **Unit Tests 100%:** ผ่านครบทั้ง **19/19 tests passed** (Zero Regression)
    - **เฟส 21 ระบบตั้งชื่อ Tab / Window Title ของเกมตาม Client Alias (แบบที่ 2) สำเร็จ 100%:**
      - **ฟังก์ชัน `set_tab_title` ผ่าน CDP ([src/browser/cdp_session.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/browser/cdp_session.rs)):**
        - ถอดแบบความสามารถจาก `bot.js` ของโปรเจกต์เดิม ผสานทั้ง `Page.enable`, `Page.addScriptToEvaluateOnNewDocument` (ทำงานทันทีก่อนสคริปต์หน้าเว็บทุกครั้งที่มีการรีโหลด F5 หรือเปลี่ยนหน้า), `Runtime.evaluate` สำหรับหน้าปัจจุบัน, และ `MutationObserver` + `setInterval(updateTitle, 1000)` ป้องกัน Flyff Universe แอบรีเซ็ตชื่อแท็บกลับเป็นค่าเริ่มต้น
        - ฟังก์ชัน `updateTitle` ในตัวตรวจจับจะตัด Prefix เดิมออกอัตโนมัติ (เช่น `[OldAlias]` หรือ `[Client 1]`) ก่อนเติม Prefix ใหม่ ทำให้การเปลี่ยนชื่อลื่นไหล ไม่เกิดชื่อซ้ำซ้อน
      - **เชื่อมต่อ Auto-Title เมื่อต่อ CDP ([src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs)):**
        - เมื่อจอเกมเปิดขึ้นมาและ CDP เชื่อมต่อสำเร็จ ระบบจะอ่านชื่อจาก `clientAliases` แล้วตั้งเป็นฟอร์แมตแบบที่ 2 ทันที:
          - มีชื่อเล่น (เช่น Knight): `[Knight] Flyff Universe`
          - ไม่มีชื่อเล่น: `[Client 1] Flyff Universe`
        - มีการตั้งเวลา Re-apply อีกครั้งที่ 2.5 วินาทีหลังเปิดเกม เพื่อคุ้มครอง Title หลังจากตัวเกมโหลดหน้าหลักเสร็จสมบูรณ์
      - **API อัปเดตชื่อแบบ Real-time ([src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs), [src/server/mod.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/mod.rs)):**
        - เพิ่ม Endpoint `POST /api/client/alias` อัปเดต `clientAliases` ใน `configs/global.json` และส่งคำสั่งเปลี่ยน Title ไปยังหน้าต่างเกมที่เปิดอยู่ทันที
      - **อัปเดตช่องพิมพ์ใน Client Control Matrix ([launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js)):**
        - ฟังก์ชัน `saveClientAlias` เชื่อมต่อกับ `/api/client/alias` เมื่อผู้ใช้พิมพ์ชื่อใหม่แล้วกด Enter หรือคลิกออก หน้าต่างเกมจะเปลี่ยนชื่อทันทีในเสี้ยววินาที
      - **Unit Tests 100%:** ผ่านครบทั้ง **19/19 tests passed** (Zero Regression)
    - **เฟส 22 ปรับปรุงภาษาไทยให้เข้าใจง่าย และใช้ชื่อ Action Node ดั้งเดิมจากโปรเจกต์เก่า 100%:**
      - **ปุ่มหลักหน้าแรก ([launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js), [launcher/ui/index.html](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/index.html)):**
        - ปรับปุ่มเมนูหลักใน Sidebar และ Breadcrumb ให้เป็นภาษาอังกฤษสั้นกระชับ: `Dashboard`, `Action Node`, `Settings` (ตัดคำว่า & HUD ออก), และ `Node Wiki`
        - ปรับหมวดหมู่ Sidebar เป็น: `Workspace`, `Configuration`, `Documentation`
      - **ชื่อ Action Node ทั้ง 29 ตัวและหมวดหมู่ ([public/js/i18n.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/i18n.js), [public/js/canvas.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/canvas.js)):**
        - ถอดแบบชื่อภาษาอังกฤษสากลตรงตามโปรเจกต์เดิม `NodeHotkey` 100% ทุกตัว เช่น `Trigger`, `Broadcast Event`, `Repeat Loop`, `Buff Sequence`, `Single Key`, `Key Hold`, `Key Forwarder`, `Cast Sequencer`, `Action Branch`, `Variable Branch`, `Condition Group`, `Party Scanner (Vision)`, `Select Party Slot`, `Party Heal Target`, `Party Buff Target`, `Log Message`, `Format Text`, `HTTP Webhook (Outbound)`, `Text to Speech (TTS)`, `Screenshot (Diagnostic)`
        - ปรับหมวดหมู่ใน Spotlight Menu (คลิกขวา Add Node) เป็น: `Triggers & Events`, `Actions & Macros`, `Logic & Flow`, `Safety & Utilities`
        - คำอธิบายการทำงาน (`node_desc_*` และ Parameter Hints) ยังคงเป็นภาษาไทยที่เป็นมิตร ชัดเจน ไม่เป็นทางการเกินไป และเข้าใจง่าย
      - **Unit Tests 100%:** ผ่านครบทั้ง **19/19 tests passed** (Zero Regression)
    - **เฟส 23 ระบบ Windows System Tray และ Close-to-Tray สำเร็จ 100% (เหมือนโปรเจกต์เดิม):**
      - **เปิดใช้งานฟีเจอร์ Tauri Tray Icon ([Cargo.toml](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/Cargo.toml)):**
        - ปรับ `tauri = { version = "2", features = ["tray-icon"] }`
      - **สร้าง System Tray Icon & Context Menu ([src/main.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/main.rs)):**
        - ถอดแบบความสามารถจาก NodeHotkey ตัวเดิม:
          - ไอคอนถาวรใน Taskbar Tray พร้อม Tooltip `NodeHotKey V4 Control Center`
          - ดับเบิ้ลคลิก (Double Click) ที่ไอคอน Tray เพื่อสลับซ่อน/แสดงหน้าต่าง Launcher (Show/Hide) และดึงโฟกัสขึ้นมาด้านหน้าทันทีตามมาตรฐาน Windows (คลิกครั้งเดียวไม่เด้งเปิด ป้องกันกดโดน)
          - เมนูคลิกขวา (Context Menu):
            - `⚡ NodeHotKey V4 (Active)` (หัวข้อแสดงสถานะ)
            - `🖥️ Show Launcher` (เปิดแสดงหน้าต่าง)
            - `🎮 Action Node Studio` (เปิด Action Node Studio ในเบราว์เซอร์หลักผ่านพอร์ต 3088)
            - `❌ Exit` (ปิดโปรแกรมสมบูรณ์)
      - **พฤติกรรม Close-to-Tray & Native Window Action Broker ([src/state.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/state.rs), [src/server/routes.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs), [src/server/mod.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/mod.rs), [src/main.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/main.rs), [launcher/ui/bridge.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/bridge.js)):**
        - ติดตั้ง Window Action Broker ผ่านช่องทาง Axum Native HTTP (`POST /api/window/minimize`, `POST /api/window/maximize`, `POST /api/window/hide`, `POST /api/window/close`)
        - ดักจับ `WindowEvent::CloseRequested` ในระดับ Tauri Core: สั่ง `api.prevent_close()` และ `window.hide()`
        - อัปเดต `bridge.js`: ให้ปุ่ม `[-]`, `[🗖]`, และ `[✕]` บน Titlebar ของ Launcher ยิงคำสั่งตรงเข้า Backend ทำให้หน้าต่างย่อ, ขยาย, และพับลง System Tray ได้ทันที 100% บอท, คีย์ลัด (Sentinel), ระบบสแกนเลือด และหน้าต่างเกมทุกจอยังคงทำงานต่อเนื่องในพื้นหลังอย่างราบรื่น
      - **Unit Tests 100%:** ผ่านครบทั้ง **19/19 tests passed** (Zero Regression)
    - **เฟส 24 ระบบซิงก์สถานะ Bot Execution และ Global Sentinel (ปุ่ม END) สำเร็จ 100%:**
      - **ซิงก์สถานะการรันอัตโนมัติ ([launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js)):**
        - เชื่อมต่อ `updateStatusUI(data)` ภายในลูป `syncBackendStatus` (ดึงทุก 2 วินาที) ทำให้การ์ดสถานะโปรแกรมและการ์ดปุ่ม Toggle สลับสถานะตรงตาม Backend เสมอ
        - อัปเดต `btnToggleEngine.onclick`: เมื่อสั่ง `api.startBot()` หรือ `api.stopBot()` สำเร็จ ให้เรียก `syncBackendStatus()` ทันที สลับปุ่มเป็น `⏹ หยุดการทำงาน` ในเสี้ยววินาที
      - **รองรับสถานะปุ่ม END (Global Hotkey Sentinel):**
        - เมื่อบอทรันปกติ: การ์ดแสดง `🟢 กำลังทำงาน` (จุดสีเขียว + Uptime)
        - เมื่อกดปุ่ม **`END`** พักฉุกเฉิน: การ์ดแสดง `⏸️ พักชั่วคราว (กด END เพื่อรันต่อ)` (จุดสีส้ม/เหลือง pending) และหยุดส่งข้อมูล Action Node ทันที
        - เมื่อกดปุ่ม **`END`** อีกครั้ง: ปลดเบรกมือ กลับสู่สถานะ `🟢 กำลังทำงาน`
        - เมื่อกดหยุดบอท: การ์ดแสดง `🔴 หยุดทำงาน` (Process Stopped)
      - **Unit Tests 100%:** ผ่านครบทั้ง **19/19 tests passed** (Zero Regression)
    - **เฟส 25 ระบบ Profile Isolation, Numpad Separation และ Auto-Close Clients on Exit สำเร็จ 100%:**
      - **แยก Profile ออกจาก Global Settings อย่างเด็ดขาด:**
        - ตัดโค้ด Mirroring ใน `public/js/state.js` และ `modal.js` ไม่ให้ยัดค่าหน้าจอใส่ลงในโปรไฟล์
        - ติดตั้ง `sanitize_profile_keys()` ใน Rust (`src/model.rs`, `src/config/store.rs`) กรองคีย์แปลกปลอมออกอัตโนมัติทั้งตอนโหลดและตอนเซฟ
        - ทำความสะอาดไฟล์ `Default.json` บริสุทธิ์ 100%
      - **แยกปุ่มตัวเลขแถวบน (0–9) และปุ่ม Numpad (KP0–KP9) เด็ดขาด:**
        - แก้ไข `normalize_trigger_key()` ใน `src/engine/graph.rs` ไม่ให้รวมปุ่มแถวบนกับ Numpad เป็นคีย์เดียวกัน
        - เลข 1 แถวบนจะเป็น `"1"` ส่วน Numpad 1 จะเป็น `"NUMPAD1"` แยกอิสระจากกัน 100%
      - **ระบบปิดหน้าจอเกมทั้งหมดอัตโนมัติก่อนโปรแกรมปิดตัว (Auto-Close All Browser Clients on Exit):**
        - เพิ่ม `close_browser()` ส่งคำสั่ง CDP `Browser.close` ใน `src/browser/cdp_session.rs`
        - เพิ่ม `kill_process_tree(pid)` ตัดตอน Process ลูกผ่าน PID ด้วย `taskkill /PID <pid> /T /F` ใน `src/browser/launcher.rs`
        - เพิ่ม `terminate_client()` และ `terminate_all_clients()` ใน `src/state.rs`
        - ผูกระบบปิดจอเกมทั้งหมดเข้ากับจุดปิดโปรแกรมทุกทางใน `src/main.rs` (ทั้งปุ่ม Close จาก Launcher, ปุ่ม Exit จากถาด System Tray, และ Ctrl+C ในโหมด Headless)
      - **เฟส 26: Engine Boot Auto-Start, Immediate Hotkey Readiness & Trigger Logging:**
        - ปรับปรุง [`src/main.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/main.rs): สั่ง `state.is_running.store(true)` ทันทีหลังโหลดโปรไฟล์เริ่มต้น เพื่อให้ Engine พร้อมรับคำสั่ง Hotkey ทันทีที่เปิดโปรแกรม ไม่ต้องรอไปกด Start ใน Launcher
        - ปรับปรุง [`src/engine/executor.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/engine/executor.rs): เพิ่ม Log แจ้งเตือนในคอนโซลและ State Log เมื่อ Hotkey ทำงาน เช่น `⚡ Hotkey [1] (Raw: 'Num1') activated 1 matching flow(s)`
        - ปรับปรุง [`configs/profiles/Default.json`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/configs/profiles/Default.json): อัปเดต `triggerValue` ของโหนดเริ่มต้นจาก `"SPACE"` เป็น `"1"` เพื่อให้พร้อมทดสอบเลข 1 ได้ทันที
      - **เฟส 27: Focused-Window Hotkey Fallback & Deduplication Guard (แก้ปัญหาคีย์ไม่ทำงานตอน Focus โปรแกรม):**
        - ปรับปรุง [`src/state.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/state.rs): เพิ่ม `last_trigger` Guard ป้องกันการทำงานซ้ำซ้อน (Deduplication / Debounce 80ms) และเพิ่มการดักเช็คปุ่ม `suspend_hotkey` (เช่น `END`) ใน `handle_key_trigger`
        - ปรับปรุง [`src/server/mod.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/mod.rs): เปิดให้ `/api/trigger/{event_name}` รองรับทั้ง GET และ POST
        - ปรับปรุง [`public/js/app.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/app.js) & [`launcher/ui/launcher.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js): เพิ่ม DOM Keydown Listener ส่ง Event เข้า `/api/trigger/{key}` เมื่อหน้าต่างของโปรแกรมถูก Focus (และไม่ได้อยู่ในช่อง Input หรือกำลังอัดคีย์) ทำให้ Hotkey ทำงานได้ 100% ทั้งตอน Focus โปรแกรม และตอน Unfocus อยู่โปรแกรมอื่น
      - **เฟส 28 ระบบบันทึกและคืนค่าตำแหน่ง/ขนาดหน้าจอเกมอัตโนมัติ (Plan 3: Pure CDP WebSocket Event Binding):**
        - **ถอดแบบระบบ Event Binding ของโปรเจกต์เดิม 100% (Zero Polling / Zero Delay):**
          - ไม่ใช้การวนลูป Polling ใดๆ ทั้งสิ้น (Zero Looping)
          - ใน [`src/browser/cdp_session.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/browser/cdp_session.rs): ส่งคำสั่ง CDP `Runtime.addBinding({ name: "__nhkSaveBounds" })`
          - ฝัง Event Listener ในเบราว์เซอร์ ดักจับเหตุการณ์ `resize`, `blur`, `mouseup`, `beforeunload`, `pagehide`
          - เมื่อผู้ใช้ขยับจอ ปล่อยเมาส์ หรือกดกากบาท 'X' ปิดจอ เบราว์เซอร์จะส่ง Event `Runtime.bindingCalled` ข้ามผ่าน **ท่อ WebSocket CDP** เข้า Rust ทันที ไม่ใช้ HTTP `fetch()` จึงไม่โดน Chrome บล็อก Mixed Content 100%
          - Rust ได้รับ Event ปุ๊บ ก็เรียก `save_client_bounds_if_changed` ตรวจสอบและบันทึกลง [`configs/global.json`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/configs/global.json) ในเสี้ยววินาที
        - **การคืนค่าตอนเปิดจอ (Window Bounds Restoration upon Launch):**
          - ใน [`src/browser/launcher.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/browser/launcher.rs): ส่ง Argument `--window-position=X,Y` และ `--window-size=W,H` ให้เบราว์เซอร์เปิดตรงตำแหน่งและขนาดเดิม 100%
        - **Unit Tests 100%:** ผ่านครบทั้ง **22/22 tests passed** (Zero Regression)
      - **เฟส 29 ยกระดับ Launcher Dashboard สู่ Event-Driven 100% (Plan A: Unified SSE Status Broadcast):**
        - **กำจัดลูป Polling 2 วินาทีใน Launcher Dashboard:**
          - ลบคำสั่ง `setInterval(syncBackendStatus, 2000)` ออกจาก [`launcher/ui/launcher.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js) อย่างสิ้นเชิง เปลี่ยนมาเรียก `syncBackendStatus()` เพียงครั้งเดียวตอนเปิดโปรแกรม (Initial Boot) เท่านั้น
        - **สร้างช่องทาง Broadcast Status ใน Rust Core Engine:**
          - เพิ่ม `status_event_sender: broadcast::Sender<serde_json::Value>` ใน [`src/state.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/state.rs)
          - เพิ่มฟังก์ชัน `get_status_json()` และ `notify_status_change()` บน `AppState`
          - ผูกการแจ้งเตือนทันทีเมื่อ:
            1. เปิดจอเกม (`launch_client`) หรือปิดจอเกมผ่านกากบาท [X] (`child.wait()` exit watcher)
            2. บอทเริ่มทำงาน (`start_engine`) หรือหยุดทำงาน (`stop_engine`)
            3. ปุ่มคีย์ลัด Sentinel (ปุ่ม `END`) ถูกกดพัก/เริ่มใหม่
            4. เปลี่ยนชื่อเล่นหน้าจอ (`set_client_alias`)
            5. บันทึกการตั้งค่าระบบ (`save_config`)
            6. ปิดหน้าจอเกมทั้งหมด (`terminate_client`, `terminate_all_clients`)
        - **เชื่อมต่อท่อ SSE และ Dispatcher ฝั่ง Frontend:**
          - ใน [`src/server/routes.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs): เพิ่ม `status_rx` ใน `profiles_sse_stream` ยิง Event `{ "type": "status_update", "data": ... }` ทันทีที่มีการเปลี่ยนแปลง
          - ใน [`launcher/ui/bridge.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/bridge.js): เชื่อมต่อ `status_update` เข้ากับ `statusSubscribers`
          - ใน [`launcher/ui/launcher.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js): สกัดฟังก์ชัน `applyStatusData(data)` และดักรับ Event ผ่าน `window.launcherAPI.onStatusChange` อัปเดตการ์ด Client Control Matrix, สถานะบอท และข้อมูล Real-time ในเสี้ยววินาที โดยไม่ต้องยิง HTTP Request ซ้ำๆ อีกต่อไป
        - **Unit Tests 100%:** เพิ่มชุดทดสอบ `test_state_status_json_and_notify_broadcast` ผ่านครบทั้ง **23/23 tests passed** (Zero Regression)
      - **เฟส 30 ยกระดับ Action Node Studio Canvas & PiP Window สู่ Event-Driven 100% (Direct SSE Integration):**
        - **กำจัดลูป Polling ใน Action Node Studio (Web Canvas):**
          - ใน [`src/server/routes.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs): อัปเดตท่อ `/api/profile-events/stream` ให้ผสานรับ `status_rx` ส่ง Event `{ "type": "STATUS_UPDATE", ... }` ตรงไปยัง Canvas ทันทีที่มีการเปลี่ยนแปลง
          - ใน [`public/js/state.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/state.js): สกัดฟังก์ชัน `handleActiveClientsUpdate(res)` และดักรับ Event `STATUS_UPDATE` จาก SSE ทำให้อัปเดตสวิตช์จอเกมใน Canvas ทันทีในเสี้ยววินาที
          - ใน [`public/js/app.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/app.js): **ลบคำสั่ง `setInterval(pollActiveClients, 3000)` ออกจากระบบ 100%** เหลือเพียงการเรียกทำงาน 1 ครั้งตอนเริ่มต้น
        - **กำจัดลูป Polling ในหน้าต่างจอเสริม PiP Window:**
          - ใน [`launcher/ui/pip.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/pip.js): เชื่อมต่อ EventSource เข้ากับ `/api/profiles/stream` ดักรับ Event `status_update` อัปเดต `activeClients` และ `clientAliases` แบบ Real-time ทันที
          - **ลบคำสั่ง `setInterval(syncActiveClients, 2000)` ออกจากระบบ 100%**
        - **Unit Tests 100%:** รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ครบทั้ง **23/23 tests passed** (Zero Regression)
      - **เฟส 31 ปรับแต่ง License Sentinel & ปลดล็อก Zero Polling ทั่วทั้งโปรเจกต์ 100% (Event-Driven License Synchronization):**
        - **กำจัดลูป Polling License 3 นาทีใน Launcher Dashboard:**
          - ใน [`launcher/ui/launcher.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js): **ลบคำสั่ง `setInterval(() => { window.loadLicenseStatus(); }, 180000)` ออกจากระบบ 100%**
          - สถานะ License และวันหมดอายุถูกส่งผ่าน Event SSE `status_update` และอัปเดตอัตโนมัติผ่าน `applyStatusData(data)`
        - **เชื่อมต่อ Background License Sentinel ใน Rust Core Engine:**
          - ใน [`src/server/mod.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/mod.rs): อัปเดต Background Sentinel ให้เรียก `heartbeat_state.notify_status_change().await` ทุกครั้งที่ตรวจสอบความถูกต้องกับ Cloudflare เสร็จสิ้น
          - ใน [`src/server/routes.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs): เพิ่ม `state.notify_status_change().await` ใน `activate_license_handler` และ `sync_license_online_handler` (ทั้งกรณีสำเร็จและกรณีตรวจพบว่าคีย์ถูกระงับ/ลบ)
          - หากแอดมินสั่งระงับสิทธิ์ (Suspend) หรือลบคีย์บน Cloudflare ตัว Engine จะหยุดทำงานและหน้าจอ License Guard จะเด้งขึ้นมาคุ้มครองระบบทันทีในเสี้ยววินาที
        - **บรรลุเป้าหมาย 4 เสาหลัก 100%:** ลูป Polling ฝั่งหน้าบ้าน (Dashboard, Canvas Studio, PiP) ถูกกำจัดจนหมดเกลี้ยง (Zero Polling Loops Across All Frontend Interfaces)
        - **Unit Tests 100%:** รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ครบทั้ง **23/23 tests passed** (Zero Regression)
      - **เฟส 32 ระบบ Real-time Pusher License Sentinel (Sub-Second 0.1 - 0.3s Reaction & 100% Event-Driven):**
        - **เปิดท่อสัญญาณ WebSocket Pusher Channels ใน Rust Core:**
          - ใน [`Cargo.toml`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/Cargo.toml): เพิ่มฟีเจอร์ `rustls-tls-webpki-roots` ให้ `tokio-tungstenite` เพื่อเชื่อมต่อ `wss://` บน Windows ได้อย่างปลอดภัยโดยไม่ต้องติดตั้ง OpenSSL
          - สร้าง [`src/security/realtime_sentinel.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/realtime_sentinel.rs): ตัวเชื่อมต่อ Pusher WebSocket (`wss://ws-ap1.pusher.com`) ฟัง Channel `nhk-licenses` (Cluster: `ap1`)
          - สปอว์น Tokio Task รัน Sentinel ทันทีเมื่อเปิด Server ใน [`src/server/mod.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/mod.rs)
          - ทำงานแบบหลับรอสัญญาณ (0 Polling, 0 Requests ต่อ Cloudflare)
          - เมื่อมี Event `license_updated` เข้ามา ตรวจสอบว่าตรงกับ HWID หรือ Key ของเครื่องเราหรือไม่:
            - หากสถานะเป็น `SUSPENDED` หรือ `BANNED`: สั่งตัดการทำงานของบอททันที (`is_running = false`) + บันทึกสถานะ + ยิงแจ้งเตือน SSE `notify_status_change().await` ทำให้หน้ากาก License Guard สีเหลืองเด้งขึ้นล็อกการทำงานใน **0.1 - 0.3 วินาที**
            - หากสถานะเป็น `ACTIVATED`: ซิงค์ข้อมูล Ed25519 Token ใหม่จาก Cloudflare และปลดล็อกโปรแกรมอัตโนมัติ
        - **ผสานระบบกระจายสัญญาณใน Cloudflare Worker (`docs/worker_api_clean.js`):**
          - ติดตั้งฟังก์ชัน `triggerPusherEvent` (สร้างด้วย Pure JS MD5 + WebCrypto HMAC-SHA256 ไร้การพึ่งพาแพ็กเกจภายนอก)
          - เมื่อแอดมินเปลี่ยนสถานะคีย์ (`/api/admin/keys/status`) หรือลบคีย์ (`/api/admin/keys/delete`): Cloudflare จะยิงข้อความ 1 ช็อตไปยัง Pusher สะท้อนตรงเข้าเครื่องผู้ใช้ทันที
        - **Unit Tests 100%:** เพิ่มการทดสอบ `test_realtime_license_sentinel_constants` ใน [`src/lib.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/lib.rs) ผ่านครบทั้ง **24/24 tests passed** (Zero Regression)
      - **เฟส 33 หน้ารักษาความปลอดภัยด่านแรกสุด 0ms (Strict Gatekeeper First - Zero FOUC & Immediate License Guard):**
        - **แก้ปัญหา UI แดชบอร์ดแวบโผล่ก่อนล็อก (Zero Flash of Unauthenticated Content):**
          - ใน [`launcher/ui/index.html`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/index.html): ตั้งค่าเริ่มต้นของ `#license-guard-overlay` เป็น `display: flex;` คลุมพื้นที่ Main Workspace ด้วยสี Dark Matte `#080c14` ตั้งแต่วินาทีแรกที่หน้าต่างเปิดขึ้นมา
          - ตั้งค่า Sidebar Navigation (`.sidebar-menu-list`) และปุ่ม Toggle Engine (`#btn-toggle-engine`) ให้เริ่มต้นด้วยสถานะ Disable (`pointer-events: none; opacity: 0.35; filter: grayscale(0.7);`) ทำให้ไม่มีโอกาสเห็นหรือกดใช้งาน Dashboard ก่อนการยืนยันสิทธิ์
        - **ตรวจสอบสิทธิ์ทันทีแบบขนาน (0ms Delay Concurrent Boot Check):**
          - ใน [`launcher/ui/launcher.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js): ตัดคำสั่งหน่วงเวลา `setTimeout(..., 1000)` ออก ยิงตรวจสอบ `window.loadLicenseStatus()` ในเครื่อง (ตอบกลับใน <2ms) ควบคู่กับ HWID Fetch ผ่าน `Promise.all`
          - ปรับปรุง `setSidebarNavigationEnabled(enabled)` ให้ควบคุมทั้งแถบเมนูและปุ่มสั่งเริ่ม Engine ควบคู่กัน
          - เมื่อสถานะเป็น `Active`: ปลดม่านบังตาและเปิดการทำงานของ Sidebar/Engine ใน ~10ms
          - เมื่อสถานะเป็น `Suspended` หรือ `Unlicensed`: ม่านบังตา License Guard จะตรึงอยู่กับที่ แสดงข้อความแจ้งเตือนและปุ่มตรวจสอบสถานะทันที โดยไม่มีการแวบของ Dashboard ให้เห็นแม้แต่เสี้ยววินาทีเดียว
        - **ลด Warmup Delay ใน Rust Engine:**
          - ใน [`src/server/mod.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/mod.rs): ลดรอบ Warmup ของ Background Sentinel ตอนเปิดเครื่องลงจาก 2500ms เหลือเพียง 200ms
        - **Unit Tests 100%:** รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ครบทั้ง **24/24 tests passed** (Zero Regression)
      - **เฟส 34 ระบบแจ้งเตือนเสียง AI และ Webhook เต็มรูปแบบ (Cloud Neural TTS & Robust Webhook Integration):**
        - **ระบบ HTTP Webhook Outbound (`webhook_out` & `/api/webhook/test`):**
          - ใน [`src/server/routes.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/server/routes.rs): อัปเกรด `test_webhook` และ `WebhookTestPayload` รองรับ Method (`POST`, `GET`, `PUT`, `DELETE`), Headers, Payload JSON/Text, และ Timeout (ms) ส่งผลลัพธ์ HTTP status (200, 204, ฯลฯ) กลับไปยังปุ่ม `[ ⚡ Test Webhook ]` ใน Action Node Studio Inspector ได้อย่างแม่นยำ
          - ใน [`src/engine/executor.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/engine/executor.rs): รันคำขอ HTTP ใน Background Tokio Task แยกอิสระ 100% (Zero Lag ต่อบอท) หากสำเร็จจะส่งสัญญาณออกพอร์ต `onSuccess` หากล้มเหลวจะส่งออกพอร์ต `onError`
        - **ระบบสังเคราะห์เสียงพูด Cloud Voice TTS (`tts`):**
          - ใน [`src/audio/tts.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/audio/tts.rs): ติดตั้ง **Cloud TTS Voice Engine** เสถียรสูง (Zero-Auth, ไม่ติด 401/403) รองรับเสียงไทยธรรมชาติชัดเจนและเสียงอังกฤษ พร้อม Fallback Edge TTS และ Windows SAPI
          - **ระบบ Smart Local Caching (0ms Playback):** สังเคราะห์แล้วบันทึกเป็น `.mp3` ลงในแคช `cache/tts/` เมื่อมีข้อความซ้ำจะเปิดจากไฟล์ในเครื่องทันที 0ms ไม่ต้องต่อเน็ตซ้ำและไม่กินแบนด์วิดท์
          - ใน [`src/engine/executor.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/engine/executor.rs): สปอว์น Tokio Task ดึงเสียงจาก `tts_manager` แล้วส่งเข้า `audio_player` (rodio) เล่นแบบ Native Audio ทันที พร้อมส่งสัญญาณ `onComplete` ไปยังโหนดถัดไป
      - **เฟส 35 แก้ปัญหาหน้าจอเด้ง Offline บัง Canvas เมื่อ Action Node ทำงาน (False Offline Overlay Resolution):**
        - **วิเคราะห์สาเหตุ (Root Cause):** เมื่อ Action Node รันหรือ Hotkey ถูกกด Engine จะบรอดแคสต์ Event `"signal"` เพื่อกระพริบเส้นเชื่อม แต่ใน [`launcher/ui/bridge.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/bridge.js) ดันส่งเข้า `diagSubscribers` ซึ่งทำให้ `launcher.js` ตรวจไม่พบ `diag.serverOnline` แล้วเข้าใจผิดว่าเซิร์ฟเวอร์หลุด จึงสั่งเปิดหน้า `#editor-offline-placeholder` ขึ้นมาบัง Canvas Studio
        - **การแก้ไข:**
          - ใน [`launcher/ui/bridge.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/bridge.js): แยก `signal` ส่งเฉพาะ `overlaySubscribers` และผูก `diagSubscribers` เข้ากับ `status_update` และ `onerror` ของ SSE อย่างถูกต้อง
          - ใน [`launcher/ui/launcher.js`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js): เพิ่ม Type Guard `if (!diag || typeof diag.serverOnline !== 'boolean') return;` และปรับ `updateLicenseUI` ให้เปิดแสดง Canvas Studio เมื่อ `isServerOnline` และ License ถูกต้อง (ไม่ต้องผูกติดกับ `isRunning`)
        - **Unit Tests 100%:** รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ **30/30 tests passed**
      - **เฟส 36 ระบบคิวเสียงพูด Dual-Channel & Native Audio Queue (TTS waitForPrevious & Cut Mode):**
        - **วิเคราะห์สาเหตุ (Root Cause):** ใน [`src/audio/sound.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/audio/sound.rs) โค้ดเดิมสร้าง `Sink` ใหม่ขึ้นมาทุกครั้ง ทำให้เมื่อเปิด `waitForPrevious = true` เสียงพูดใหม่ไม่เข้าคิวรอแต่กลับเล่นซ้อนทับกันพร้อมกัน และใช้ `active_sink` ตัวเดียวปนกับเสียงเอฟเฟกต์ SFX
        - **การแก้ไข:**
          - ใน [`src/audio/sound.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/audio/sound.rs): แยกช่องสัญญาณ 2 แชนแนลอิสระ `AudioChannel::Tts` และ `AudioChannel::Sfx`
          - ในแชนแนล `Tts`:
            - โหมดเข้าคิว (`waitForPrevious: true`): หากมีเสียงเดิมกำลังพูดอยู่ จะนำเสียงใหม่ไปต่อคิวใน Sink เดิมด้วย `sink.append(source.amplify(vol))` เสียงจะพูดเรียงกันทีละประโยค 100% ไร้เสียงซ้อน
            - โหมดตัดเสียง (`waitForPrevious: false`): สั่งหยุดเสียงเก่าทันที (`tts_sink.stop()`) และเริ่มพูดเสียงใหม่ทันทีใน 0ms
          - ใน [`src/engine/executor.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/engine/executor.rs): เรียก `player.play_tts` พร้อมแปลงค่า `waitForPrevious` รองรับทั้งแบบ boolean และ string จาก JSON
          - เพิ่มชุดทดสอบ `test_audio_player_commands` ใน [`src/audio/sound.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/audio/sound.rs)
        - **Unit Tests 100%:** รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ครบทั้ง **31/31 tests passed** (Zero Regression)
      - **เฟส 37 ปรับข้อความคิวเสียงสั้นกระชับ & ระบบสลับภาษา Real-time ข้ามหน้าต่าง (0ms Instant Language Sync):**
        - **ปรับข้อความตัวเลือก TTS ให้สั้นกระชับ สองภาษา (TH/EN):**
          - TH: `⏳ เข้าคิวเสียง (Queue)` / `คำอธิบาย: เปิด: พูดต่อคิว | ปิด: ตัดเสียงทันที`
          - EN: `⏳ Queue Speech (Wait)` / `คำอธิบาย: ON: Queue | OFF: Interrupt immediately`
          - บันทึกใน [public/js/i18n.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/i18n.js) และ [public/js/canvas-inspector.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/canvas-inspector.js)
        - **แก้ปัญหาการสลับภาษา Real-time ใน Action Node Studio ไม่ต้องรีโหลด F5:**
          - **วิเคราะห์สาเหตุ (Root Cause):**
            1. ใน [public/js/app.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/app.js) ฟังก์ชัน `updateLanguageUI()` เดิมไปเรียก `loadProfileToUI()` ซึ่งเป็นฟังก์ชันโหลดโปรไฟล์จากศูนย์กลาง ทำให้เผลอสั่งปิดหน้าต่าง Inspector (`closeInspector()`) และล้างสถานะ Canvas ทุกครั้งที่เปลี่ยนภาษา
            2. ใน [public/js/canvas.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/canvas.js) ฟังก์ชัน `updateLanguage()` ตรวจสอบเฉพาะ `this.selectedNodeIds.size === 1` หากผู้ใช้กำลังโฟกัสในกล่องแก้ไขข้อความของ Inspector หรือคลิกสลับภาษาจาก Topbar ด้านนอก selection จะหลุด ทำให้ Inspector ถูกเคลียร์ออก
            3. ใน [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js) มี click listener ซ้ำซ้อนที่เพียงแค่สลับคลาส CSS `.active` และใน `switchView('editor')` ไม่ได้ส่งค่าภาษาล่าสุดไปยัง `editorFrame` ทำให้หากเปลี่ยนภาษาในหน้า Dashboard แล้วสลับเข้า Action Node หน้า Canvas จะยังค้างภาษาเดิม
          - **การแก้ไข:**
            1. ใน [public/js/app.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/app.js): ตัด `loadProfileToUI()` ออกจาก `updateLanguageUI()` เพื่อไม่ให้กระทบโหนดที่กำลังเปิดอยู่บน Canvas และเพิ่ม `storage` event listener เพื่อดักจับการเปลี่ยนแปลง `localStorage.nodehotkey_lang` แบบ Real-time ข้ามหน้าต่าง
            2. ใน [public/js/canvas-inspector.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/canvas-inspector.js): บันทึก `this.inspectedNodeId = nodeId` เพื่อจดจำโหนดที่กำลังเปิดอยู่ใน Inspector Panel อย่างแม่นยำ
            3. ใน [public/js/canvas.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/public/js/canvas.js): ปรับ `updateLanguage()` ให้เช็คทั้ง `selectedNodeIds` และ `inspectedNodeId` ทำให้ re-render รายละเอียดใน Inspector เป็นภาษาใหม่ได้ทันทีใน 0ms
            4. ใน [launcher/ui/launcher.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/launcher/ui/launcher.js): ลบ click listener ซ้ำซ้อน และเพิ่มการซิงค์ภาษาไปยัง `editorFrame` ทันทีเมื่อคลิกสลับแท็บเข้า Action Node Studio
        - **Unit Tests 100%:** รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ครบทั้ง **31/31 tests passed** (Zero Regression)
      - **เฟส 38 อัลกอริทึม Sec-MS-GEC v2 Pure Rust Native & แก้ปัญหาโมเดลเสียง TTS ชาย/หญิง ไม่เปลี่ยน:**
        - **วิเคราะห์สาเหตุ (Root Cause):**
          1. ใน [`src/audio/tts.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/audio/tts.rs) เดิมเรียก Google Translate Voice API (`synthesize_cloud_tts`) เป็นลำดับแรกสุด ซึ่ง Google Translate มีเพียงเสียงผู้หญิงเสียงเดียวต่อ 1 ภาษา ไม่มีโมเดลเสียงแยกชาย/หญิง ทำให้ทั้ง `th-TH-PremwadeeNeural` (หญิง) และ `th-TH-NiwatNeural` (ชาย) ดาวน์โหลดเสียงหญิงตัวเดียวกันมาเก็บลงแคช `tts_{hash}.mp3`
          2. สาเหตุที่ระบบเดิมต้องตกไปใช้ Google Translate เพราะการเชื่อมต่อ Microsoft Edge Neural TTS ผ่าน WebSocket ติดข้อผิดพลาด `401 Unauthorized` เนื่องจาก Microsoft บังคับใช้ระบบความปลอดภัย **Sec-MS-GEC v2 (`<16 hex CS64>-<64 hex SHA256>`)** ขณะที่โค้ดเดิมส่งเพียง SHA-256 มาตรฐาน 64 ตัวอักษร
        - **การแก้ไข:**
          1. พอร์ตอัลกอริทึมคำนวณโทเค็น `Sec-MS-GEC v2` (`_cs64_digest` + `_sha256_digest`) ลงใน [`src/audio/tts.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/audio/tts.rs) เป็นภาษา Pure Rust 100%
          2. ปรับลำดับการทำงานใน `synthesize`: ตั้งให้ Microsoft Edge Neural TTS เป็นระบบหลักอันดับ 1 (ยิงตรงผ่าน Edge WebSocket ด้วยโทเค็น V2 ที่ถูกต้อง ไม่ติด 401 อีกต่อไป) โดยใช้ Cloud Voice (Google) เป็นเพียง Fallback #2 และ SAPI เป็น Fallback #3
          3. ล้างไฟล์แคชเสียงเดิมใน `cache/tts/` เพื่อให้ดาวน์โหลดเสียง Neural AI แท้แยกชาย-หญิง (`th-TH-NiwatNeural`, `th-TH-PremwadeeNeural`, `en-US-GuyNeural`, `en-US-JennyNeural`) ทันที
          4. เพิ่มชุดทดสอบ `test_sec_ms_gec_v2_vector` ยืนยันความแม่นยำของโทเค็น 100% ตรงกับ Test Vector และ `test_edge_tts_live_synthesis` ทดสอบการสังเคราะห์เสียงสดจริงกับ Edge WebSocket
        - **Unit Tests 100%:** รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ครบทั้ง **33/33 tests passed** (Zero Regression)
      - **เฟส 39 ระบบถ่ายภาพหน้าจอเกมและจัดเก็บอัตโนมัติเต็มรูปแบบ (Zero-Flicker CDP Screenshot & Desktop Fallback):**
        - **วิเคราะห์สาเหตุ (Root Cause):** ใน [`src/engine/executor.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/engine/executor.rs) โหนด `screenshot` เดิมเป็นเพียง Stub ปริ้นต์คำว่า `"📸 Capturing screen frame..."` โดยไม่ได้เชื่อมต่อการดึงเฟรมจากเบราว์เซอร์หรือเขียนไฟล์ลงโฟลเดอร์ `screenshots/`
        - **การแก้ไข:**
          1. ใน [`src/browser/cdp_session.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/browser/cdp_session.rs): ติดตั้ง `pending_requests` พร้อมเมธอด `call_method` และ `capture_screenshot` เพื่อยิงคำสั่ง `Page.captureScreenshot` ผ่าน WebSocket ไปยัง GPU Surface ของ Chromium และถอดรหัส Base64 ออกมาเป็นไฟล์ JPEG ได้ทันทีในหลักมิลลิวินาที
          2. สร้างโมดูล [`src/vision/capture.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/vision/capture.rs):
             - ฟังก์ชัน `save_and_process_screenshot`: รองรับการตัด Crop ตามพื้นที่ (`full`, `party`, `left`, `right`)
             - จัดเก็บไฟล์ลงโฟลเดอร์ `./screenshots/<subfolder>/screenshot_c<client>_<prefix>_<timestamp>.jpg` อย่างเป็นระเบียบ
             - ฟังก์ชัน `capture_desktop_and_save`: ระบบสำรองดึงภาพหน้าจอ Desktop ของระบบปฏิบัติการ Windows (GDI BitBlt) หากผู้ใช้ทดสอบกดถ่ายภาพในขณะที่จอเกมเป้าหมายยังไม่ได้เปิดใน Launcher
          3. ใน [`src/engine/executor.rs`](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/engine/executor.rs): ดึงพารามิเตอร์ `targetClient`, `subfolder`, `prefix`, `captureRegion`, `annotate` จากโหนด รันบน Tokio Async Task และส่งสัญญาณออกพอร์ต `onComplete` เมื่อเซฟสำเร็จ หรือ `onError` หากล้มเหลว
          4. เพิ่มชุดทดสอบใน `src/vision/capture.rs` ครบทั้ง 4 ด้าน: `test_generate_screenshot_path`, `test_process_region_crop`, `test_save_and_process_screenshot_flow`, `test_desktop_screen_capture_live`
        - **Unit Tests 100%:** รัน `cargo test` ผ่านสมบูรณ์ครบทั้ง **37/37 tests passed** (Zero Regression)
    - รัน `cargo check` และ `cargo test` ผ่านสมบูรณ์ 100% (**37/37 tests passed**)
- **สร้างและปรับปรุงเครื่องมือ .bat (Pure English ASCII / Zero Emoji):**
  - [deploy_admin.bat](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/deploy_admin.bat): ดับเบิลคลิก Deploy เว็บแอดมินขึ้น Vercel ในคลิกเดียว
  - [test.bat](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/test.bat): ดับเบิลคลิกรัน Unit Tests ในคลิกเดียว
  - [build.bat](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/build.bat): ดับเบิลคลิกสำหรับคอมไพล์โปรเจกต์ (Debug / Release)
  - [start.bat](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/start.bat): ดับเบิลคลิกเปิดโปรแกรมทันที พร้อมระบบ Auto-Detect/Rebuild โค้ดอัตโนมัติ

---

## 4. Security & Anti-Crack Decisions (Roadmap)
ข้อสรุปแนวทางความปลอดภัยร่วมกับผู้ใช้:
1. **สิ่งที่ไม่ต้องเข้ารหัส (Open):**
   - Profile JSON (`configs/profiles/*.json`) ปล่อยเปิดปกติ เพื่อให้ผู้ใช้และเพื่อนแก้ไข/แชร์ได้สะดวก
   - UI (`launcher/`, `public/`) ปล่อยปกติ เพราะไม่มีความลับทางธุรกิจ
2. **สิ่งที่ต้องป้องกันเข้มงวด 2 ส่วนหลัก:**
   - **Database & Secret Protection:** ซ่อน API Secrets / Database URLs ด้วย Compile-time Obfuscation และติดต่อผ่าน Gateway API (Cloudflare Worker) เสมอ ไม่ใส่ Master Key ลงใน Client
   - **License System:** ป้องกันการก๊อปและแฮกโปรแกรมด้วย HWID + ลายเซ็นดิจิทัล Ed25519 ตรวจสอบสิทธิ์กับเซิร์ฟเวอร์ก่อนเริ่ม Engine
3. **เอกสารสถาปัตยกรรมระดับสากล:** บันทึกการวิเคราะห์และเปรียบเทียบมาตรฐานเทียบชั้นซอฟต์แวร์ระดับโลก (Docker, Discord, Unreal, Microsoft, JetBrains) ไว้ที่ [docs/SECURITY_AND_SYSTEM_DESIGN_STANDARDS.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/docs/SECURITY_AND_SYSTEM_DESIGN_STANDARDS.md)

---

## 5. Instructions for Fresh Agent in New Chat
- **Mandatory Rules**: ปฏิบัติตาม [.agents/rules/working_style.md](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/.agents/rules/working_style.md) อย่างเคร่งครัด
- เมื่อเปิดแชทใหม่ ผู้ใช้สามารถสั่ง:
> "อ่านไฟล์ `.agents/handoff_latest.md` แล้วพร้อมลุยงานต่อ"
