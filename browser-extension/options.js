const $ = (id) => document.getElementById(id);
const DEFAULTS = { port: 4141, token: "", capture: true, minSizeMB: 0 };

async function load() {
  const cfg = { ...DEFAULTS, ...(await chrome.storage.local.get(DEFAULTS)) };
  $("port").value = cfg.port;
  $("token").value = cfg.token;
  $("capture").checked = cfg.capture;
  $("minsize").value = cfg.minSizeMB;
}

async function save() {
  const port = Number($("port").value) || DEFAULTS.port;
  const token = $("token").value.trim();
  await chrome.storage.local.set({
    port,
    token,
    capture: $("capture").checked,
    minSizeMB: Math.max(0, Number($("minsize").value) || 0),
  });

  // Verify the connection right away.
  let msg = "Saved.";
  try {
    const res = await fetch(`http://127.0.0.1:${port}/api/ping`, { headers: { "X-LODM-Token": token } });
    msg = res.ok ? "Saved. Connected to LODM." : res.status === 401 ? "Saved. Wrong key." : `Saved. HTTP ${res.status}`;
  } catch {
    msg = "Saved. LODM is not reachable.";
  }
  $("status").textContent = msg;
}

$("save").addEventListener("click", save);
load();
