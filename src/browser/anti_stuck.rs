/// JavaScript script payload to inject into browser pages on launch
/// Detects physical window blur and notifies NodeHotkey/Rust to release held keys
pub const ANTI_STUCK_INJECTION_SCRIPT: &str = r#"
(() => {
    if (window.__nodeHotkeyAntiStuckInjected) return;
    window.__nodeHotkeyAntiStuckInjected = true;

    const heldPhysicalKeys = new Set();
    const heldPhysicalButtons = new Set();

    window.addEventListener('keydown', (e) => {
        if (e.isTrusted) heldPhysicalKeys.add(e.code);
    }, true);

    window.addEventListener('keyup', (e) => {
        if (e.isTrusted) heldPhysicalKeys.delete(e.code);
    }, true);

    window.addEventListener('mousedown', (e) => {
        if (e.isTrusted) heldPhysicalButtons.add(e.button);
    }, true);

    window.addEventListener('mouseup', (e) => {
        if (e.isTrusted) heldPhysicalButtons.delete(e.button);
    }, true);

    const releaseInputs = () => {
        if (heldPhysicalKeys.size > 0 || heldPhysicalButtons.size > 0) {
            heldPhysicalKeys.clear();
            heldPhysicalButtons.clear();
            if (document.pointerLockElement) {
                try { document.exitPointerLock(); } catch(e) {}
            }
        }
    };

    window.addEventListener('blur', releaseInputs, true);
    window.addEventListener('focusout', releaseInputs, true);
    document.addEventListener('visibilitychange', () => {
        if (document.hidden) releaseInputs();
    }, true);
})();
"#;
