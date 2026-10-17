/**
 * NodeHotkey Unified Cloudflare Service
 * - Community Hub & Profile Workshop API
 * - Ed25519 Machine Licensing & Serial Key Auto-Bind API (D1 Database)
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

export default {
    async fetch(request, env, ctx) {
        const url = new URL(request.url);
        const { pathname, searchParams } = url;

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
                    return new Response(JSON.stringify({ success: false, error: 'Key and HWID are required' }), {
                        status: 400,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                const keyRecord = await env.DB.prepare(
                    'SELECT * FROM license_keys WHERE key = ?'
                ).bind(key).first();

                if (!keyRecord) {
                    return new Response(JSON.stringify({
                        success: false,
                        error: 'ไม่พบ License Key นี้ในระบบ กรุณาตรวจสอบความถูกต้อง'
                    }), {
                        status: 404,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                if (keyRecord.status === 'BANNED') {
                    return new Response(JSON.stringify({
                        success: false,
                        error: 'License Key นี้ถูกระงับสิทธิ์การใช้งาน (Banned)'
                    }), {
                        status: 403,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                const nowSec = Math.floor(Date.now() / 1000);
                let finalExpiresAt = keyRecord.expires_at;

                // เช็คการผูกเครื่อง
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

                    // บันทึกลงตาราง licenses หลักด้วย
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
                        return new Response(JSON.stringify({
                            success: false,
                            error: 'License Key นี้หมดอายุการใช้งานแล้ว'
                        }), {
                            status: 403,
                            headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                        });
                    }
                } else {
                    // ❌ โดนเครื่องอื่นเอาไปเปิดใช้ก่อนแล้ว! ปฏิเสธทันที
                    return new Response(JSON.stringify({
                        success: false,
                        error: '❌ คีย์นี้ถูกเปิดใช้งานบนเครื่องอื่นไปแล้ว ไม่สามารถใช้งานกับเครื่องนี้ได้'
                    }), {
                        status: 403,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                // เซ็น Ed25519
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

                return new Response(JSON.stringify({
                    success: true,
                    licenseKey: tokenB64,
                    tier: keyRecord.tier || 'PRO',
                    expires_at: finalExpiresAt,
                    message: 'Key activated and locked to this machine successfully.'
                }), {
                    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                });
            }

            // =========================================================================
            // ☁️ ROUTE 2: เช็คสิทธิ์และซิงค์ออนไลน์ (SYNC CLOUD)
            // =========================================================================
            if (pathname === '/api/license/check' && request.method === 'POST') {
                const body = await request.json().catch(() => ({}));
                const hwid = (body.hwid || '').trim();

                if (!hwid) {
                    return new Response(JSON.stringify({ success: false, error: 'HWID is required' }), {
                        status: 400,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                const record = await env.DB.prepare(
                    'SELECT * FROM licenses WHERE hwid = ?'
                ).bind(hwid).first();

                if (!record) {
                    return new Response(JSON.stringify({
                        success: false,
                        error: 'HWID not registered. Please enter a valid serial key to activate.'
                    }), {
                        status: 404,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                if (record.is_active !== 1) {
                    return new Response(JSON.stringify({
                        success: false,
                        error: 'License has been revoked or banned by administrator.'
                    }), {
                        status: 403,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                const nowSec = Math.floor(Date.now() / 1000);
                if (record.expires_at !== -1 && record.expires_at <= nowSec) {
                    return new Response(JSON.stringify({
                        success: false,
                        error: 'License has expired.'
                    }), {
                        status: 403,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
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

                return new Response(JSON.stringify({
                    success: true,
                    licenseKey: tokenB64,
                    tier: record.tier || 'PRO',
                    expires_at: record.expires_at,
                    message: 'License verified and issued successfully.'
                }), {
                    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                });
            }

            // =========================================================================
            // 📦 ROUTE: ชุมชน PROFILE WORKSHOP (ระบบเดิม)
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
                return new Response(JSON.stringify({ success: true, profiles: results || [] }), {
                    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                });
            }

            if (pathname.startsWith('/api/profiles/') && pathname !== '/api/profiles/delete' && request.method === 'GET') {
                const id = pathname.replace('/api/profiles/', '').trim();
                const profile = await env.DB.prepare(`SELECT * FROM profiles WHERE id = ?`).bind(id).first();

                if (!profile) {
                    return new Response(JSON.stringify({ success: false, error: 'Profile not found' }), {
                        status: 404,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                const isPreview = searchParams.get('preview') === '1' || searchParams.get('preview') === 'true';
                if (!isPreview) {
                    ctx.waitUntil(
                        env.DB.prepare(`UPDATE profiles SET downloads_count = downloads_count + 1 WHERE id = ?`).bind(id).run()
                    );
                }

                delete profile.author_secret;
                return new Response(JSON.stringify({ success: true, profile }), {
                    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                });
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
                    return new Response(JSON.stringify({ success: false, error: 'Missing required fields' }), {
                        status: 400,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                const targetId = id || `prf_${crypto.randomUUID().slice(0, 12)}`;
                const existing = await env.DB.prepare(`SELECT author_id, author_secret FROM profiles WHERE id = ?`).bind(targetId).first();

                if (existing) {
                    if (existing.author_secret !== author_secret) {
                        return new Response(JSON.stringify({ success: false, error: 'Unauthorized: คุณไม่ใช่เจ้าของโปรไฟล์นี้ ไม่สามารถแก้ไขทับได้' }), {
                            status: 403,
                            headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                        });
                    }

                    await env.DB.prepare(`
            UPDATE profiles 
            SET profile_name = ?, author_name = ?, description = ?, tags = ?, profile_data = ?, version = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
          `).bind(profile_name, author_name, description, tags, JSON.stringify(profile_data), version, targetId).run();

                    return new Response(JSON.stringify({ success: true, id: targetId, message: 'Profile updated successfully' }), {
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                } else {
                    await env.DB.prepare(`
            INSERT INTO profiles (id, profile_name, author_id, author_name, author_secret, description, tags, profile_data, version)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
          `).bind(targetId, profile_name, author_id, author_name, author_secret, description, tags, JSON.stringify(profile_data), version).run();

                    return new Response(JSON.stringify({ success: true, id: targetId, message: 'Profile published successfully' }), {
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
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
                    return new Response(JSON.stringify({ success: false, error: 'Missing profile id or author secret' }), {
                        status: 400,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                const existing = await env.DB.prepare(`SELECT author_secret FROM profiles WHERE id = ?`).bind(targetId).first();
                if (!existing) {
                    return new Response(JSON.stringify({ success: false, error: 'Profile not found' }), {
                        status: 404,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                if (existing.author_secret !== secret) {
                    return new Response(JSON.stringify({ success: false, error: 'Unauthorized: คุณไม่ใช่เจ้าของโปรไฟล์นี้ ไม่สามารถลบได้' }), {
                        status: 403,
                        headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                    });
                }

                await env.DB.prepare(`DELETE FROM profiles WHERE id = ?`).bind(targetId).run();

                return new Response(JSON.stringify({ success: true, message: 'Deleted profile successfully' }), {
                    headers: { ...corsHeaders, 'Content-Type': 'application/json' },
                });
            }

            return new Response(JSON.stringify({ success: false, error: 'Endpoint not found' }), {
                status: 404,
                headers: { ...corsHeaders, 'Content-Type': 'application/json' },
            });
        } catch (err) {
            return new Response(JSON.stringify({ success: false, error: err.message }), {
                status: 500,
                headers: { ...corsHeaders, 'Content-Type': 'application/json' },
            });
        }
    }
};
