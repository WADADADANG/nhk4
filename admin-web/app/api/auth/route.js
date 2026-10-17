import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';
import crypto from 'crypto';

// Rate Limiting Map: Map<ip, { attempts: number, lockedUntil: number }>
const rateLimitMap = new Map();
const MAX_ATTEMPTS = 5;
const LOCKOUT_MS = 15 * 60 * 1000; // 15 minutes

// Google official test keys as default fallback
const DEFAULT_RECAPTCHA_SECRET = '6LeIxAcTAAAAAGG-vFI1TnRWxMZNFuojJ4WifJWe';

function getClientIp(request) {
  const forwarded = request.headers.get('x-forwarded-for');
  if (forwarded) {
    return forwarded.split(',')[0].trim();
  }
  return request.headers.get('x-real-ip') || '127.0.0.1';
}

function checkRateLimit(ip) {
  const record = rateLimitMap.get(ip);
  const now = Date.now();

  if (!record) {
    return { allowed: true, remaining: MAX_ATTEMPTS };
  }

  // Check if lockout has expired
  if (record.lockedUntil && record.lockedUntil > now) {
    const remainingSeconds = Math.ceil((record.lockedUntil - now) / 1000);
    const remainingMinutes = Math.ceil(remainingSeconds / 60);
    return { allowed: false, remaining: 0, lockedMinutes: remainingMinutes };
  }

  // Reset if lockout is over
  if (record.lockedUntil && record.lockedUntil <= now) {
    rateLimitMap.delete(ip);
    return { allowed: true, remaining: MAX_ATTEMPTS };
  }

  return { allowed: true, remaining: Math.max(0, MAX_ATTEMPTS - record.attempts) };
}

function recordFailedAttempt(ip) {
  const now = Date.now();
  const record = rateLimitMap.get(ip) || { attempts: 0, lockedUntil: 0 };
  record.attempts += 1;

  if (record.attempts >= MAX_ATTEMPTS) {
    record.lockedUntil = now + LOCKOUT_MS;
  }

  rateLimitMap.set(ip, record);
  return record;
}

function resetRateLimit(ip) {
  rateLimitMap.delete(ip);
}

export async function GET() {
  const cookieStore = await cookies();
  const session = cookieStore.get('nhk_admin_session');

  const siteKey = process.env.NEXT_PUBLIC_RECAPTCHA_SITE_KEY || '6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI';

  if (session && session.value) {
    return NextResponse.json({ authenticated: true, siteKey });
  }

  return NextResponse.json({ authenticated: false, siteKey });
}

export async function POST(request) {
  const ip = getClientIp(request);
  const rateLimitStatus = checkRateLimit(ip);

  // 1. Check Rate Limit Lockout
  if (!rateLimitStatus.allowed) {
    return NextResponse.json({
      success: false,
      error: `คุณกรอกข้อมูลผิดเกินกำหนด (5 ครั้ง) ระบบถูกระงับชั่วคราว กรุณารออีก ${rateLimitStatus.lockedMinutes} นาที`,
      isLocked: true,
      remainingMinutes: rateLimitStatus.lockedMinutes
    }, { status: 429 });
  }

  try {
    const body = await request.json();
    const { username, password, recaptchaToken } = body;
    const adminUsername = process.env.ADMIN_USERNAME || 'admin';
    const adminPassword = process.env.ADMIN_PASSWORD || 'admin1234';
    const recaptchaSecret = process.env.RECAPTCHA_SECRET_KEY || DEFAULT_RECAPTCHA_SECRET;

    // 2. Google reCAPTCHA Verification
    if (!recaptchaToken) {
      return NextResponse.json({
        success: false,
        error: 'กรุณากดติ๊กยืนยัน reCAPTCHA (ฉันไม่ใช่โปรแกรมอัตโนมัติ)'
      }, { status: 400 });
    }

    try {
      const verifyUrl = 'https://www.google.com/recaptcha/api/siteverify';
      const verifyRes = await fetch(verifyUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
          secret: recaptchaSecret,
          response: recaptchaToken,
          remoteip: ip
        })
      });

      const verifyData = await verifyRes.json();
      if (!verifyData.success) {
        return NextResponse.json({
          success: false,
          error: 'การยืนยัน Google reCAPTCHA ล้มเหลว กรุณากดติ๊กยืนยันใหม่อีกครั้ง'
        }, { status: 400 });
      }
    } catch (err) {
      console.error('reCAPTCHA verification error:', err);
      return NextResponse.json({
        success: false,
        error: 'ไม่สามารถติดต่อเซิร์ฟเวอร์ Google reCAPTCHA ได้ กรุณาลองใหม่'
      }, { status: 500 });
    }

    // 3. Username & Password Verification (Timing-Safe Check)
    if (!username || !password) {
      return NextResponse.json({ success: false, error: 'กรุณากรอกชื่อผู้ใช้และรหัสผ่าน' }, { status: 400 });
    }

    const userBuffer = Buffer.from(username.trim());
    const adminUserBuffer = Buffer.from(adminUsername.trim());
    const pwdBuffer = Buffer.from(password);
    const adminPwdBuffer = Buffer.from(adminPassword);

    const isUserMatch = userBuffer.length === adminUserBuffer.length && crypto.timingSafeEqual(userBuffer, adminUserBuffer);
    const isPwdMatch = pwdBuffer.length === adminPwdBuffer.length && crypto.timingSafeEqual(pwdBuffer, adminPwdBuffer);

    const isMatch = isUserMatch && isPwdMatch;

    if (isMatch) {
      resetRateLimit(ip);

      const sessionToken = crypto.createHmac('sha256', adminPassword)
        .update(`session_${Date.now()}_${ip}`)
        .digest('hex');

      const cookieStore = await cookies();
      cookieStore.set('nhk_admin_session', sessionToken, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'strict',
        maxAge: 60 * 60 * 24 * 7, // 7 days
        path: '/',
      });

      return NextResponse.json({
        success: true,
        message: 'เข้าสู่ระบบสำเร็จ'
      });
    }

    // Failed Authentication Attempt
    const updatedRecord = recordFailedAttempt(ip);
    const remaining = Math.max(0, MAX_ATTEMPTS - updatedRecord.attempts);

    if (updatedRecord.lockedUntil > Date.now()) {
      return NextResponse.json({
        success: false,
        error: 'คุณกรอกข้อมูลเข้าสู่ระบบผิดเกินกำหนด (5 ครั้ง) ระบบถูกระงับชั่วคราว 15 นาที',
        isLocked: true,
        remainingMinutes: 15
      }, { status: 429 });
    }

    return NextResponse.json({
      success: false,
      error: `ชื่อผู้ใช้หรือรหัสผ่านไม่ถูกต้อง (เหลือโอกาสลองอีก ${remaining} ครั้ง)`,
      remainingAttempts: remaining
    }, { status: 401 });

  } catch (error) {
    return NextResponse.json({ success: false, error: error.message }, { status: 500 });
  }
}

export async function DELETE() {
  const cookieStore = await cookies();
  cookieStore.delete('nhk_admin_session');
  return NextResponse.json({ success: true, message: 'ออกจากระบบสำเร็จ' });
}
