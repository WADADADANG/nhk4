# 🛡️ NHK License Portal (Next.js Security-First Edition)

ระบบเว็บพอร์ทัลสำหรับผู้ดูแลระบบ (Admin) ในการจัดการ Serial Key, สิทธิ์การใช้งาน และตรวจสอบอุปกรณ์ Hardware ID (HWID) สำหรับโปรเจกต์ **NodeHotKeyV4**

---

## 🔒 ฟีเจอร์ความปลอดภัย (Security First)
- **Next.js Server-Side Protection:** ระบบสุ่มสร้างคีย์และการเข้ารหัสทำงานที่ฝั่ง Server เท่านั้น รหัสลับไม่หลุดไปหน้าเบราว์เซอร์
- **HttpOnly Secure Session Cookies:** ป้องกันการดักจับ Token และป้องกันการแอบดูรหัสผ่านผ่าน Browser DevTools
- **Zero Client Leaks:** ข้อมูล Environment Variables (`ADMIN_PASSWORD`) ถูกเก็บเป็นความลับหลังบ้าน

---

## 🚀 วิธีเปิดใช้งานบนเครื่องของคุณ (Local Run)

### วิธีที่ 1: ดับเบิลคลิกเปิดทันที
- ดับเบิลคลิกที่ไฟล์ `admin.bat` ในโฟลเดอร์หลักของโปรเจกต์
- ระบบจะเริ่มเปิดเซิร์ฟเวอร์และเด้งหน้าเบราว์เซอร์ `http://localhost:3000` ให้อัตโนมัติ

### วิธีที่ 2: รันผ่าน Terminal
```bash
cd admin-web
npm run dev
```

---

## 🔑 บัญชีเข้าใช้งานเริ่มต้น (Default Credentials)
- **ชื่อผู้ใช้ (Username):** `admin`
- **รหัสผ่าน (Password):** `admin1234`
- สามารถปรับเปลี่ยนได้ที่ไฟล์ `admin-web/.env.local`:
  ```env
  ADMIN_USERNAME="admin"
  ADMIN_PASSWORD="รหัสผ่านใหม่ที่คุณต้องการ"
  ```

---

## ☁️ วิธีนำขึ้น Vercel (Online Deploy ฟรี)

1. **อัปโหลดโค้ดขึ้น GitHub** (หรือใช้ Vercel CLI)
2. ไปที่ [Vercel Dashboard](https://vercel.com) แล้วกด **Add New Project**
3. เลือกโฟลเดอร์ Root Directory เป็น: `admin-web`
4. ในส่วน **Environment Variables** ให้เพิ่มตัวแปร:
   - Name: `ADMIN_USERNAME` | Value: `ชื่อผู้ใช้ที่ต้องการ`
   - Name: `ADMIN_PASSWORD` | Value: `รหัสผ่านลับที่คุณต้องการ`
   - Name: `NEXT_PUBLIC_RECAPTCHA_SITE_KEY` | Value: `Site Key ของ Google`
   - Name: `RECAPTCHA_SECRET_KEY` | Value: `Secret Key ของ Google`
5. กด **Deploy** ภายใน 1 นาที คุณจะได้ลิงก์เว็บออนไลน์ที่สามารถเปิดใช้งานผ่านมือถือหรืออุปกรณ์ใดๆ ได้อย่างปลอดภัย 100%!
