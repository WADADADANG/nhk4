'use client';

import { useState, useEffect } from 'react';
import {
  Key, ShieldCheck, Lock, Unlock, LogOut, Copy, Check,
  Terminal, Search, RefreshCw, Cpu,
  AlertCircle, Settings, Eye, EyeOff, Layers, Download, Trash2,
  Pause, Play
} from 'lucide-react';

export default function AdminPortal() {
  // Authentication State
  const [authStatus, setAuthStatus] = useState('checking'); // 'checking' | 'authenticated' | 'unauthenticated'
  const [usernameInput, setUsernameInput] = useState('');
  const [passwordInput, setPasswordInput] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [loginError, setLoginError] = useState('');
  const [isLoggingIn, setIsLoggingIn] = useState(false);
  const [recaptchaToken, setRecaptchaToken] = useState('');
  const [siteKey, setSiteKey] = useState('6LeIxAcTAAAAAJcZVRqyHh71UMIEGNQ_MXjiZKhI');
  const [isLocked, setIsLocked] = useState(false);
  const [remainingMinutes, setRemainingMinutes] = useState(0);

  // App Navigation & Tabs
  const [activeTab, setActiveTab] = useState('generator'); // 'generator' | 'inventory' | 'devices' | 'settings'

  // Generator Form State
  const [genCount, setGenCount] = useState(1);
  const [genTier, setGenTier] = useState('PRO');
  const [genDuration, setGenDuration] = useState(1);
  const [genNote, setGenNote] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);

  // Results & Inventory Data
  const [latestGenerated, setLatestGenerated] = useState([]);
  const [latestSql, setLatestSql] = useState('');
  const [allKeys, setAllKeys] = useState([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState('ALL');
  const [isLoadingInventory, setIsLoadingInventory] = useState(false);

  // Toasts
  const [toasts, setToasts] = useState([]);
  const [copiedId, setCopiedId] = useState(null);

  // Cloudflare Settings
  const [workerUrl, setWorkerUrl] = useState('https://nodehotkey-api.kitsada19972540.workers.dev');

  const addToast = (message, type = 'success') => {
    const id = Date.now();
    setToasts(prev => [...prev, { id, message, type }]);
    setTimeout(() => {
      setToasts(prev => prev.filter(t => t.id !== id));
    }, 3200);
  };

  useEffect(() => {
    checkAuth();
    loadLocalInventory();
  }, []);

  useEffect(() => {
    if (authStatus === 'authenticated') {
      loadInventory();
    }
  }, [authStatus]);

  // Setup Google reCAPTCHA Callbacks & Script
  useEffect(() => {
    if (authStatus === 'unauthenticated') {
      window.onRecaptchaSuccess = (token) => {
        setRecaptchaToken(token);
        setLoginError('');
      };
      window.onRecaptchaExpired = () => {
        setRecaptchaToken('');
      };

      if (!document.getElementById('google-recaptcha-script')) {
        const script = document.createElement('script');
        script.id = 'google-recaptcha-script';
        script.src = 'https://www.google.com/recaptcha/api.js';
        script.async = true;
        script.defer = true;
        document.body.appendChild(script);
      } else if (window.grecaptcha && window.grecaptcha.reset) {
        window.grecaptcha.reset();
      }
    }
  }, [authStatus]);

  const checkAuth = async () => {
    try {
      const res = await fetch('/api/auth');
      const data = await res.json();
      if (data.siteKey) {
        setSiteKey(data.siteKey);
      }
      if (data.authenticated) {
        setAuthStatus('authenticated');
      } else {
        setAuthStatus('unauthenticated');
      }
    } catch {
      setAuthStatus('unauthenticated');
    }
  };

  const handleLogin = async (e) => {
    e.preventDefault();
    if (!usernameInput.trim() || !passwordInput.trim()) return;

    if (!recaptchaToken) {
      setLoginError('กรุณากดติ๊กยืนยัน reCAPTCHA (ฉันไม่ใช่โปรแกรมอัตโนมัติ)');
      return;
    }

    setIsLoggingIn(true);
    setLoginError('');

    try {
      const res = await fetch('/api/auth', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          username: usernameInput.trim(),
          password: passwordInput,
          recaptchaToken: recaptchaToken
        })
      });
      const data = await res.json();

      if (res.ok && data.success) {
        setAuthStatus('authenticated');
        setIsLocked(false);
        addToast('เข้าสู่ระบบสำเร็จ');
        loadInventory();
      } else {
        if (data.isLocked) {
          setIsLocked(true);
          setRemainingMinutes(data.remainingMinutes || 15);
        }
        setLoginError(data.error || 'รหัสผ่านไม่ถูกต้อง');
        if (typeof window !== 'undefined' && window.grecaptcha && window.grecaptcha.reset) {
          window.grecaptcha.reset();
          setRecaptchaToken('');
        }
      }
    } catch {
      setLoginError('เกิดข้อผิดพลาดในการเชื่อมต่อ');
      if (typeof window !== 'undefined' && window.grecaptcha && window.grecaptcha.reset) {
        window.grecaptcha.reset();
        setRecaptchaToken('');
      }
    } finally {
      setIsLoggingIn(false);
    }
  };

  const handleLogout = async () => {
    try {
      await fetch('/api/auth', { method: 'DELETE' });
      setAuthStatus('unauthenticated');
      setUsernameInput('');
      setPasswordInput('');
      addToast('ออกจากระบบแล้ว');
    } catch (err) {
      console.error(err);
    }
  };

  const loadInventory = async (showSuccessToast = false) => {
    setIsLoadingInventory(true);
    try {
      const res = await fetch(`/api/keys?t=${Date.now()}`, { cache: 'no-store' });
      if (res.ok) {
        const data = await res.json();
        if (data && data.success && Array.isArray(data.keys)) {
          const normalized = data.keys.map(k => ({
            key: k.key,
            tier: k.tier,
            durationDays: k.duration_days !== undefined ? k.duration_days : (k.durationDays || 0),
            status: k.status,
            boundHwid: k.bound_hwid !== undefined ? k.bound_hwid : (k.boundHwid || null),
            userNote: k.user_note !== undefined ? k.user_note : (k.userNote || ''),
            createdAt: k.created_at || k.createdAt || new Date().toISOString(),
            expiresAt: k.expires_at || k.expiresAt
          }));
          setAllKeys(normalized);
          try { localStorage.setItem('nhk_admin_keys_v4', JSON.stringify(normalized)); } catch (e) {}
          if (showSuccessToast) {
            addToast(`ซิงค์ข้อมูล Cloudflare D1 ล่าสุดเรียบร้อย (${normalized.length} รายการ) ☁️`);
          }
          return;
        } else if (data && data.error) {
          addToast(`โหลดข้อมูลจาก D1 ไม่สำเร็จ: ${data.error}`, 'error');
        }
      } else {
        const err = await res.json().catch(() => ({}));
        addToast(`เชื่อมต่อ Cloudflare ไม่สำเร็จ: ${err.error || res.statusText}`, 'error');
      }
    } catch (err) {
      console.warn('Failed to load keys from Cloudflare D1, falling back to local cache:', err);
      if (showSuccessToast) {
        addToast(`เชื่อมต่อ Cloudflare ไม่สำเร็จ: ${err.message}`, 'error');
      }
    } finally {
      setIsLoadingInventory(false);
    }
    loadLocalInventory();
  };

  const loadLocalInventory = () => {
    try {
      const stored = localStorage.getItem('nhk_admin_keys_v4');
      if (stored) {
        setAllKeys(JSON.parse(stored));
      } else {
        setAllKeys([]);
      }
    } catch (err) {
      console.error(err);
    }
  };

  const saveInventory = (keys) => {
    setAllKeys(keys);
    try {
      localStorage.setItem('nhk_admin_keys_v4', JSON.stringify(keys));
    } catch (err) {
      console.error(err);
    }
  };

  const handleGenerateKeys = async (e) => {
    e.preventDefault();
    setIsGenerating(true);

    try {
      const res = await fetch('/api/keys', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          count: genCount,
          tier: genTier,
          durationDays: genDuration,
          userNote: genNote
        })
      });

      const data = await res.json();

      if (res.ok && data.success) {
        setLatestGenerated(data.keys);
        setLatestSql(data.sql);

        const updated = [...data.keys, ...allKeys];
        saveInventory(updated);

        if (data.syncedToD1) {
          addToast(`สร้าง ${data.count} Serial Keys และบันทึกลง Cloudflare D1 สำเร็จ! ☁️`);
        } else {
          addToast(`สร้าง ${data.count} Serial Keys สำเร็จ (Local)${data.syncError ? ': ' + data.syncError : ''}`, 'warning');
        }
        // Refresh directly from D1
        loadInventory();
      } else {
        addToast(data.error || 'สร้างคีย์ไม่สำเร็จ', 'error');
      }
    } catch (err) {
      addToast(err.message, 'error');
    } finally {
      setIsGenerating(false);
    }
  };

  const copyToClipboard = (text, id = null) => {
    navigator.clipboard.writeText(text);
    if (id) {
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 1800);
    }
    addToast('คัดลอกลงคลิปบอร์ดแล้ว');
  };

  const toggleSuspendKey = async (keyString) => {
    const target = allKeys.find(k => k.key === keyString);
    if (!target) return;
    const newStatus = target.status === 'SUSPENDED'
      ? (target.boundHwid ? 'ACTIVATED' : 'UNUSED')
      : 'SUSPENDED';

    const updated = allKeys.map(k => k.key === keyString ? { ...k, status: newStatus } : k);
    saveInventory(updated);

    try {
      const res = await fetch('/api/keys', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key: keyString, status: newStatus })
      });
      const data = await res.json().catch(() => ({}));
      if (res.ok && data.success) {
        if (newStatus === 'SUSPENDED') {
          addToast(`หยุดใช้งานชั่วคราวคีย์ ${keyString} บน Cloudflare D1 เรียบร้อย 🟡`, 'warning');
        } else {
          addToast(`เปิดใช้งานคีย์ ${keyString} ต่อบน Cloudflare D1 เรียบร้อย 🟢`);
        }
      } else {
        addToast(`บันทึกบน Cloudflare ไม่สำเร็จ: ${data.error || 'API error'}`, 'error');
      }
    } catch (err) {
      addToast(`เชื่อมต่อไม่สำเร็จ: ${err.message}`, 'error');
    }
  };

  const toggleBanKey = async (keyString) => {
    const target = allKeys.find(k => k.key === keyString);
    if (!target) return;
    const newStatus = target.status === 'BANNED'
      ? (target.boundHwid ? 'ACTIVATED' : 'UNUSED')
      : 'BANNED';

    const updated = allKeys.map(k => k.key === keyString ? { ...k, status: newStatus } : k);
    saveInventory(updated);

    try {
      const res = await fetch('/api/keys', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key: keyString, status: newStatus })
      });
      const data = await res.json().catch(() => ({}));
      if (res.ok && data.success) {
        addToast(newStatus === 'BANNED' ? `แบนคีย์ ${keyString} บน Cloudflare D1 เรียบร้อย 🔴` : `ปลดแบนคีย์ ${keyString} เรียบร้อย 🟢`);
      } else {
        addToast(`บันทึกบน Cloudflare ไม่สำเร็จ: ${data.error || 'API error'}`, 'error');
      }
    } catch (err) {
      addToast(`เชื่อมต่อไม่สำเร็จ: ${err.message}`, 'error');
    }
  };

  const handleDeleteKey = async (keyString) => {
    if (!confirm(`คุณต้องการลบคีย์ ${keyString} ออกจากฐานข้อมูล Cloudflare D1 ถาวรใช่หรือไม่?\n\n(หากเครื่องกำลังใช้งานอยู่ ระบบจะตัดสิทธิ์และลบ License ในเครื่องทันที)`)) {
      return;
    }

    const updated = allKeys.filter(k => k.key !== keyString);
    saveInventory(updated);

    try {
      const res = await fetch('/api/keys', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ key: keyString })
      });
      const data = await res.json().catch(() => ({}));
      if (res.ok && data.success) {
        addToast(`ลบคีย์ ${keyString} ออกจาก Cloudflare D1 เรียบร้อย 🗑️`);
      } else {
        addToast(`ลบบน Cloudflare ไม่สำเร็จ: ${data.error || 'API error'}`, 'error');
      }
    } catch (err) {
      addToast(`เชื่อมต่อไม่สำเร็จ: ${err.message}`, 'error');
    }
  };

  const filteredKeys = allKeys.filter(k => {
    const matchesFilter = statusFilter === 'ALL' || k.status === statusFilter;
    const q = searchQuery.toLowerCase();
    const matchesQuery = !q ||
      k.key.toLowerCase().includes(q) ||
      (k.userNote && k.userNote.toLowerCase().includes(q)) ||
      (k.boundHwid && k.boundHwid.toLowerCase().includes(q));
    return matchesFilter && matchesQuery;
  });

  const activeDevices = allKeys.filter(k => k.boundHwid);

  // Metrics
  const statTotal = allKeys.length;
  const statUnused = allKeys.filter(k => k.status === 'UNUSED').length;
  const statActivated = allKeys.filter(k => k.status === 'ACTIVATED').length;
  const statSuspended = allKeys.filter(k => k.status === 'SUSPENDED').length;
  const statBanned = allKeys.filter(k => k.status === 'BANNED').length;

  // --------------------------------------------------------------------------
  // RENDER: LOADING
  // --------------------------------------------------------------------------
  if (authStatus === 'checking') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-[#09090b]">
        <div className="flex items-center gap-3 text-zinc-500 text-sm">
          <RefreshCw className="w-4 h-4 animate-spin text-zinc-300" />
          <span className="font-mono">Loading workspace...</span>
        </div>
      </div>
    );
  }

  // --------------------------------------------------------------------------
  // RENDER: LOGIN CARD (LINEAR / SHADCN STYLE)
  // --------------------------------------------------------------------------
  if (authStatus === 'unauthenticated') {
    return (
      <div className="login-wrapper">
        <div className="login-card">
          <div className="login-header">
            <div className="login-icon-box">
              <Lock className="w-5 h-5 text-zinc-300" />
            </div>
            <h1 className="login-title">NHK Admin</h1>
            <p className="login-sub">Enter credentials to access workspace</p>
          </div>

          {isLocked ? (
            <div className="p-4 bg-rose-950/40 border border-rose-800/60 rounded-lg flex flex-col gap-2 text-rose-300 text-xs">
              <div className="flex items-center gap-2 font-bold text-sm">
                <AlertCircle className="w-4 h-4 shrink-0 text-rose-400" />
                <span>ถูกระงับการเข้าสู่ระบบชั่วคราว</span>
              </div>
              <p className="text-zinc-400 leading-relaxed">
                เนื่องจากมีการกรอกข้อมูลผิดติดต่อกันครบ 5 ครั้ง เพื่อความปลอดภัย ระบบได้ระงับการเข้าใช้งานชั่วคราวเป็นเวลา 15 นาที
              </p>
              <div className="pt-2 text-center text-xs text-zinc-500 font-mono">
                กรุณารอประมาณ {remainingMinutes} นาทีแล้วลองใหม่อีกครั้ง
              </div>
            </div>
          ) : (
            <form onSubmit={handleLogin} className="flex flex-col gap-3.5">
              <div className="form-group">
                <label htmlFor="adminUsername">Username</label>
                <input
                  id="adminUsername"
                  type="text"
                  className="form-input text-sm"
                  placeholder="Username"
                  value={usernameInput}
                  onChange={(e) => setUsernameInput(e.target.value)}
                  autoFocus
                  required
                />
              </div>

              <div className="form-group">
                <label htmlFor="adminPassword">Password</label>
                <div className="relative">
                  <input
                    id="adminPassword"
                    type={showPassword ? 'text' : 'password'}
                    className="form-input pr-10 code-font text-sm"
                    placeholder="Password"
                    value={passwordInput}
                    onChange={(e) => setPasswordInput(e.target.value)}
                    required
                  />
                  <button
                    type="button"
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-zinc-500 hover:text-zinc-300"
                    onClick={() => setShowPassword(!showPassword)}
                  >
                    {showPassword ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
              </div>

              {/* Google reCAPTCHA v2 Checkbox Widget */}
              <div className="form-group flex justify-center py-1">
                <div
                  className="g-recaptcha"
                  data-sitekey={siteKey}
                  data-theme="dark"
                  data-callback="onRecaptchaSuccess"
                  data-expired-callback="onRecaptchaExpired"
                ></div>
              </div>

              {loginError && (
                <div className="text-xs text-rose-400 flex items-center gap-1.5 mt-0.5">
                  <AlertCircle className="w-3.5 h-3.5 shrink-0" />
                  <span>{loginError}</span>
                </div>
              )}

              <button
                type="submit"
                disabled={isLoggingIn}
                className="btn btn-primary btn-block mt-2"
              >
                {isLoggingIn ? (
                  <>
                    <RefreshCw className="w-4 h-4 animate-spin" />
                    <span>Verifying...</span>
                  </>
                ) : (
                  <>
                    <Unlock className="w-4 h-4" />
                    <span>Sign In</span>
                  </>
                )}
              </button>
            </form>
          )}

          <div className="text-center text-xs text-zinc-600 pt-2 border-t border-zinc-800/60 font-mono">
            NodeHotKeyV4 • Local Dev Security Gate
          </div>
        </div>
      </div>
    );
  }

  // --------------------------------------------------------------------------
  // RENDER: WORKSPACE (UNIFIED DEV / LINEAR STYLE)
  // --------------------------------------------------------------------------
  return (
    <div className="min-h-screen bg-[#09090b]">
      {/* App Header */}
      <header className="app-header">
        <div className="header-container">
          <div className="brand">
            <div className="brand-logo">
              <span>🦀</span>
            </div>
            <div className="brand-info">
              <div className="brand-title">
                <span>NHK Portal</span>
                <span className="badge-version">v4.0</span>
              </div>
              <div className="brand-sub">License & Serial Key Workspace</div>
            </div>
          </div>

          <div className="header-actions">
            <button
              className="btn btn-secondary btn-sm flex items-center gap-1.5"
              onClick={() => loadInventory(true)}
              disabled={isLoadingInventory}
              title="ดึงข้อมูลล่าสุดจาก Cloudflare D1"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${isLoadingInventory ? 'animate-spin text-emerald-400' : 'text-zinc-400'}`} />
              <span className="hidden sm:inline">{isLoadingInventory ? 'กำลังซิงค์...' : 'ซิงค์ D1'}</span>
            </button>
            <div className="status-indicator">
              <span className="status-dot"></span>
              <span>Online</span>
            </div>
            <button
              className="btn btn-secondary btn-sm"
              onClick={handleLogout}
              title="Sign Out"
            >
              <LogOut className="w-3.5 h-3.5 text-zinc-400" />
              <span className="hidden sm:inline">Sign Out</span>
            </button>
          </div>
        </div>
      </header>

      {/* Main Container */}
      <main className="app-main">
        <div className="main-container">

          {/* Navigation Tabs (Segmented Control) */}
          <nav className="nav-tabs">
            <button
              className={`nav-tab ${activeTab === 'generator' ? 'active' : ''}`}
              onClick={() => setActiveTab('generator')}
            >
              <Key className="w-3.5 h-3.5" />
              <span>สร้าง Serial Key</span>
            </button>
            <button
              className={`nav-tab ${activeTab === 'inventory' ? 'active' : ''}`}
              onClick={() => {
                setActiveTab('inventory');
                loadInventory();
              }}
            >
              <Layers className="w-3.5 h-3.5" />
              <span>คลังคีย์ทั้งหมด</span>
              <span className="tab-badge">{statTotal}</span>
            </button>
            <button
              className={`nav-tab ${activeTab === 'devices' ? 'active' : ''}`}
              onClick={() => {
                setActiveTab('devices');
                loadInventory();
              }}
            >
              <Cpu className="w-3.5 h-3.5" />
              <span>เครื่องลูกข่าย (HWID)</span>
              <span className="tab-badge">{activeDevices.length}</span>
            </button>
            <button
              className={`nav-tab ${activeTab === 'settings' ? 'active' : ''}`}
              onClick={() => setActiveTab('settings')}
            >
              <Settings className="w-3.5 h-3.5" />
              <span>การตั้งค่า API</span>
            </button>
          </nav>

          {/* Metrics Grid */}
          <section className="metrics-grid">
            <div className="metric-card">
              <div className="metric-icon">
                <Key className="w-4 h-4" />
              </div>
              <div className="metric-content">
                <div className="metric-label">คีย์ทั้งหมด</div>
                <div className="metric-value">{statTotal}</div>
              </div>
            </div>

            <div className="metric-card">
              <div className="metric-icon">
                <ShieldCheck className="w-4 h-4 text-emerald-400" />
              </div>
              <div className="metric-content">
                <div className="metric-label">ว่างพร้อมใช้ (Unused)</div>
                <div className="metric-value text-emerald-400">{statUnused}</div>
              </div>
            </div>

            <div className="metric-card">
              <div className="metric-icon">
                <Cpu className="w-4 h-4 text-purple-400" />
              </div>
              <div className="metric-content">
                <div className="metric-label">เปิดใช้งานแล้ว (Activated)</div>
                <div className="metric-value text-purple-400">{statActivated}</div>
              </div>
            </div>

            <div className="metric-card">
              <div className="metric-icon">
                <Pause className="w-4 h-4 text-amber-400" />
              </div>
              <div className="metric-content">
                <div className="metric-label">หยุดชั่วคราว (Suspended)</div>
                <div className="metric-value text-amber-400">{statSuspended}</div>
              </div>
            </div>

            <div className="metric-card">
              <div className="metric-icon">
                <AlertCircle className="w-4 h-4 text-rose-400" />
              </div>
              <div className="metric-content">
                <div className="metric-label">ถูกระงับ (Banned)</div>
                <div className="metric-value text-rose-400">{statBanned}</div>
              </div>
            </div>
          </section>

          {/* ================================================================= */}
          {/* TAB 1: GENERATOR VIEW                                             */}
          {/* ================================================================= */}
          {activeTab === 'generator' && (
            <div className="grid-layout-2col">
              {/* Form Panel */}
              <div className="panel-card">
                <div className="panel-header">
                  <div className="panel-title">
                    <Key className="w-4 h-4 text-zinc-400" />
                    <span>สร้าง Serial Key ใหม่</span>
                  </div>
                  <div className="panel-sub">ระบบสุ่มสร้างคีย์ตามมาตรฐาน NHKV4 (Server-side Crypto)</div>
                </div>

                <form onSubmit={handleGenerateKeys} className="panel-body form-grid">
                  <div className="form-group">
                    <label>จำนวนคีย์ที่ต้องการสร้าง</label>
                    <div className="radio-pill-group">
                      {[1, 5, 10, 25].map(num => (
                        <label key={num} className="radio-pill">
                          <input
                            type="radio"
                            name="count"
                            value={num}
                            checked={genCount === num}
                            onChange={() => setGenCount(num)}
                          />
                          <span>{num} คีย์</span>
                        </label>
                      ))}
                    </div>
                  </div>

                  <div className="form-row-2">
                    <div className="form-group">
                      <label htmlFor="tierSelect">ระดับสิทธิ์ (Tier)</label>
                      <select
                        id="tierSelect"
                        className="form-select"
                        value={genTier}
                        onChange={(e) => setGenTier(e.target.value)}
                      >
                        <option value="PRO">PRO</option>
                        <option value="VIP">VIP</option>
                        <option value="TRIAL">TRIAL</option>
                      </select>
                    </div>

                    <div className="form-group">
                      <label htmlFor="durationSelect">อายุการใช้งาน (Duration)</label>
                      <select
                        id="durationSelect"
                        className="form-select"
                        value={genDuration}
                        onChange={(e) => setGenDuration(parseInt(e.target.value, 10))}
                      >
                        <option value="1">1 วัน</option>
                        <option value="3">3 วัน</option>
                        <option value="7">7 วัน</option>
                        <option value="15">15 วัน</option>
                        <option value="30">30 วัน</option>
                        <option value="90">90 วัน</option>
                        <option value="180">180 วัน</option>
                        <option value="365">365 วัน</option>
                        <option value="0">ถาวร</option>
                      </select>
                    </div>
                  </div>

                  <div className="form-group">
                    <label htmlFor="noteInput">บันทึก Note ลูกค้า / ช่องทางจำหน่าย</label>
                    <input
                      id="noteInput"
                      type="text"
                      className="form-input"
                      placeholder="เช่น ลูกค้าคุณบาส Discord หรือ Shopee Batch 1"
                      value={genNote}
                      onChange={(e) => setGenNote(e.target.value)}
                    />
                  </div>

                  <div className="pt-2">
                    <button
                      type="submit"
                      disabled={isGenerating}
                      className="btn btn-primary btn-block"
                    >
                      {isGenerating ? (
                        <>
                          <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                          <span>กำลังสร้างคีย์...</span>
                        </>
                      ) : (
                        <>
                          <Key className="w-3.5 h-3.5" />
                          <span>สร้าง Serial Keys ทันที</span>
                        </>
                      )}
                    </button>
                  </div>
                </form>
              </div>

              {/* Output Panel */}
              <div className="panel-card">
                <div className="panel-header">
                  <div className="panel-title">
                    <Terminal className="w-4 h-4 text-zinc-400" />
                    <span>ผลลัพธ์คีย์ล่าสุด ({latestGenerated.length})</span>
                  </div>
                  {latestGenerated.length > 0 && (
                    <div className="header-tools">
                      <button
                        className="btn btn-sm btn-secondary"
                        onClick={() => copyToClipboard(latestGenerated.map(k => k.key).join('\n'))}
                      >
                        <Copy className="w-3 h-3" />
                        <span>คัดลอกคีย์ทั้งหมด</span>
                      </button>
                      <button
                        className="btn btn-sm btn-secondary"
                        onClick={() => copyToClipboard(latestSql)}
                      >
                        <Terminal className="w-3 h-3" />
                        <span>คัดลอก SQL D1</span>
                      </button>
                    </div>
                  )}
                </div>

                <div className="panel-body">
                  {latestGenerated.length === 0 ? (
                    <div className="empty-state">
                      <div className="empty-icon">🔑</div>
                      <div className="empty-title">ยังไม่มีคีย์ที่สร้างในรอบนี้</div>
                      <div className="empty-sub">กดปุ่ม &quot;สร้าง Serial Keys&quot; ฝั่งซ้ายเพื่อสุ่มสร้างคีย์ใหม่</div>
                    </div>
                  ) : (
                    <>
                      <div className="keys-output-list">
                        {latestGenerated.map((item, idx) => (
                          <div key={idx} className="key-card-item">
                            <span className="key-code-text">{item.key}</span>
                            <div className="key-meta-tags">
                              <span className={`badge-tag ${item.tier.toLowerCase()}`}>{item.tier}</span>
                              <button
                                className="btn btn-sm btn-outline"
                                onClick={() => copyToClipboard(item.key, item.key)}
                              >
                                {copiedId === item.key ? (
                                  <Check className="w-3 h-3 text-emerald-400" />
                                ) : (
                                  <Copy className="w-3 h-3" />
                                )}
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>

                      {latestSql && (
                        <div className="sql-preview-box">
                          <div className="sql-header flex justify-between items-center">
                            <span>Cloudflare D1 SQL INSERT:</span>
                            <button
                              className="text-xs text-zinc-400 hover:text-white flex items-center gap-1"
                              onClick={() => copyToClipboard(latestSql)}
                            >
                              <Copy className="w-3 h-3" /> Copy
                            </button>
                          </div>
                          <pre><code>{latestSql}</code></pre>
                        </div>
                      )}
                    </>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* ================================================================= */}
          {/* TAB 2: INVENTORY VIEW                                             */}
          {/* ================================================================= */}
          {activeTab === 'inventory' && (
            <div className="panel-card">
              <div className="panel-header">
                <div className="flex items-center gap-3">
                  <div className="panel-title">
                    <Layers className="w-4 h-4 text-zinc-400" />
                    <span>คลังรายการ License Keys ({filteredKeys.length})</span>
                  </div>
                  <button
                    className="btn btn-sm btn-outline flex items-center gap-1.5 text-xs text-zinc-300 hover:text-white"
                    onClick={() => loadInventory(true)}
                    disabled={isLoadingInventory}
                    title="รีเฟรชข้อมูลจาก Cloudflare D1"
                  >
                    <RefreshCw className={`w-3.5 h-3.5 ${isLoadingInventory ? 'animate-spin text-emerald-400' : 'text-zinc-400'}`} />
                    <span>{isLoadingInventory ? 'กำลังซิงค์...' : 'รีเฟรช D1'}</span>
                  </button>
                </div>

                <div className="filter-bar">
                  <div className="search-box">
                    <Search className="w-3.5 h-3.5 text-zinc-500" />
                    <input
                      type="text"
                      placeholder="Search key, note, or HWID..."
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                    />
                  </div>

                  <div className="filter-pills">
                    {['ALL', 'ACTIVATED', 'SUSPENDED', 'UNUSED', 'BANNED'].map(f => (
                      <button
                        key={f}
                        className={`filter-pill ${statusFilter === f ? 'active' : ''}`}
                        onClick={() => setStatusFilter(f)}
                      >
                        {f === 'ALL' ? 'All' : f === 'SUSPENDED' ? 'Suspended' : f}
                      </button>
                    ))}
                  </div>
                </div>
              </div>

              <div className="panel-body table-responsive p-0">
                {filteredKeys.length === 0 ? (
                  <div className="empty-state">
                    <div className="empty-icon">📦</div>
                    <div className="empty-title">ไม่พบคีย์ในเงื่อนไขที่ค้นหา</div>
                    <div className="empty-sub">ลองปรับคำค้นหาหรือตัวกรองสถานะใหม่</div>
                  </div>
                ) : (
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>License Key</th>
                        <th>Tier</th>
                        <th>Duration</th>
                        <th>Status</th>
                        <th>Bound HWID</th>
                        <th>Note</th>
                        <th className="text-right">Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {filteredKeys.map((item, idx) => (
                        <tr key={idx}>
                          <td>
                            <span className="font-mono font-medium text-zinc-200">{item.key}</span>
                          </td>
                          <td>
                            <span className={`badge-tag ${item.tier.toLowerCase()}`}>{item.tier}</span>
                          </td>
                          <td className="text-zinc-400 text-xs">
                            {item.durationDays === 0 ? 'ถาวร' : `${item.durationDays} วัน`}
                          </td>
                          <td>
                            <span className={`status-badge ${item.status.toLowerCase()}`}>
                              {item.status}
                            </span>
                          </td>
                          <td>
                            {item.boundHwid ? (
                              <span className="font-mono text-xs text-zinc-400">{item.boundHwid}</span>
                            ) : (
                              <span className="text-zinc-600 text-xs">-</span>
                            )}
                          </td>
                          <td className="text-zinc-400 text-xs">{item.userNote || '-'}</td>
                          <td className="text-right">
                            <div className="flex items-center justify-end gap-1.5">
                              <button
                                className="btn btn-sm btn-outline"
                                onClick={() => copyToClipboard(item.key, item.key)}
                                title="Copy"
                              >
                                {copiedId === item.key ? (
                                  <Check className="w-3 h-3 text-emerald-400" />
                                ) : (
                                  <Copy className="w-3 h-3" />
                                )}
                              </button>

                              {item.status === 'SUSPENDED' ? (
                                <button
                                  className="btn btn-sm btn-warning-outline flex items-center gap-1"
                                  onClick={() => toggleSuspendKey(item.key)}
                                  title="ปลดระงับ / เปิดใช้งานต่อ"
                                >
                                  <Play className="w-3 h-3" />
                                  <span>เปิดต่อ</span>
                                </button>
                              ) : item.status !== 'BANNED' ? (
                                <button
                                  className="btn btn-sm btn-outline text-amber-400 hover:text-amber-300 flex items-center gap-1"
                                  onClick={() => toggleSuspendKey(item.key)}
                                  title="หยุดใช้งานชั่วคราว"
                                >
                                  <Pause className="w-3 h-3" />
                                  <span>หยุดชั่วคราว</span>
                                </button>
                              ) : null}

                              <button
                                className={`btn btn-sm ${item.status === 'BANNED' ? 'btn-outline' : 'btn-danger-outline'}`}
                                onClick={() => toggleBanKey(item.key)}
                              >
                                {item.status === 'BANNED' ? 'Unban' : 'Ban'}
                              </button>

                              <button
                                className="btn btn-sm btn-outline text-rose-400 hover:text-rose-300 hover:border-rose-400 p-1.5"
                                onClick={() => handleDeleteKey(item.key)}
                                title="ลบคีย์ออกจากระบบถาวร"
                              >
                                <Trash2 className="w-3.5 h-3.5" />
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            </div>
          )}

          {/* ================================================================= */}
          {/* TAB 3: DEVICES VIEW                                               */}
          {/* ================================================================= */}
          {activeTab === 'devices' && (
            <div className="panel-card">
              <div className="panel-header">
                <div className="flex items-center justify-between w-full">
                  <div>
                    <div className="panel-title flex items-center gap-2">
                      <Cpu className="w-4 h-4 text-zinc-400" />
                      <span>เครื่องลูกข่ายที่ลงทะเบียน (Bound Devices: {activeDevices.length})</span>
                    </div>
                    <div className="panel-sub">รายชื่อเครื่องคอมพิวเตอร์ที่ได้ทำการ Redeem ผูก Serial Key สำเร็จ</div>
                  </div>
                  <button
                    className="btn btn-sm btn-outline flex items-center gap-1.5 text-xs text-zinc-300 hover:text-white"
                    onClick={() => loadInventory(true)}
                    disabled={isLoadingInventory}
                    title="รีเฟรชข้อมูลจาก Cloudflare D1"
                  >
                    <RefreshCw className={`w-3.5 h-3.5 ${isLoadingInventory ? 'animate-spin text-emerald-400' : 'text-zinc-400'}`} />
                    <span>{isLoadingInventory ? 'กำลังซิงค์...' : 'รีเฟรช D1'}</span>
                  </button>
                </div>
              </div>

              <div className="panel-body table-responsive p-0">
                {activeDevices.length === 0 ? (
                  <div className="empty-state">
                    <div className="empty-icon">💻</div>
                    <div className="empty-title">ยังไม่มีเครื่องที่เปิดใช้งานในระบบ</div>
                    <div className="empty-sub">เมื่อลูกค้าเปิด Launcher แล้วกด Activate สิทธิ์จะปรากฏที่นี่โดยอัตโนมัติ</div>
                  </div>
                ) : (
                  <table className="data-table">
                    <thead>
                      <tr>
                        <th>Hardware ID (HWID)</th>
                        <th>คีย์ที่ผูก</th>
                        <th>Tier</th>
                        <th>Status</th>
                        <th>Note</th>
                        <th className="text-right">Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {activeDevices.map((item, idx) => (
                        <tr key={idx}>
                          <td>
                            <div className="flex items-center gap-2">
                              <Cpu className="w-3.5 h-3.5 text-zinc-400" />
                              <span className="font-mono text-xs text-zinc-200 font-medium">{item.boundHwid}</span>
                            </div>
                          </td>
                          <td>
                            <span className="font-mono text-xs text-zinc-400">{item.key}</span>
                          </td>
                          <td>
                            <span className={`badge-tag ${item.tier.toLowerCase()}`}>{item.tier}</span>
                          </td>
                          <td>
                            <span className={`status-badge ${item.status.toLowerCase()}`}>{item.status}</span>
                          </td>
                          <td className="text-zinc-400 text-xs">{item.userNote || '-'}</td>
                          <td className="text-right">
                            <div className="flex items-center justify-end gap-1.5">
                              <button
                                className="btn btn-sm btn-outline"
                                onClick={() => copyToClipboard(item.key, item.key)}
                                title="Copy Key"
                              >
                                {copiedId === item.key ? (
                                  <Check className="w-3 h-3 text-emerald-400" />
                                ) : (
                                  <Copy className="w-3 h-3" />
                                )}
                              </button>

                              {item.status === 'SUSPENDED' ? (
                                <button
                                  className="btn btn-sm btn-warning-outline flex items-center gap-1"
                                  onClick={() => toggleSuspendKey(item.key)}
                                  title="ปลดระงับ / เปิดใช้งานต่อ"
                                >
                                  <Play className="w-3 h-3" />
                                  <span>เปิดต่อ</span>
                                </button>
                              ) : item.status !== 'BANNED' ? (
                                <button
                                  className="btn btn-sm btn-outline text-amber-400 hover:text-amber-300 flex items-center gap-1"
                                  onClick={() => toggleSuspendKey(item.key)}
                                  title="หยุดใช้งานชั่วคราว"
                                >
                                  <Pause className="w-3 h-3" />
                                  <span>หยุดชั่วคราว</span>
                                </button>
                              ) : null}

                              <button
                                className={`btn btn-sm ${item.status === 'BANNED' ? 'btn-outline' : 'btn-danger-outline'}`}
                                onClick={() => toggleBanKey(item.key)}
                              >
                                {item.status === 'BANNED' ? 'Unban' : 'Ban'}
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                )}
              </div>
            </div>
          )}

          {/* ================================================================= */}
          {/* TAB 4: SETTINGS VIEW                                              */}
          {/* ================================================================= */}
          {activeTab === 'settings' && (
            <div className="panel-card max-w-700">
              <div className="panel-header">
                <div className="panel-title">
                  <Settings className="w-4 h-4 text-zinc-400" />
                  <span>การตั้งค่า API & คลังข้อมูล</span>
                </div>
                <div className="panel-sub">Cloudflare Worker & Workspace Backup</div>
              </div>

              <div className="panel-body form-grid">
                <div className="form-group">
                  <label htmlFor="workerUrlInput">Cloudflare Worker API Endpoint</label>
                  <input
                    id="workerUrlInput"
                    type="text"
                    className="form-input code-font text-xs"
                    value={workerUrl}
                    onChange={(e) => setWorkerUrl(e.target.value)}
                  />
                  <div className="field-hint">ที่อยู่ Worker Gateway ที่เชื่อมกับ Cloudflare D1</div>
                </div>

                <div className="form-group">
                  <label>การเปลี่ยนบัญชีผู้ดูแล (Admin Credentials)</label>
                  <div className="p-3 bg-[#111114] border border-zinc-800 rounded text-xs text-zinc-400 leading-relaxed font-mono">
                    💡 กำหนดชื่อผู้ใช้และรหัสผ่านได้ที่ไฟล์ <span className="text-zinc-200">.env.local</span> (หรือใน Vercel Environment Variables):
                    <br />
                    <span className="text-zinc-200">ADMIN_USERNAME=&quot;admin&quot;</span>
                    <br />
                    <span className="text-zinc-200">ADMIN_PASSWORD=&quot;your_new_password&quot;</span>
                  </div>
                </div>

                <div className="divider"></div>

                <div className="form-group">
                  <label>สำรองข้อมูลคลังคีย์ (Backup & Export)</label>
                  <div className="button-group">
                    <button
                      type="button"
                      className="btn btn-secondary"
                      onClick={() => {
                        const blob = new Blob([JSON.stringify(allKeys, null, 2)], { type: 'application/json' });
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = `nhk_keys_backup_${new Date().toISOString().split('T')[0]}.json`;
                        a.click();
                        addToast('ดาวน์โหลดไฟล์ Backup สำเร็จ');
                      }}
                    >
                      <Download className="w-3.5 h-3.5" />
                      <span>ดาวน์โหลด JSON Backup</span>
                    </button>

                    <button
                      type="button"
                      className="btn btn-danger-outline"
                      onClick={() => {
                        if (confirm('คุณแน่ใจหรือไม่ว่าต้องการล้างแคชคีย์ในเบราว์เซอร์?')) {
                          localStorage.removeItem('nhk_admin_keys_v4');
                          setAllKeys([]);
                          addToast('ล้างข้อมูลเรียบร้อย');
                        }
                      }}
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                      <span>ล้างแคชในเบราว์เซอร์</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

        </div>
      </main>

      {/* Toast Notifications */}
      <div className="toast-container">
        {toasts.map(t => (
          <div key={t.id} className="toast">
            {t.type === 'error' ? (
              <AlertCircle className="w-4 h-4 text-rose-400" />
            ) : (
              <Check className="w-4 h-4 text-emerald-400" />
            )}
            <span>{t.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
