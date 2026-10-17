# ☁️ Cloudflare D1 + Worker: License Management Server Setup
*(คู่มือติดตั้งระบบจัดการ License และ Serial Key บน Cloudflare สถาปัตยกรรม Clean Architecture ฟรี 100%)*

ระบบนี้ใช้ Cloudflare Worker ทำหน้าที่เป็น **Pure REST API** (JSON เท่านั้น ไม่ปนเปื้อน HTML) สำหรับ:
1. **Redeem Serial Key:** รับคีย์ที่ลูกค้ากรอก ➔ ล็อกกับ HWID เครื่องลูกค้าทันที ➔ ส่ง Ed25519 Token กลับไปปลดล็อคโปรแกรม
2. **Online Sync (Sync Cloud):** เช็คสิทธิ์และอัปเดตสถานะอัตโนมัติ
3. **Community Profiles:** รองรับระบบ Community Hub & Profile Workshop เดิม

---

## 🔑 ข้อมูลกุญแจเข้ารหัส Ed25519 & ความปลอดภัย
- **Public Key (ฝังในไบนารี .exe ของ Client เรียบร้อยแล้ว):**
  ```text
  714cdfa8d726680844df3e9bdaebccb99ec511a578c6db8240fc5155bbb9a2ba
  ```
- **Private Key (สำหรับใส่ใน Cloudflare Worker Secrets เท่านั้น):**
  - คุณสามารถดู Private Key และ Public Key ได้อย่างปลอดภัยจากเครื่องของคุณโดยรันคำสั่ง:
    ```bash
    cargo run -- --keys-info
    ```
  - นำค่า `Private Key Seed` ที่ได้ ไปใส่ในหน้า Cloudflare Dashboard:
    - ไปที่ Worker **nodehotkey-api** > **Settings** > **Variables and Secrets**
    - เพิ่ม Secret: **`LICENSE_PRIVATE_KEY`** (วางรหัส Private Key)
    - เพิ่ม Secret: **`ADMIN_SECRET`** (ตั้งรหัสผ่านสำหรับหน้า Admin Web)

---

## 🗄️ ขั้นตอนที่ 1: ตารางใน Cloudflare D1 Database (รันครั้งเดียว)

เปิด Cloudflare Dashboard ไปที่ **Storage & Databases** > **D1 SQLite Database** > เลือกฐานข้อมูลของคุณ > แท็บ **Console**

### 1. คำสั่งสร้างตาราง Serial Keys (`license_keys`):
```sql
CREATE TABLE IF NOT EXISTS license_keys (
    key TEXT PRIMARY KEY,
    tier TEXT NOT NULL DEFAULT 'PRO',
    duration_days INTEGER NOT NULL DEFAULT -1,
    bound_hwid TEXT DEFAULT NULL,
    status TEXT NOT NULL DEFAULT 'UNUSED',
    expires_at INTEGER DEFAULT NULL,
    activated_at DATETIME DEFAULT NULL,
    user_note TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_license_keys_bound_hwid ON license_keys(bound_hwid);
```

### 2. คำสั่งสร้างตารางสิทธิ์หลัก (`licenses`):
```sql
CREATE TABLE IF NOT EXISTS licenses (
    id TEXT PRIMARY KEY,
    hwid TEXT NOT NULL UNIQUE,
    tier TEXT NOT NULL DEFAULT 'PRO',
    expires_at INTEGER NOT NULL DEFAULT -1,
    is_active INTEGER NOT NULL DEFAULT 1,
    user_note TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_licenses_hwid ON licenses(hwid);
```

---

## ⚡ ขั้นตอนที่ 2: โค้ด Cloudflare Worker Backend (`worker.js`)
*(Pure REST API เพียวๆ ~300 บรรทัด สะอาดตามมาตรฐานสากล ไม่มี HTML ปนเปื้อน)*

นำโค้ดในไฟล์ [docs/worker_api_clean.js](file:///c:/Users/WADADADANG/Desktop/MyProjects/NodeHotKeyV4/docs/worker_api_clean.js) ไปวางในหน้า **Edit code** ของ Worker `nodehotkey-api` บน Cloudflare แล้วกด **Deploy** ได้ทันที

---

## 🛠️ ขั้นตอนที่ 3: วิธีสร้าง Serial Key ส่งให้ลูกค้า (ง่ายมาก)

เปิด Terminal ในเครื่องของคุณแล้วพิมพ์คำสั่ง:

### 1. สุ่มสร้าง 10 คีย์ (PRO ตลอดชีพ):
```bash
cargo run -- --gen-serial 10
```

### 2. สุ่มสร้าง 5 คีย์ (แบบ 30 วัน):
```bash
cargo run -- --gen-serial 5 PRO 30
```

ระบบจะพิมพ์รหัสคีย์สวยๆ เช่น `NHK-PRO-4A2B-8C1D-E9F0` พร้อมคำสั่ง SQL `INSERT INTO license_keys ...` ที่สามารถก๊อปปี้ไปวางใน D1 Console ครั้งเดียวจบ

---

## 🎮 ขั้นตอนที่ 4: สำหรับลูกค้า (ใช้งานง่ายที่สุด)
1. เมื่อลูกค้าซื้อคีย์ คุณแค่ส่งรหัส เช่น `NHK-PRO-4A2B-8C1D-E9F0` ให้ลูกค้า
2. ลูกค้าเปิดโปรแกรม ➔ ไปที่ Settings ➔ วางคีย์ลงในช่อง `[ วาง License Key ที่ได้รับที่นี่ ]` ➔ กดปุ่ม **`[✨ Activate]`**
3. โปรแกรมจะยิงไปที่ Cloudflare เพื่อ **ล็อกคีย์เข้ากับ HWID ของเครื่องลูกค้ารายนั้นทันที** และเปิดใช้งานได้ตลอดไป!
