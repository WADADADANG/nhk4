/**
 * bridge.js - NodeHotkey Rust Native Core Web & Tauri Bridge
 * Polyfills window.launcherAPI using Rust Axum HTTP / SSE & Tauri IPC
 */
(function() {
  if (window.launcherAPI) return; // Already present in Electron preload

  console.log('🦀 [NodeHotkey Bridge] Initializing Rust Native API Bridge...');

  // 🍞 Modern Floating Toast Notification System
  window.showToast = function(message, type = 'info', title = null, duration = 3800) {
    let container = document.getElementById('toast-container');
    if (!container) {
      container = document.createElement('div');
      container.id = 'toast-container';
      container.className = 'toast-container';
      document.body.appendChild(container);
    }

    let tType = (type || 'info').toLowerCase();
    let iconChar = 'ℹ️';
    let defaultTitle = 'แจ้งเตือน';

    if (tType === 'success') {
      iconChar = '✓';
      defaultTitle = 'ดำเนินการสำเร็จ';
    } else if (tType === 'warning') {
      iconChar = '⚠️';
      defaultTitle = 'ข้อควรระวัง';
    } else if (tType === 'error') {
      iconChar = '✕';
      defaultTitle = 'เกิดข้อผิดพลาด';
    }

    const tTitle = title || defaultTitle;
    const toastEl = document.createElement('div');
    toastEl.className = `toast-item toast-${tType}`;

    const iconDiv = document.createElement('div');
    iconDiv.className = 'toast-icon-wrap';
    iconDiv.textContent = iconChar;

    const contentDiv = document.createElement('div');
    contentDiv.className = 'toast-content';

    const titleDiv = document.createElement('div');
    titleDiv.className = 'toast-title';
    titleDiv.textContent = tTitle;

    const msgDiv = document.createElement('div');
    msgDiv.className = 'toast-message';
    msgDiv.textContent = String(message || '');

    contentDiv.appendChild(titleDiv);
    contentDiv.appendChild(msgDiv);

    const closeBtn = document.createElement('button');
    closeBtn.type = 'button';
    closeBtn.className = 'toast-close-btn';
    closeBtn.title = 'ปิด';
    closeBtn.textContent = '✕';

    let closed = false;
    function closeToast() {
      if (closed) return;
      closed = true;
      toastEl.classList.add('toast-closing');
      setTimeout(() => {
        if (toastEl.parentNode) toastEl.parentNode.removeChild(toastEl);
      }, 240);
    }

    closeBtn.onclick = closeToast;

    toastEl.appendChild(iconDiv);
    toastEl.appendChild(contentDiv);
    toastEl.appendChild(closeBtn);

    container.appendChild(toastEl);

    if (duration > 0) {
      setTimeout(closeToast, duration);
    }

    return toastEl;
  };

  function escapeHtml(str) {
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  // 💬 Modern Clean Dark Matte Confirm Dialog Modal (Style 2 Standard)
  window.showConfirm = function(options) {
    return new Promise((resolve) => {
      let opts = options;
      if (typeof opts === 'string') {
        opts = { message: opts };
      }
      opts = opts || {};

      const rawMsg = String(opts.message || '');
      const isDanger = opts.type === 'danger' || rawMsg.includes('ลบ') || rawMsg.includes('ล้าง') || rawMsg.toLowerCase().includes('delete') || rawMsg.toLowerCase().includes('clear');
      const dialogType = opts.type || (isDanger ? 'danger' : 'info');
      const title = opts.title || (isDanger ? 'ยืนยันการดำเนินการ' : 'แจ้งเตือน');
      const icon = opts.icon || (isDanger ? '🗑️' : (dialogType === 'warning' ? '⚠️' : 'ℹ️'));
      
      let cancelText = opts.cancelText || 'ยกเลิก';
      const cleanCancel = cancelText.replace(/^[✕✖]\s*/, '');
      const cancelBtnHtml = `✕ ${escapeHtml(cleanCancel)}`;

      let confirmText = opts.confirmText || (isDanger ? 'ยืนยันดำเนินการ' : 'ตกลง');
      const confirmIcon = opts.confirmIcon || (isDanger ? '🗑️' : (dialogType === 'warning' ? '⚠️' : '✓'));
      const hasLeadingEmoji = /^[\p{Emoji}\u2000-\u32ff]/u.test(confirmText);
      const confirmBtnHtml = hasLeadingEmoji
        ? escapeHtml(confirmText)
        : `<span>${confirmIcon}</span> <span>${escapeHtml(confirmText)}</span>`;

      // Split primary message and hint callout if multiple lines or separated by \n\n
      let primaryText = rawMsg;
      let hintText = opts.hint || '';
      const isEn = (typeof currentLang !== 'undefined' && currentLang === 'en');
      let hintHead = opts.hintHead || (isEn ? 'Important:' : 'ข้อมูลสำคัญ:');

      if (!hintText && rawMsg.includes('\n\n')) {
        const parts = rawMsg.split('\n\n');
        primaryText = parts[0];
        hintText = parts.slice(1).join('\n\n');
      } else if (!hintText && rawMsg.includes('\n')) {
        const parts = rawMsg.split('\n');
        primaryText = parts[0];
        hintText = parts.slice(1).join('\n');
      }

      // Create overlay
      const overlay = document.createElement('div');
      overlay.className = 'nhk-dialog-overlay';

      const card = document.createElement('div');
      card.className = `nhk-dialog-card type-${dialogType}`;

      // Header
      const header = document.createElement('div');
      header.className = 'nhk-dialog-header';
      header.innerHTML = `
        <div class="nhk-dialog-title-group">
          <span class="nhk-dialog-icon">${icon}</span>
          <span class="nhk-dialog-title">${escapeHtml(title)}</span>
        </div>
        <button type="button" class="nhk-dialog-close-btn" title="ปิด">✕</button>
      `;

      // Body
      const body = document.createElement('div');
      body.className = 'nhk-dialog-body';

      let bodyHtml = `<div class="nhk-dialog-main-text">${escapeHtml(primaryText)}</div>`;
      if (hintText) {
        bodyHtml += `
          <div class="nhk-dialog-hint-box">
            💡 <strong class="nhk-dialog-hint-head">${escapeHtml(hintHead)}</strong>
            <span>${escapeHtml(hintText)}</span>
          </div>
        `;
      }
      body.innerHTML = bodyHtml;

      // Footer
      const footer = document.createElement('div');
      footer.className = 'nhk-dialog-footer';

      const cancelBtn = document.createElement('button');
      cancelBtn.type = 'button';
      cancelBtn.className = 'nhk-dialog-btn nhk-dialog-btn-cancel';
      cancelBtn.innerHTML = cancelBtnHtml;

      const confirmBtn = document.createElement('button');
      confirmBtn.type = 'button';
      const confirmColorClass = dialogType === 'danger' ? 'nhk-dialog-btn-danger' : (dialogType === 'warning' ? 'nhk-dialog-btn-warning' : 'nhk-dialog-btn-confirm');
      confirmBtn.className = `nhk-dialog-btn ${confirmColorClass}`;
      confirmBtn.innerHTML = confirmBtnHtml;

      footer.appendChild(cancelBtn);
      footer.appendChild(confirmBtn);

      card.appendChild(header);
      card.appendChild(body);
      card.appendChild(footer);
      overlay.appendChild(card);
      document.body.appendChild(overlay);

      confirmBtn.focus();

      let settled = false;
      function cleanup(result) {
        if (settled) return;
        settled = true;
        document.removeEventListener('keydown', onKey);
        if (overlay.parentNode) {
          overlay.parentNode.removeChild(overlay);
        }
        resolve(result);
      }

      function onKey(e) {
        if (e.key === 'Escape') {
          e.preventDefault();
          cleanup(false);
        } else if (e.key === 'Enter') {
          if (document.activeElement === cancelBtn) {
            cleanup(false);
          } else {
            cleanup(true);
          }
        }
      }

      document.addEventListener('keydown', onKey);

      const closeBtn = header.querySelector('.nhk-dialog-close-btn');
      if (closeBtn) closeBtn.onclick = () => cleanup(false);
      cancelBtn.onclick = () => cleanup(false);
      confirmBtn.onclick = () => cleanup(true);

      overlay.onclick = (e) => {
        if (e.target === overlay) {
          cleanup(false);
        }
      };
    });
  };

  // 💬 Modern Clean Dark Matte Alert Modal (Style 2 Standard)
  window.showAlert = function(options) {
    return new Promise((resolve) => {
      let opts = options;
      if (typeof opts === 'string') {
        opts = { message: opts };
      }
      opts = opts || {};

      const rawMsg = String(opts.message || '');
      const title = opts.title || 'แจ้งเตือน';
      const icon = opts.icon || 'ℹ️';
      const okText = opts.okText || 'เข้าใจแล้ว';
      const isDanger = opts.type === 'danger';
      const isWarning = opts.type === 'warning';
      const dialogType = opts.type || (isDanger ? 'danger' : (isWarning ? 'warning' : 'info'));

      let primaryText = rawMsg;
      let hintText = opts.hint || '';
      const isEn = (typeof currentLang !== 'undefined' && currentLang === 'en');
      let hintHead = opts.hintHead || (isEn ? 'Notice:' : 'หมายเหตุ:');

      if (!hintText && rawMsg.includes('\n\n')) {
        const parts = rawMsg.split('\n\n');
        primaryText = parts[0];
        hintText = parts.slice(1).join('\n\n');
      }

      const overlay = document.createElement('div');
      overlay.className = 'nhk-dialog-overlay';

      const card = document.createElement('div');
      card.className = `nhk-dialog-card type-${dialogType}`;

      const header = document.createElement('div');
      header.className = 'nhk-dialog-header';
      header.innerHTML = `
        <div class="nhk-dialog-title-group">
          <span class="nhk-dialog-icon">${icon}</span>
          <span class="nhk-dialog-title">${escapeHtml(title)}</span>
        </div>
        <button type="button" class="nhk-dialog-close-btn" title="ปิด">✕</button>
      `;

      const body = document.createElement('div');
      body.className = 'nhk-dialog-body';

      let bodyHtml = `<div class="nhk-dialog-main-text">${escapeHtml(primaryText)}</div>`;
      if (hintText) {
        bodyHtml += `
          <div class="nhk-dialog-hint-box">
            💡 <strong class="nhk-dialog-hint-head">${escapeHtml(hintHead)}</strong>
            <span>${escapeHtml(hintText)}</span>
          </div>
        `;
      }
      body.innerHTML = bodyHtml;

      const footer = document.createElement('div');
      footer.className = 'nhk-dialog-footer';

      const okBtn = document.createElement('button');
      okBtn.type = 'button';
      const okColorClass = dialogType === 'danger' ? 'nhk-dialog-btn-danger' : (dialogType === 'warning' ? 'nhk-dialog-btn-warning' : 'nhk-dialog-btn-confirm');
      okBtn.className = `nhk-dialog-btn ${okColorClass}`;
      okBtn.innerHTML = `<span>✓</span> <span>${escapeHtml(okText)}</span>`;

      footer.appendChild(okBtn);
      card.appendChild(header);
      card.appendChild(body);
      card.appendChild(footer);
      overlay.appendChild(card);
      document.body.appendChild(overlay);

      okBtn.focus();

      let settled = false;
      function cleanup() {
        if (settled) return;
        settled = true;
        document.removeEventListener('keydown', onKey);
        if (overlay.parentNode) {
          overlay.parentNode.removeChild(overlay);
        }
        resolve();
      }

      function onKey(e) {
        if (e.key === 'Escape' || e.key === 'Enter') {
          cleanup();
        }
      }

      document.addEventListener('keydown', onKey);
      const closeBtn = header.querySelector('.nhk-dialog-close-btn');
      if (closeBtn) closeBtn.onclick = cleanup;
      okBtn.onclick = cleanup;
      overlay.onclick = (e) => {
        if (e.target === overlay) cleanup();
      };
    });
  };

  // Modern override for browser window.alert
  window.alert = function(msg) {
    if (!msg) return;
    const str = String(msg);
    let type = 'warning';
    let title = 'แจ้งเตือน';
    if (str.includes('สำเร็จ') || str.includes('Success') || str.includes('🎉') || str.includes('✓')) {
      type = 'success';
      title = 'สำเร็จ (Success)';
    } else if (str.includes('ไม่สามารถ') || str.includes('ผิดพลาด') || str.includes('Error') || str.includes('Failed') || str.includes('❌') || str.includes('🔒')) {
      type = 'error';
      title = 'ข้อผิดพลาด (Error)';
    }
    window.showToast(str, type, title, 3800);
  };

  const logSubscribers = new Set();
  const statusSubscribers = new Set();
  const diagSubscribers = new Set();
  const overlaySubscribers = new Set();

  function getServerUrl(path) {
    const origin = window.location.origin;
    if (origin.startsWith('http')) {
      return origin + path;
    }
    return 'http://127.0.0.1:3088' + path;
  }

  // Connect to SSE Telemetry Stream
  let sseSource = null;
  function connectSse() {
    try {
      sseSource = new EventSource(getServerUrl('/api/profiles/stream'));
      sseSource.onmessage = function(e) {
        try {
          const data = JSON.parse(e.data);
          if (data.type === 'log') {
            logSubscribers.forEach(cb => cb(data));
          } else if (data.type === 'signal') {
            overlaySubscribers.forEach(cb => cb(data));
          } else if (data.type === 'status_update') {
            statusSubscribers.forEach(cb => cb(data.data));
            if (data.data) {
              diagSubscribers.forEach(cb => cb({ serverOnline: true, port: data.data.port, ...data.data }));
            }
          }
        } catch(err) {}
      };
      sseSource.onerror = function() {
        if (sseSource) sseSource.close();
        diagSubscribers.forEach(cb => cb({ serverOnline: false, error: 'SSE connection lost' }));
        setTimeout(connectSse, 2500);
      };
    } catch(err) {
      setTimeout(connectSse, 3000);
    }
  }
  connectSse();

  window.launcherAPI = {
    // Process Control
    startBot: async function() {
      const res = await fetch(getServerUrl('/api/engine/start'), { method: 'POST' });
      const data = await res.json().catch(() => ({ success: false, error: 'Unknown server error' }));
      if (!res.ok || data.success === false) {
        window.showToast(data.error || '🔒 Cannot start engine: Invalid or missing license. Please check Settings.', 'error', 'ไม่สามารถเริ่มบอทได้');
        statusSubscribers.forEach(cb => cb({ running: false, isRunning: false }));
        return data;
      }
      statusSubscribers.forEach(cb => cb({ running: true, isRunning: true }));
      return data;
    },
    stopBot: async function() {
      const res = await fetch(getServerUrl('/api/engine/stop'), { method: 'POST' });
      const data = await res.json().catch(() => ({ success: true }));
      statusSubscribers.forEach(cb => cb({ running: false, isRunning: false }));
      return data;
    },
    restartBot: async function() {
      await fetch(getServerUrl('/api/engine/stop'), { method: 'POST' });
      await new Promise(r => setTimeout(r, 400));
      return await fetch(getServerUrl('/api/engine/start'), { method: 'POST' });
    },
    getBotStatus: async function() {
      const res = await fetch(getServerUrl('/api/status'));
      return await res.json();
    },

    // Config
    getGlobalConfig: async function() {
      const res = await fetch(getServerUrl('/api/config'));
      return await res.json();
    },
    saveGlobalSettings: async function(settings) {
      const current = await this.getGlobalConfig();
      current.globalSettings = Object.assign({}, current.globalSettings, settings);
      return await fetch(getServerUrl('/api/config'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(current)
      });
    },

    // Logs & Storage
    openLogFolder: async function() { console.log('Open logs requested'); },
    getLogPath: async function() { return 'logs/'; },
    openScreenshotsFolder: async function() { console.log('Open screenshots requested'); },
    getStorageStats: async function() {
      return {
        logs: { fileCount: 0, formattedSize: '0 B' },
        screenshots: { fileCount: 0, formattedSize: '0 B' },
        totalFiles: 0,
        formattedTotal: '0 B'
      };
    },
    clearLogs: async function() { return { deletedFiles: 0, formattedFreed: '0 B' }; },
    clearScreenshots: async function() { return { deletedFiles: 0, formattedFreed: '0 B' }; },
    clearAllStorage: async function() { return { deletedFiles: 0, formattedFreed: '0 B' }; },

    // Window Controls
    minimizeWindow: function() {
      fetch(getServerUrl('/api/window/minimize'), { method: 'POST' }).catch(function() {});
    },
    maximizeWindow: function() {
      fetch(getServerUrl('/api/window/maximize'), { method: 'POST' }).catch(function() {});
    },
    closeWindow: function() {
      fetch(getServerUrl('/api/window/hide'), { method: 'POST' }).catch(function() {});
    },

    // Updater
    checkUpdate: async function() { return { hasUpdate: false }; },
    downloadUpdate: async function() { return true; },
    applyUpdate: async function() { return true; },
    hotReloadUi: async function() { window.location.reload(); },
    restartEngine: async function() { return this.restartBot(); },
    relaunchApp: async function() { window.location.reload(); },

    // External
    openWebDashboard: async function() { window.open(getServerUrl('/'), '_blank'); },
    openExternal: async function(url) { window.open(url, '_blank'); },

    // PiP
    togglePiP: async function(clientId) { console.log('togglePiP', clientId); },
    closePiP: async function(clientId) { console.log('closePiP', clientId); },

    // Overlay Window Controls
    toggleOverlay: async function() {
      return fetch(getServerUrl('/api/window/overlay/toggle'), { method: 'POST' }).catch(() => {});
    },
    showOverlay: async function() {
      return fetch(getServerUrl('/api/window/overlay/show'), { method: 'POST' }).catch(() => {});
    },
    hideOverlay: async function() {
      return fetch(getServerUrl('/api/window/overlay/hide'), { method: 'POST' }).catch(() => {});
    },
    resizeOverlay: async function(width, height) {
      return fetch(getServerUrl('/api/window/overlay/resize'), {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ width, height })
      }).catch(() => {});
    },
    closeOverlay: async function() {
      return fetch(getServerUrl('/api/window/overlay/hide'), { method: 'POST' }).catch(() => {});
    },

    // Event Subscriptions
    onLogMessage: function(cb) {
      logSubscribers.add(cb);
      return () => logSubscribers.delete(cb);
    },
    onStatusChange: function(cb) {
      statusSubscribers.add(cb);
      return () => statusSubscribers.delete(cb);
    },
    onDiagnosticsUpdate: function(cb) {
      diagSubscribers.add(cb);
      return () => diagSubscribers.delete(cb);
    },
    onOverlayUpdate: function(cb) {
      overlaySubscribers.add(cb);
      return () => overlaySubscribers.delete(cb);
    },
    onHotReload: function(cb) {
      return () => {};
    }
  };

  console.log('✅ [NodeHotkey Bridge] Rust Native API Bridge mounted on window.launcherAPI successfully!');
})();
