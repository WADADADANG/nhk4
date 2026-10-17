import { cookies } from 'next/headers';
import { NextResponse } from 'next/server';
import crypto from 'crypto';

const WORKER_URL = process.env.CLOUDFLARE_WORKER_URL || 'https://nodehotkey-api.kitsada19972540.workers.dev';
const ADMIN_SECRET = process.env.ADMIN_SECRET;

function getAdminSecret() {
  if (!ADMIN_SECRET) {
    throw new Error('Server configuration error: ADMIN_SECRET is not configured in environment variables');
  }
  return ADMIN_SECRET;
}

// Generate random uppercase alphanumeric token (avoiding confusing characters like 0, O, 1, I)
function generateRandomChunk(length = 4) {
  const chars = 'ABCDEFGHJKLMNPQRSTUVWXYZ23456789';
  let result = '';
  const bytes = crypto.randomBytes(length);
  for (let i = 0; i < length; i++) {
    result += chars[bytes[i] % chars.length];
  }
  return result;
}

function generateSerialKey(tier = 'PRO') {
  const cleanTier = (tier || 'PRO').toUpperCase();
  const c1 = generateRandomChunk(4);
  const c2 = generateRandomChunk(4);
  const c3 = generateRandomChunk(4);
  return `NHK-${cleanTier}-${c1}-${c2}-${c3}`;
}

async function verifyAdminAuth() {
  const cookieStore = await cookies();
  const session = cookieStore.get('nhk_admin_session');
  return !!(session && session.value);
}

// ─── GET: ดึงรายการคีย์ทั้งหมดจาก Cloudflare D1 ───
export async function GET() {
  if (!await verifyAdminAuth()) {
    return NextResponse.json({ success: false, error: 'Unauthorized' }, { status: 401 });
  }

  try {
    const res = await fetch(`${WORKER_URL}/api/admin/keys`, {
      headers: {
        'Authorization': `Bearer ${getAdminSecret()}`,
        'Content-Type': 'application/json'
      },
      cache: 'no-store'
    });

    if (res.ok) {
      const data = await res.json();
      return NextResponse.json(data);
    } else {
      const err = await res.json().catch(() => ({}));
      return NextResponse.json({ success: false, error: err.error || 'Failed to fetch from Cloudflare' }, { status: res.status });
    }
  } catch (error) {
    return NextResponse.json({ success: false, error: error.message }, { status: 500 });
  }
}

// ─── POST: สร้างคีย์และบันทึกลง Cloudflare D1 ทันที ───
export async function POST(request) {
  if (!await verifyAdminAuth()) {
    return NextResponse.json({ success: false, error: 'Unauthorized: กรุณาเข้าสู่ระบบก่อนทำรายการ' }, { status: 401 });
  }

  try {
    const body = await request.json();
    const count = Math.min(Math.max(parseInt(body.count || '1', 10), 1), 100);
    const tier = (body.tier || 'PRO').toUpperCase();
    const durationDays = parseInt(body.durationDays || '0', 10);
    const userNote = (body.userNote || '').trim();

    const generatedKeys = [];
    const sqlValues = [];

    for (let i = 0; i < count; i++) {
      const key = generateSerialKey(tier);
      generatedKeys.push({
        key,
        tier,
        durationDays,
        status: 'UNUSED',
        boundHwid: null,
        userNote: userNote || `Generated ${new Date().toISOString().split('T')[0]}`,
        createdAt: new Date().toISOString()
      });

      const noteEscaped = (userNote || '').replace(/'/g, "''");
      sqlValues.push(`('${key}', '${tier}', ${durationDays}, NULL, 'UNUSED', '${noteEscaped}')`);
    }

    const sqlStatement = `INSERT INTO license_keys (key, tier, duration_days, bound_hwid, status, user_note) VALUES\n  ${sqlValues.join(',\n  ')};`;

    // ยิงบันทึกลง Cloudflare D1 จริงทันที
    let syncedToD1 = false;
    let syncError = null;

    try {
      const workerRes = await fetch(`${WORKER_URL}/api/admin/keys/create`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${getAdminSecret()}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ keys: generatedKeys })
      });

      if (workerRes.ok) {
        syncedToD1 = true;
      } else {
        const errJson = await workerRes.json().catch(() => ({}));
        syncError = errJson.error || 'Worker returned error status';
      }
    } catch (e) {
      syncError = e.message;
    }

    return NextResponse.json({
      success: true,
      keys: generatedKeys,
      sql: sqlStatement,
      count: generatedKeys.length,
      syncedToD1,
      syncError
    });
  } catch (error) {
    return NextResponse.json({ success: false, error: error.message }, { status: 500 });
  }
}

