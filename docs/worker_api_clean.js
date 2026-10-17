/**
 * NodeHotkey Cloudflare Worker - Pure Engine API
 * - Ed25519 Machine Licensing & Serial Key Auto-Bind API (POST /api/license/redeem)
 * - Online Sync Engine (POST /api/license/check)
 * - Community Hub Profile Workshop API (GET/POST /api/profiles)
 */

function hexToBytes(hex) {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(hex.substr(i * 2, 2), 16);
  }
  return bytes;
}

function bufferToBase64(buffer) {
  let binary = '';
  const bytes = new Uint8Array(buffer);
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

// Ed25519 Sign using Native Web Crypto (Zero External Dependencies)
async function signEd25519(messageStr, privateKeyHex) {
  const pkcs8Prefix = new Uint8Array([
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20
  ]);
  const seedBytes = hexToBytes(privateKeyHex);
  const pkcs8Key = new Uint8Array(pkcs8Prefix.length + seedBytes.length);
  pkcs8Key.set(pkcs8Prefix, 0);
  pkcs8Key.set(seedBytes, pkcs8Prefix.length);

  const cryptoKey = await crypto.subtle.importKey(
    'pkcs8',
    pkcs8Key,
    { name: 'Ed25519' },
    false,
    ['sign']
  );

  const msgBytes = new TextEncoder().encode(messageStr);
  const sigBuffer = await crypto.subtle.sign({ name: 'Ed25519' }, cryptoKey, msgBytes);
  return bufferToBase64(sigBuffer);
}

// 🛡️ In-Memory Edge Cache & Anti-Spam Rate Limiting (Reduces D1 reads to near zero & prevents request spam)
const licenseCheckCache = new Map();
const ipRateLimitMap = new Map();

function checkIpRateLimit(ip, limit = 30, windowMs = 60000) {
  const now = Date.now();
  let record = ipRateLimitMap.get(ip);
  if (!record || now - record.start > windowMs) {
    record = { start: now, count: 1 };
    ipRateLimitMap.set(ip, record);
    return true;
  }
  record.count++;
  return record.count <= limit;
}

// ⚡ Lightweight Pure JS MD5 (Zero Dependencies for Pusher REST API)
function md5(str) {
  function safeAdd(x, y) {
    const lsw = (x & 0xffff) + (y & 0xffff);
    const msw = (x >> 16) + (y >> 16) + (lsw >> 16);
    return (msw << 16) | (lsw & 0xffff);
  }
  function bitRotateLeft(num, cnt) {
    return (num << cnt) | (num >>> (32 - cnt));
  }
  function md5cmn(q, a, b, x, s, t) {
    return safeAdd(bitRotateLeft(safeAdd(safeAdd(a, q), safeAdd(x, t)), s), b);
  }
  function md5ff(a, b, c, d, x, s, t) {
    return md5cmn((b & c) | (~b & d), a, b, x, s, t);
  }
  function md5gg(a, b, c, d, x, s, t) {
    return md5cmn((b & d) | (c & ~d), a, b, x, s, t);
  }
  function md5hh(a, b, c, d, x, s, t) {
    return md5cmn(b ^ c ^ d, a, b, x, s, t);
  }
  function md5ii(a, b, c, d, x, s, t) {
    return md5cmn(c ^ (b | ~d), a, b, x, s, t);
  }
  function binlMD5(x, len) {
    x[len >> 5] |= 0x80 << (len % 32);
    x[(((len + 64) >>> 9) << 4) + 14] = len;
    let a = 1732584193, b = -271733879, c = -1732584194, d = 271733878;
    for (let i = 0; i < x.length; i += 16) {
      const olda = a, oldb = b, oldc = c, oldd = d;
      a = md5ff(a, b, c, d, x[i], 7, -680876936);
      d = md5ff(d, a, b, c, x[i + 1], 12, -389564586);
      c = md5ff(c, d, a, b, x[i + 2], 17, 606105819);
      b = md5ff(b, c, d, a, x[i + 3], 22, -1044525330);
      a = md5ff(a, b, c, d, x[i + 4], 7, -176418897);
      d = md5ff(d, a, b, c, x[i + 5], 12, 1200080426);
      c = md5ff(c, d, a, b, x[i + 6], 17, -1473231341);
      b = md5ff(b, c, d, a, x[i + 7], 22, -45705983);
      a = md5ff(a, b, c, d, x[i + 8], 7, 1770035416);
      d = md5ff(d, a, b, c, x[i + 9], 12, -1958414417);
      c = md5ff(c, d, a, b, x[i + 10], 17, -42063);
      b = md5ff(b, c, d, a, x[i + 11], 22, -1990404162);
      a = md5ff(a, b, c, d, x[i + 12], 7, 1804603682);
      d = md5ff(d, a, b, c, x[i + 13], 12, -40341101);
      c = md5ff(c, d, a, b, x[i + 14], 17, -1502000000);
      b = md5ff(b, c, d, a, x[i + 15], 22, 1236535329);

      a = md5gg(a, b, c, d, x[i + 1], 5, -165796510);
      d = md5gg(d, a, b, c, x[i + 6], 9, -1069501632);
      c = md5gg(c, d, a, b, x[i + 11], 14, 643717713);
      b = md5gg(b, c, d, a, x[i], 20, -373897302);
      a = md5gg(a, b, c, d, x[i + 5], 5, -701558691);
      d = md5gg(d, a, b, c, x[i + 10], 9, 38016083);
      c = md5gg(c, d, a, b, x[i + 15], 14, -660478335);
      b = md5gg(b, c, d, a, x[i + 4], 20, -405537848);
      a = md5gg(a, b, c, d, x[i + 9], 5, 568446438);
      d = md5gg(d, a, b, c, x[i + 14], 9, -1019803690);
      c = md5gg(c, d, a, b, x[i + 3], 14, -187363961);
      b = md5gg(b, c, d, a, x[i + 8], 20, 1163531501);
      a = md5gg(a, b, c, d, x[i + 13], 5, -1444681467);
      d = md5gg(d, a, b, c, x[i + 2], 9, -51403784);
      c = md5gg(c, d, a, b, x[i + 7], 14, 1735328473);
      b = md5gg(b, c, d, a, x[i + 12], 20, -1926607734);

      a = md5hh(a, b, c, d, x[i + 5], 4, -378558);
      d = md5hh(d, a, b, c, x[i + 8], 11, -2022574463);
      c = md5hh(c, d, a, b, x[i + 11], 16, 1839030562);
      b = md5hh(b, c, d, a, x[i + 14], 23, -35309556);
      a = md5hh(a, b, c, d, x[i + 1], 4, -1530992060);
      d = md5hh(d, a, b, c, x[i + 4], 11, 1272893353);
      c = md5hh(c, d, a, b, x[i + 7], 16, -155497632);
      b = md5hh(b, c, d, a, x[i + 10], 23, -1094730640);
      a = md5hh(a, b, c, d, x[i + 13], 4, 681279174);
      d = md5hh(d, a, b, c, x[i], 11, -358537222);
      c = md5hh(c, d, a, b, x[i + 3], 16, -722521979);
      b = md5hh(b, c, d, a, x[i + 6], 23, 76029189);
      a = md5hh(a, b, c, d, x[i + 9], 4, -640364487);
      d = md5hh(d, a, b, c, x[i + 12], 11, -421815835);
      c = md5hh(c, d, a, b, x[i + 15], 16, 530742520);
      b = md5hh(b, c, d, a, x[i + 2], 23, -995338651);

      a = md5ii(a, b, c, d, x[i], 6, -198630844);
      d = md5ii(d, a, b, c, x[i + 7], 10, 1126891415);
      c = md5ii(c, d, a, b, x[i + 14], 15, -1416354905);
      b = md5ii(b, c, d, a, x[i + 5], 21, -57434055);
      a = md5ii(a, b, c, d, x[i + 12], 6, 1700485571);
      d = md5ii(d, a, b, c, x[i + 3], 10, -1894986606);
      c = md5ii(c, d, a, b, x[i + 10], 15, -1051523);
      b = md5ii(b, c, d, a, x[i + 1], 21, -2054922799);
      a = md5ii(a, b, c, d, x[i + 8], 6, 1873313359);
      d = md5ii(d, a, b, c, x[i + 15], 10, -30611744);
      c = md5ii(c, d, a, b, x[i + 6], 15, -1560198380);
      b = md5ii(b, c, d, a, x[i + 13], 21, 1309151649);
      a = md5ii(a, b, c, d, x[i + 4], 6, -145523070);
      d = md5ii(d, a, b, c, x[i + 11], 10, -1120210379);
      c = md5ii(c, d, a, b, x[i + 2], 15, 718787259);
      b = md5ii(b, c, d, a, x[i + 9], 21, -343485551);

      a = safeAdd(a, olda);
      b = safeAdd(b, oldb);
      c = safeAdd(c, oldc);
      d = safeAdd(d, oldd);
    }
    return [a, b, c, d];
  }
  function str2binl(str) {
    const bin = [];
    const mask = (1 << 8) - 1;
    for (let i = 0; i < str.length * 8; i += 8) {
      bin[i >> 5] |= (str.charCodeAt(i / 8) & mask) << (i % 32);
    }
    return bin;
  }
  function binl2hex(binarray) {
    const hexTab = '0123456789abcdef';
    let str = '';
    for (let i = 0; i < binarray.length * 4; i++) {
      str += hexTab.charAt((binarray[i >> 2] >> ((i % 4) * 8 + 4)) & 0x0f) +
             hexTab.charAt((binarray[i >> 2] >> ((i % 4) * 8)) & 0x0f);
    }
    return str;
  }
  return binl2hex(binlMD5(str2binl(str), str.length * 8));
}

async function hmacSha256Hex(keyStr, messageStr) {
  const enc = new TextEncoder();
  const key = await crypto.subtle.importKey(
    'raw',
    enc.encode(keyStr),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const signature = await crypto.subtle.sign('HMAC', key, enc.encode(messageStr));
  return Array.from(new Uint8Array(signature)).map(b => b.toString(16).padStart(2, '0')).join('');
}

// ⚡ Trigger Instant Real-time Event via Pusher Channels (0.1 - 0.3s sub-second reaction)
async function triggerPusherEvent(env, channel, eventName, dataObj) {
  try {
    const appId = env.PUSHER_APP_ID || '2202474';
    const key = env.PUSHER_KEY || '121974b05826cd21a5da';
    const secret = env.PUSHER_SECRET || '8aacc237f6d5f5c121e1';
    const cluster = env.PUSHER_CLUSTER || 'ap1';

    const body = JSON.stringify({
      name: eventName,
      channels: [channel],
      data: JSON.stringify(dataObj),
    });

    const bodyMd5 = md5(body);
    const timestamp = Math.floor(Date.now() / 1000);
    const path = `/apps/${appId}/events`;
    const stringToSign = `POST\n${path}\nauth_key=${key}&auth_timestamp=${timestamp}&auth_version=1.0&body_md5=${bodyMd5}`;
    const authSignature = await hmacSha256Hex(secret, stringToSign);

    const url = `https://api-${cluster}.pusher.com${path}?auth_key=${key}&auth_timestamp=${timestamp}&auth_version=1.0&body_md5=${bodyMd5}&auth_signature=${authSignature}`;

    const resp = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: body,
    });

    return resp.ok;
  } catch (err) {
    console.error('Failed to trigger Pusher event:', err);
    return false;
  }
}

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const { pathname, searchParams } = url;

    // CORS Headers
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    };

    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    try {
      // =========================================================================
      // 🎁 ROUTE 1: ปลดล็อคด้วย SERIAL KEY (REDEEM) -> BIND HWID อัตโนมัติ
      // =========================================================================
      if (pathname === '/api/license/redeem' && request.method === 'POST') {
        const body = await request.json().catch(() => ({}));
        const key = (body.key || '').trim();
        const hwid = (body.hwid || '').trim();

        if (!key || !hwid) {
          return Response.json({ success: false, error: 'Key and HWID are required' }, { status: 400, headers: corsHeaders });
        }

        const keyRecord = await env.DB.prepare(
          'SELECT * FROM license_keys WHERE key = ?'
        ).bind(key).first();

        if (!keyRecord) {
          return Response.json({
            success: false,
            error: 'ไม่พบ License Key นี้ในระบบ กรุณาตรวจสอบความถูกต้อง'
          }, { status: 404, headers: corsHeaders });
        }

        if (keyRecord.status === 'BANNED') {
          return Response.json({
            success: false,
            error: 'License Key นี้ถูกระงับสิทธิ์การใช้งาน (Banned)'
          }, { status: 403, headers: corsHeaders });
        }

        const nowSec = Math.floor(Date.now() / 1000);
        let finalExpiresAt = keyRecord.expires_at;

        // เช็คการผูกเครื่อง (HWID Binding)
        if (!keyRecord.bound_hwid) {
          // 👉 ครั้งแรก: ยังไม่เคยผูกกับใคร -> ล็อกกับ HWID เครื่องนี้ทันที!
          if (keyRecord.duration_days && keyRecord.duration_days > 0) {
            finalExpiresAt = nowSec + (keyRecord.duration_days * 86400);
          } else {
            finalExpiresAt = -1; // ตลอดชีพ
          }

          await env.DB.prepare(
            `UPDATE license_keys 
             SET bound_hwid = ?, status = 'ACTIVATED', activated_at = CURRENT_TIMESTAMP, expires_at = ?
             WHERE key = ?`
          ).bind(hwid, finalExpiresAt, key).run();

          // บันทึกลงตาราง licenses หลักด้วย เพื่อให้ sync_license (Sync Cloud) ใช้งานต่อได้
          await env.DB.prepare(
            `INSERT INTO licenses (id, hwid, tier, expires_at, is_active, user_note, updated_at)
             VALUES (?, ?, ?, ?, 1, ?, CURRENT_TIMESTAMP)
             ON CONFLICT(hwid) DO UPDATE SET
               tier = excluded.tier,
               expires_at = excluded.expires_at,
               is_active = 1,
               updated_at = CURRENT_TIMESTAMP`
          ).bind(`lic_${keyRecord.key}`, hwid, keyRecord.tier, finalExpiresAt, `Redeemed: ${key}`).run();

        } else if (keyRecord.bound_hwid === hwid) {
          // 👉 เครื่องเดิมมากดซ้ำ (อนุญาต)
          if (finalExpiresAt !== -1 && finalExpiresAt <= nowSec) {
            return Response.json({
              success: false,
              error: 'License Key นี้หมดอายุการใช้งานแล้ว'
            }), { status: 403, headers: corsHeaders };
          }
        } else {
          // ❌ โดนเครื่องอื่นเอาไปเปิดใช้ก่อนแล้ว! ปฏิเสธทันที
          return Response.json({
            success: false,
            error: '❌ คีย์นี้ถูกเปิดใช้งานบนเครื่องอื่นไปแล้ว ไม่สามารถใช้งานกับเครื่องนี้ได้'
          }, { status: 403, headers: corsHeaders });
        }

        // เซ็น Ed25519 ส่งกลับให้ Client
        const claims = {
          hwid: hwid,
          tier: keyRecord.tier || 'PRO',
          expires_at: finalExpiresAt,
          issued_at: nowSec,
          user_id: `Key:${key}`,
        };

        const canonical = `HWID:${claims.hwid}|TIER:${claims.tier}|EXPIRES:${claims.expires_at}|ISSUED:${claims.issued_at}|USER:${claims.user_id}`;
        const privKeyHex = env.LICENSE_PRIVATE_KEY || '6c99f62b942594245d72f8073f719cd8f5c7525f01bbafb4f27e96e5ed07b1bc';

        const signatureB64 = await signEd25519(canonical, privKeyHex);

        const tokenObj = {
          algorithm: 'Ed25519',
          claims: claims,
          signature: signatureB64,
        };
        const tokenB64 = btoa(JSON.stringify(tokenObj));

        return Response.json({
          success: true,
          licenseKey: tokenB64,
          tier: keyRecord.tier || 'PRO',
          expires_at: finalExpiresAt,
          message: 'Key activated and locked to this machine successfully.'
        }, { headers: corsHeaders });
      }

      // =========================================================================
      // ☁️ ROUTE 2: เช็คสิทธิ์และซิงค์ออนไลน์ (SYNC CLOUD)
      // =========================================================================
      if (pathname === '/api/license/check' && request.method === 'POST') {
        const clientIp = request.headers.get('cf-connecting-ip') || request.headers.get('x-forwarded-for') || 'default';
        if (!checkIpRateLimit(clientIp, 30, 60000)) {
          return Response.json({ success: false, error: 'Too many requests. Please wait a minute.' }, { status: 429, headers: corsHeaders });
        }

        const body = await request.json().catch(() => ({}));
        const hwid = (body.hwid || '').trim();

        if (!hwid) {
          return Response.json({ success: false, error: 'HWID is required' }, { status: 400, headers: corsHeaders });
        }

        // ⚡ Check In-Memory Edge Cache (Instant response, 0 D1 reads, protects quota)
        const cached = licenseCheckCache.get(hwid);
        if (cached && Date.now() < cached.expires) {
          return Response.json(cached.data, { status: cached.status, headers: corsHeaders });
        }

        const record = await env.DB.prepare(
          'SELECT * FROM licenses WHERE hwid = ?'
        ).bind(hwid).first();

        if (!record) {
          const errData = {
            success: false,
            error: 'HWID not registered. Please enter a valid serial key to activate.'
          };
          licenseCheckCache.set(hwid, { data: errData, status: 404, expires: Date.now() + 30000 });
          return Response.json(errData, { status: 404, headers: corsHeaders });
        }

        let currentStatus = record.status;
        if (!currentStatus) {
          const keyRow = await env.DB.prepare('SELECT status FROM license_keys WHERE bound_hwid = ?').bind(hwid).first();
          if (keyRow) currentStatus = keyRow.status;
        }

        if (currentStatus === 'SUSPENDED') {
          const errData = {
            success: false,
            error: 'License has been suspended temporarily by administrator.'
          };
          licenseCheckCache.set(hwid, { data: errData, status: 403, expires: Date.now() + 30000 });
          return Response.json(errData, { status: 403, headers: corsHeaders });
        }

        if (record.is_active !== 1 || currentStatus === 'BANNED') {
          const errData = {
            success: false,
            error: 'License has been revoked or banned by administrator.'
          };
          licenseCheckCache.set(hwid, { data: errData, status: 403, expires: Date.now() + 30000 });
          return Response.json(errData, { status: 403, headers: corsHeaders });
        }

        const nowSec = Math.floor(Date.now() / 1000);
        if (record.expires_at !== -1 && record.expires_at <= nowSec) {
          const errData = {
            success: false,
            error: 'License has expired.'
          };
          licenseCheckCache.set(hwid, { data: errData, status: 403, expires: Date.now() + 30000 });
          return Response.json(errData, { status: 403, headers: corsHeaders });
        }

        const claims = {
          hwid: record.hwid,
          tier: record.tier || 'PRO',
          expires_at: record.expires_at,
          issued_at: nowSec,
          user_id: record.user_note || null,
        };

        const canonical = `HWID:${claims.hwid}|TIER:${claims.tier}|EXPIRES:${claims.expires_at}|ISSUED:${claims.issued_at}|USER:${claims.user_id || 'ANONYMOUS'}`;
        const privKeyHex = env.LICENSE_PRIVATE_KEY || '6c99f62b942594245d72f8073f719cd8f5c7525f01bbafb4f27e96e5ed07b1bc';

        const signatureB64 = await signEd25519(canonical, privKeyHex);

        const tokenObj = {
          algorithm: 'Ed25519',
          claims: claims,
          signature: signatureB64,
        };
        const tokenB64 = btoa(JSON.stringify(tokenObj));

        const successData = {
          success: true,
          licenseKey: tokenB64,
          tier: record.tier || 'PRO',
          expires_at: record.expires_at,
          message: 'License verified and issued successfully.'
        };
        // Cache valid license for 3 minutes (180s) to reduce D1 query consumption
        licenseCheckCache.set(hwid, { data: successData, status: 200, expires: Date.now() + 180000 });

        return Response.json(successData, { headers: corsHeaders });
      }

      // =========================================================================
      // 📦 ROUTE: ชุมชน PROFILE WORKSHOP (ระบบเดิมที่คุณสร้างไว้)
      // =========================================================================
      if (pathname === '/api/profiles' && request.method === 'GET') {
        const q = searchParams.get('q') || '';
        const sort = searchParams.get('sort') || 'downloads';
        const limit = Math.min(parseInt(searchParams.get('limit') || '50', 10), 100);

        let query = `
          SELECT id, profile_name, author_id, author_name, description, tags, downloads_count, likes_count, version, created_at, updated_at
          FROM profiles
        `;
        const params = [];

        if (q) {
          query += ` WHERE profile_name LIKE ? OR description LIKE ? OR tags LIKE ? OR author_name LIKE ?`;
          const searchTerm = `%${q}%`;
          params.push(searchTerm, searchTerm, searchTerm, searchTerm);
        }

        if (sort === 'recent') {
          query += ` ORDER BY updated_at DESC LIMIT ?`;
        } else {
          query += ` ORDER BY downloads_count DESC, updated_at DESC LIMIT ?`;
        }
        params.push(limit);

        const { results } = await env.DB.prepare(query).bind(...params).all();
        return Response.json({ success: true, profiles: results || [] }, { headers: corsHeaders });
      }

      if (pathname.startsWith('/api/profiles/') && pathname !== '/api/profiles/delete' && request.method === 'GET') {
        const id = pathname.replace('/api/profiles/', '').trim();
        const profile = await env.DB.prepare(`SELECT * FROM profiles WHERE id = ?`).bind(id).first();

        if (!profile) {
          return Response.json({ success: false, error: 'Profile not found' }, { status: 404, headers: corsHeaders });
        }

        const isPreview = searchParams.get('preview') === '1' || searchParams.get('preview') === 'true';
        if (!isPreview) {
          ctx.waitUntil(
            env.DB.prepare(`UPDATE profiles SET downloads_count = downloads_count + 1 WHERE id = ?`).bind(id).run()
          );
        }

        delete profile.author_secret;
        return Response.json({ success: true, profile }, { headers: corsHeaders });
      }

      if (pathname === '/api/profiles' && request.method === 'POST') {
        const body = await request.json();
        const {
          id,
          profile_name,
          author_id,
          author_name,
          author_secret,
          description = '',
          tags = '',
          profile_data,
          version = '1.0.0'
        } = body;

        if (!profile_name || !author_id || !author_secret || !profile_data) {
          return Response.json({ success: false, error: 'Missing required fields' }, { status: 400, headers: corsHeaders });
        }

        const targetId = id || `prf_${crypto.randomUUID().slice(0, 12)}`;
        const existing = await env.DB.prepare(`SELECT author_id, author_secret FROM profiles WHERE id = ?`).bind(targetId).first();

        if (existing) {
          if (existing.author_secret !== author_secret) {
            return Response.json({ success: false, error: 'Unauthorized' }, { status: 403, headers: corsHeaders });
          }

          await env.DB.prepare(`
            UPDATE profiles 
            SET profile_name = ?, author_name = ?, description = ?, tags = ?, profile_data = ?, version = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
          `).bind(profile_name, author_name, description, tags, JSON.stringify(profile_data), version, targetId).run();

          return Response.json({ success: true, id: targetId, message: 'Profile updated successfully' }, { headers: corsHeaders });
        } else {
          await env.DB.prepare(`
            INSERT INTO profiles (id, profile_name, author_id, author_name, author_secret, description, tags, profile_data, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
          `).bind(targetId, profile_name, author_id, author_name, author_secret, description, tags, JSON.stringify(profile_data), version).run();

          return Response.json({ success: true, id: targetId, message: 'Profile published successfully' }, { headers: corsHeaders });
        }
      }

      if ((pathname === '/api/profiles/delete' && request.method === 'POST') ||
          (pathname.startsWith('/api/profiles/') && request.method === 'DELETE')) {
        let targetId = '';
        let secret = '';

        if (request.method === 'POST') {
          const body = await request.json().catch(() => ({}));
          targetId = body.id;
          secret = body.author_secret;
        } else {
          targetId = pathname.replace('/api/profiles/', '').trim();
          const auth = request.headers.get('Authorization') || '';
          secret = auth.replace('Bearer ', '').trim();
        }

        if (!targetId || !secret) {
          return Response.json({ success: false, error: 'Missing profile id or author secret' }, { status: 400, headers: corsHeaders });
        }

        const existing = await env.DB.prepare(`SELECT author_secret FROM profiles WHERE id = ?`).bind(targetId).first();
        if (!existing) {
          return Response.json({ success: false, error: 'Profile not found' }, { status: 404, headers: corsHeaders });
        }

        if (existing.author_secret !== secret) {
          return Response.json({ success: false, error: 'Unauthorized' }, { status: 403, headers: corsHeaders });
        }

        await env.DB.prepare(`DELETE FROM profiles WHERE id = ?`).bind(targetId).run();

        return Response.json({ success: true, message: 'Deleted profile successfully' }, { headers: corsHeaders });
      }

      // =========================================================================
      // 🛡️ ROUTE 3: ADMIN KEY MANAGEMENT API (เชื่อมต่อกับ ADMIN WEB)
      // =========================================================================
      if (pathname.startsWith('/api/admin/keys')) {
        const auth = request.headers.get('Authorization') || request.headers.get('X-Admin-Key') || '';
        const token = auth.replace('Bearer ', '').trim();
        const expectedSecret = env.ADMIN_SECRET || 'nhk_admin_secret_2026';

        if (!token || token !== expectedSecret) {
          return Response.json({ success: false, error: 'Unauthorized: Invalid Admin Secret' }, { status: 401, headers: corsHeaders });
        }

        // 3.1 ดึงรายการคีย์ทั้งหมดจาก Cloudflare D1
        if (pathname === '/api/admin/keys' && request.method === 'GET') {
          const { results } = await env.DB.prepare(`
            SELECT key, tier, duration_days, bound_hwid, status, expires_at, activated_at, user_note, created_at
            FROM license_keys
            ORDER BY created_at DESC
          `).all();
          return Response.json({ success: true, keys: results || [] }, { headers: corsHeaders });
        }

        // 3.2 บันทึกสร้างคีย์ใหม่ลง D1 (Batch Insert)
        if (pathname === '/api/admin/keys/create' && request.method === 'POST') {
          const body = await request.json().catch(() => ({}));
          const keys = Array.isArray(body.keys) ? body.keys : [body];
          
          for (const item of keys) {
            if (!item.key) continue;
            await env.DB.prepare(`
              INSERT INTO license_keys (key, tier, duration_days, bound_hwid, status, user_note)
              VALUES (?, ?, ?, ?, ?, ?)
              ON CONFLICT(key) DO UPDATE SET
                tier = excluded.tier,
                duration_days = excluded.duration_days,
                user_note = excluded.user_note
            `).bind(
              item.key,
              (item.tier || 'PRO').toUpperCase(),
              item.duration_days || item.durationDays || 0,
              item.bound_hwid || item.boundHwid || null,
              item.status || 'UNUSED',
              item.user_note || item.userNote || null
            ).run();
          }
          return Response.json({ success: true, message: 'Keys saved to D1 successfully', count: keys.length }, { headers: corsHeaders });
        }

        // 3.3 อัปเดตสถานะคีย์ (SUSPENDED / ACTIVATED / BANNED) ซิงค์ทั้ง 2 ตาราง
        if (pathname === '/api/admin/keys/status' && request.method === 'POST') {
          const body = await request.json().catch(() => ({}));
          const { key, status } = body;
          if (!key || !status) {
            return Response.json({ success: false, error: 'Missing key or status' }, { status: 400, headers: corsHeaders });
          }

          const cleanStatus = status.toUpperCase();
          // อัปเดตในตาราง license_keys
          await env.DB.prepare(`UPDATE license_keys SET status = ? WHERE key = ?`).bind(cleanStatus, key).run();

          // ซิงค์ไปยังตาราง licenses ทันทีหากคีย์นี้ผูก HWID ไว้แล้ว
          const keyRow = await env.DB.prepare(`SELECT bound_hwid FROM license_keys WHERE key = ?`).bind(key).first();
          if (keyRow && keyRow.bound_hwid) {
            const isActive = cleanStatus === 'ACTIVATED' ? 1 : 0;
            try {
              // กรณีที่ตาราง licenses มีคอลัมน์ status
              await env.DB.prepare(`
                UPDATE licenses 
                SET status = ?, is_active = ?, updated_at = CURRENT_TIMESTAMP
                WHERE hwid = ?
              `).bind(cleanStatus, isActive, keyRow.bound_hwid).run();
            } catch (_) {
              // กรณีที่ตาราง licenses มีเฉพาะ is_active
              await env.DB.prepare(`
                UPDATE licenses 
                SET is_active = ?, updated_at = CURRENT_TIMESTAMP
                WHERE hwid = ?
              `).bind(isActive, keyRow.bound_hwid).run();
            }
          }

          // ⚡ Flush edge cache so client status updates immediately
          licenseCheckCache.clear();

          // ⚡ Trigger Instant Real-time Event via Pusher Channels (0.1 - 0.3s sub-second reaction)
          ctx.waitUntil(
            triggerPusherEvent(env, 'nhk-licenses', 'license_updated', {
              key: key,
              hwid: keyRow ? keyRow.bound_hwid : null,
              status: cleanStatus,
            })
          );

          return Response.json({ success: true, key, status: cleanStatus }, { headers: corsHeaders });
        }

        // 3.4 ลบคีย์ออกจากระบบ (ลบทั้งใน license_keys และ licenses ทันที)
        if (pathname === '/api/admin/keys/delete' && request.method === 'POST') {
          const body = await request.json().catch(() => ({}));
          const { key } = body;
          if (!key) {
            return Response.json({ success: false, error: 'Missing key' }, { status: 400, headers: corsHeaders });
          }

          const keyRow = await env.DB.prepare(`SELECT bound_hwid FROM license_keys WHERE key = ?`).bind(key).first();
          if (keyRow && keyRow.bound_hwid) {
            await env.DB.prepare(`DELETE FROM licenses WHERE hwid = ?`).bind(keyRow.bound_hwid).run();
          }
          await env.DB.prepare(`DELETE FROM license_keys WHERE key = ?`).bind(key).run();

          // ⚡ Flush edge cache so deleted key takes effect immediately
          licenseCheckCache.clear();

          // ⚡ Trigger Instant Real-time Event via Pusher Channels (0.1 - 0.3s sub-second reaction)
          ctx.waitUntil(
            triggerPusherEvent(env, 'nhk-licenses', 'license_updated', {
              key: key,
              hwid: keyRow ? keyRow.bound_hwid : null,
              status: 'DELETED',
            })
          );

          return Response.json({ success: true, key, deleted: true }, { headers: corsHeaders });
        }
      }

      return Response.json({ success: false, error: 'Endpoint not found' }, { status: 404, headers: corsHeaders });
    } catch (err) {
      return Response.json({ success: false, error: err.message }, { status: 500, headers: corsHeaders });
    }
  }
};
