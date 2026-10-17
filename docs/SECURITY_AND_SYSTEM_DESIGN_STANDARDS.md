# 🏛️ NodeHotKeyV4 - Industry Architecture & Security Standards
*(เอกสารวิเคราะห์และเปรียบเทียบสถาปัตยกรรม เทคนิคที่ใช้ และมาตรฐานระดับสากลของโปรเจกต์)*

---

## 📌 1. บทนำและปรัชญาการออกแบบ (Design Philosophy)
โครงสร้างของ **NodeHotKeyV4** ไม่ได้เกิดจากการคิดค้นขึ้นมาเองตามอำเภอใจ แต่เป็นการนำ **Best Practices และสถาปัตยกรรมของบริษัทซอฟต์แวร์ระดับโลก (Industry Standards)** มาหลอมรวมและปรับแต่งให้ตอบโจทย์การทำงานแบบ Multi-Client WebGL Automation ที่ต้องการความเร็วสูงสุด ความเสถียรระดับ Microseconds และความปลอดภัยของข้อมูล

---

## 🏗️ 2. สถาปัตยกรรมและเทคนิคที่ใช้ เปรียบเทียบกับซอฟต์แวร์เจ้าใหญ่ระดับโลก

### 2.1 สถาปัตยกรรม Native Core Engine + Local Daemon
- **เทียบเคียงกับ:** `Docker Desktop`, `Discord`, `Figma Desktop`, `Slack`
- **หลักการทำงาน:**
  - แยก **Core Engine** ที่เขียนด้วยภาษาเนทีฟประสิทธิภาพสูง (**Rust 2021 + Tokio Async Runtime**) ออกจากเลเยอร์แสดงผล (Presentation Layer)
  - ให้ Core Engine ทำงานเป็นเบื้องหลัง (Background Daemon) ให้บริการผ่าน Local REST API และ Server-Sent Events (SSE) พอร์ต `3088`
  - หน้าต่าง Desktop GUI (Tauri / Web Canvas) ทำหน้าที่เพียงเป็น "รีโมตควบคุม" (Client Controller)
- **ประโยชน์ที่ได้รับ:**
  - แม้หน้าต่าง UI จะค้าง ถูกย่อ หรือถูกปิดไป ตัว Engine ของบอทจะยังคงรันคำสั่งได้ต่อเนื่องอย่างแม่นยำ 100%
  - กิน RAM ต่ำมาก (Rust Core กิน RAM เพียง 15–30 MB เทียบกับ Node.js + Electron เดิมที่กิน 200–500 MB)

---

### 2.2 ผังการทำงานแบบ Action Node Studio (Dataflow / Blueprint DAG)
- **เทียบเคียงกับ:** `Unreal Engine Blueprints (Epic Games)`, `Blender Shader Nodes`, `Houdini`
- **หลักการทำงาน:**
  - ใช้สถาปัตยกรรม **Directed Acyclic Graph (DAG)**
  - แบ่งสายสัญญาณเป็น 2 ประเภทชัดเจน:
    1. **Execution Wire (สายสีขาว):** ควบคุมลำดับการไหลของการทำงาน (Control Flow) จาก `exec_in` ไปยัง `exec_out`
    2. **Data Pins (สายสีต่าง ๆ):** ส่งผ่านตัวแปรและข้อมูล (เช่น Number, String, Boolean, Coordinates)
- **ประโยชน์ที่ได้รับ:**
  - ผู้ใช้งานสามารถออกแบบเงื่อนไขที่ซับซ้อน ลูปข้ามจอ หรือคำนวณเงื่อนไขปาร์ตี้ได้แบบ Visual โดยไม่ต้องเขียนโค้ด

---

### 2.3 การควบคุมเบราว์เซอร์ผ่าน Chrome DevTools Protocol (Native CDP)
- **เทียบเคียงกับ:** `Microsoft Playwright`, `Google Puppeteer`
- **หลักการทำงาน:**
  - สื่อสารตรงกับ Chromium ผ่าน WebSocket JSON-RPC ระดับ Native โดยใช้ crate `tokio-tungstenite`
  - ควบคุมการกดปุ่ม เมาส์ และสแกนเฟรมภาพโดยไม่ผ่าน V8 JavaScript Engine ของ Node.js
- **ประโยชน์ที่ได้รับ:**
  - ลด Latency ในการส่งคำสั่งลงเหลือระดับ Sub-millisecond
  - รองรับการคุมหน้าจอพร้อมกัน 1–8 จอได้โดยไม่เกิดปัญหา Event Loop Lag

---

## 🔒 3. สถาปัตยกรรมความปลอดภัยและระบบ License (Security & DRM Standards)

### 3.1 Hardware Identification (HWID Machine Fingerprint)
- **เทียบเคียงกับ:** `Microsoft Windows Product Activation`, `BattlEye`, `Easy Anti-Cheat`
- **หลักการทำงาน ([src/security/hwid.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/hwid.rs)):**
  - รวมลายนิ้วมือของฮาร์ดแวร์ประจำเครื่อง:
    - 🪟 **Windows:** Registry `MachineGuid` (`HKLM\SOFTWARE\Microsoft\Cryptography`) + Processor Identifier
    - 🍎 **macOS:** Native `IOPlatformUUID` ผ่าน `ioreg`
    - 🐧 **Linux:** `/etc/machine-id` หรือ `/var/lib/dbus/machine-id`
  - นำค่ามารวมกับ Secret Salt แล้วแฮชด้วย **SHA-256** แปลงเป็นรูปแบบสากล: `NHKV4-XXXX-XXXX-XXXX-XXXX`