async function triggerPusherEvent(channel, eventName, dataObj) {
  try {
    const appId = process.env.PUSHER_APP_ID || '2202474';
    const key = process.env.PUSHER_KEY || '121974b05826cd21a5da';
    const secret = process.env.PUSHER_SECRET || '8aacc237f6d5f5c121e1';
    const cluster = process.env.PUSHER_CLUSTER || 'ap1';

    const body = JSON.stringify({
      name: eventName,
      channels: [channel],
      data: JSON.stringify(dataObj),
    });

    const bodyMd5 = crypto.createHash('md5').update(body).digest('hex');
    const timestamp = Math.floor(Date.now() / 1000);
    const path = `/apps/${appId}/events`;
    const stringToSign = `POST\n${path}\nauth_key=${key}&auth_timestamp=${timestamp}&auth_version=1.0&body_md5=${bodyMd5}`;
    const authSignature = crypto.createHmac('sha256', secret).update(stringToSign).digest('hex');

    const url = `https://api-${cluster}.pusher.com${path}?auth_key=${key}&auth_timestamp=${timestamp}&auth_version=1.0&body_md5=${bodyMd5}&auth_signature=${authSignature}`;

    await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: body,
    });
  } catch (err) {
    console.error('Failed to trigger Pusher from admin-web:', err);
  }
}

// ─── PATCH: อัปเดตสถานะคีย์ (SUSPENDED / ACTIVATED / BANNED) ───
export async function PATCH(request) {
  if (!await verifyAdminAuth()) {
    return NextResponse.json({ success: false, error: 'Unauthorized' }, { status: 401 });
  }

  try {
    const body = await request.json();
    const { key, status } = body;

    const workerRes = await fetch(`${WORKER_URL}/api/admin/keys/status`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${getAdminSecret()}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ key, status })
    });

    // ⚡ Real-time Pusher push directly from Admin Web (0.1 - 0.2s reaction)
    await triggerPusherEvent('nhk-licenses', 'license_updated', { key, status });

    const data = await workerRes.json().catch(() => ({}));
    return NextResponse.json(data, { status: workerRes.status });
  } catch (error) {
    return NextResponse.json({ success: false, error: error.message }, { status: 500 });
  }
}

// ─── DELETE: ลบคีย์ออกจาก Cloudflare D1 (ทั้งตาราง license_keys และ licenses) ───
export async function DELETE(request) {
  if (!await verifyAdminAuth()) {
    return NextResponse.json({ success: false, error: 'Unauthorized' }, { status: 401 });
  }

  try {
    const body = await request.json().catch(() => ({}));
    const key = body.key;

    if (!key) {
      return NextResponse.json({ success: false, error: 'Key is required' }, { status: 400 });
    }

    const workerRes = await fetch(`${WORKER_URL}/api/admin/keys/delete`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${getAdminSecret()}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({ key })
    });

    // ⚡ Real-time Pusher push directly from Admin Web (0.1 - 0.2s reaction)
    await triggerPusherEvent('nhk-licenses', 'license_updated', { key, status: 'DELETED' });

    const data = await workerRes.json().catch(() => ({}));
    return NextResponse.json(data, { status: workerRes.status });
  } catch (error) {
    return NextResponse.json({ success: false, error: error.message }, { status: 500 });
  }
}
