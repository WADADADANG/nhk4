/**
 * dialog.js - Universal Custom Confirm & Alert Modal
 * Clean Dark Matte Theme (Strictly No Neon)
 */
(function() {
  function escapeHtml(str) {
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  window.showConfirm = function(options) {
    // If running inside iframe and parent has showConfirm, delegate to top window for optimal viewport coverage
    if (window.parent && window.parent !== window && typeof window.parent.showConfirm === 'function') {
      return window.parent.showConfirm(options);
    }

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
      let hintHead = opts.hintHead || 'ข้อมูลสำคัญ:';

      if (!hintText && rawMsg.includes('\n\n')) {
        const parts = rawMsg.split('\n\n');
        primaryText = parts[0];
        hintText = parts.slice(1).join('\n\n');
      } else if (!hintText && rawMsg.includes('\n')) {
        const parts = rawMsg.split('\n');
        primaryText = parts[0];
        hintText = parts.slice(1).join('\n');
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

  window.showAlert = function(options) {
    if (window.parent && window.parent !== window && typeof window.parent.showAlert === 'function') {
      return window.parent.showAlert(options);
    }

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
      let hintHead = opts.hintHead || 'หมายเหตุ:';

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
})();