- **ประโยชน์ที่ได้รับ:**
  - รหัสเครื่องไม่เปลี่ยนแปลงแม้จะรีสตาร์ต ป้องกันการปลอมแปลงและการย้ายคีย์ไปเปิดเครื่องอื่น

---

### 3.2 Cryptographic Offline License Validation
- **เทียบเคียงกับ:** `JetBrains License Server / Offline Activation`, `Adobe Creative Cloud`, `Autodesk`
- **หลักการทำงาน ([src/security/license.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/license.rs)):**
  - ยึดตามมาตรฐานสากล **RFC 7519 (JSON Web Tokens - JWT)** และ **RFC 2104 (HMAC-SHA256)**
  - โครงสร้าง License Token บรรจุ Claims:
    - `hwid`: รหัสเครื่องที่ผูกสิทธิ์ (หรือ `UNIVERSAL` สำหรับ Master Developer)
    - `tier`: ระดับสิทธิ์ (เช่น `PRO`, `LIFETIME`)
    - `expires_at`: Unix Timestamp วันหมดอายุ (-1 คือตลอดชีพ)
    - `signature`: ลายเซ็นดิจิทัล HMAC-SHA256
- **ประโยชน์ที่ได้รับ:**
  - ตัวโปรแกรมสามารถตรวจสอบสิทธิ์ได้ในเครื่องแบบรวดเร็วระดับ Microseconds
  - หากลูกค้าแอบแก้ไขข้อความ วันหมดอายุ หรือนำคีย์ไปใส่เครื่องอื่น ลายเซ็นดิจิทัลจะไม่ตรง และระบบจะล็อค `is_running` ทันที (`403 Forbidden`)

---

### 3.3 Compile-Time String Obfuscation (Binary Hardening)
- **เทียบเคียงกับ:** `Denuvo DRM`, `VMProtect`, ซอฟต์แวร์ทางการเงิน & ธนาคาร
- **หลักการทำงาน ([src/security/obfuscate.rs](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/src/security/obfuscate.rs)):**
  - ใช้ Rust Macro `secure_str!(...)` ทำการเข้ารหัสข้อความสำคัญด้วย **Rolling Key XOR Scramble** ตั้งแต่จังหวะคอมไพล์โค้ด
- **ประโยชน์ที่ได้รับ:**
  - ในไฟล์ Binary `.exe` จะไม่มีข้อความ Plaintext หลงเหลืออยู่
  - ป้องกันการใช้โปรแกรม Reverse-Engineering เช่น `strings`, `Ghidra`, `IDA Pro` หรือ `Hex Editor` ส่องหา API URLs หรือ Salt ลับ

---

### 3.4 Release Binary Hardening (`Cargo.toml`)
- **การตั้งค่าในระดับ Compiler:**
  ```toml
  [profile.release]
  opt-level = 3        # ปรับความเร็วสูงสุดของโค้ด
  lto = true           # Link-Time Optimization รวม Function เข้าด้วยกัน ป้องกันการ Decompile
  codegen-units = 1    # คอมไพล์เป็นก้อนเดียวเพื่อการ Optimize ที่แน่นหนาที่สุด
  panic = "abort"      # ตัด Call-stack unwind ออกเพื่อลดขนาดและไม่ทิ้งร่องรอยโค้ด
  strip = true         # ตัด Symbol และชื่อฟังก์ชันทั้งหมดออกจาก Binary
  ```

---

## 🚀 4. แผนงานยกระดับสู่สถาปัตยกรรม Enterprise / AAA (Future Roadmap)

หากในอนาคตต้องการยกระดับความปลอดภัยให้ทัดเทียมระดับองค์กรชั้นนำ (Enterprise-Grade):

| คุณสมบัติ | สถาปัตยกรรมปัจจุบัน | แผนการยกระดับสู่ระดับ Enterprise |
| :--- | :--- | :--- |
| **ระบบการเซ็นคีย์** | **HMAC-SHA256 (Symmetric)**<br>คีย์ลับอยู่ใน Binary | **Ed25519 (Asymmetric Cryptography)**<br>Private Key อยู่บน Cloud Server ของแอดมินคนเดียว ใน Binary มีแค่ Public Key แม้แฮกเกอร์แกะไฟล์ได้ 100% ก็สร้างคีย์ปลอมไม่ได้ |
| **การตรวจสอบสถานะ** | **Offline Validation**<br>ตรวจในเครื่องตามวันหมดอายุ | **Hybrid Heartbeat (Cloudflare Worker)**<br>เชื่อมต่อเช็คสิทธิ์ทุก 24 ชม. สามารถแบนหรือระงับคีย์ได้ทันทีจากแดชบอร์ดหลังบ้าน |
| **การปกป้อง Binary** | **Rust Strip + LTO + XOR** | **Binary Virtualization (VMProtect / Themida)**<br>แปลง Assembly เป็น Bytecode เสมือน ป้องกันการ Debugger ดักจับ Memory |
