// LODM browser integration (Manifest V3).
// Talks to the local LODM API (127.0.0.1:<port>) using the secret token from LODM > Settings.

const DEFAULTS = { port: 4141, token: "", capture: true, minSizeMB: 0 };

async function getConfig() {
  return { ...DEFAULTS, ...(await chrome.storage.local.get(DEFAULTS)) };
}

async function lodmFetch(cfg, path, init = {}) {
  const res = await fetch(`http://127.0.0.1:${cfg.port}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      "X-LODM-Token": cfg.token,
      ...(init.headers || {}),
    },
  });
  return res;
}

async function isLodmReachable(cfg) {
  if (!cfg.token) return false;
  try {
    const res = await lodmFetch(cfg, "/api/ping");
    return res.ok;
  } catch {
    return false;
  }
}

async function cookieHeader(url) {
  try {
    const cookies = await chrome.cookies.getAll({ url });
    return cookies.map((c) => `${c.name}=${c.value}`).join("; ");
  } catch {
    return "";
  }
}

/** Send a URL to LODM. Resolves true on success. */
async function sendToLodm(cfg, { url, referrer, filename }) {
  const body = {
    url,
    filename: filename || undefined,
    referer: referrer || undefined,
    cookie: (await cookieHeader(url)) || undefined,
    user_agent: navigator.userAgent,
  };
  try {
    const res = await lodmFetch(cfg, "/api/downloads", { method: "POST", body: JSON.stringify(body) });
    return res.ok;
  } catch {
    return false;
  }
}

function notify(title, message) {
  chrome.notifications.create({
    type: "basic",
    iconUrl: "icon.png",
    title,
    message,
  });
}

// Browser download -> LODM (only when LODM is running and capture is on).
chrome.downloads.onCreated.addListener(async (item) => {
  const cfg = await getConfig();
  if (!cfg.capture) return;
  const url = item.finalUrl || item.url;
  if (!/^https?:\/\//i.test(url)) return;
  if (cfg.minSizeMB > 0 && item.fileSize > 0 && item.fileSize < cfg.minSizeMB * 1024 * 1024) return;
  if (!(await isLodmReachable(cfg))) return; // leave the browser download alone

  const name = item.filename ? item.filename.split(/[\\/]/).pop() : "";
  const ok = await sendToLodm(cfg, { url, referrer: item.referrer, filename: name });
  if (ok) {
    chrome.downloads.cancel(item.id, () => chrome.downloads.erase({ id: item.id }));
  }
});

// Right-click on a link -> "Download with LODM".
chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "lodm-link",
    title: "Download with LODM",
    contexts: ["link", "video", "audio", "image"],
  });
});

chrome.contextMenus.onClicked.addListener(async (info, tab) => {
  if (info.menuItemId !== "lodm-link") return;
  const url = info.linkUrl || info.srcUrl;
  if (!url) return;
  const cfg = await getConfig();
  const ok = (await isLodmReachable(cfg)) && (await sendToLodm(cfg, { url, referrer: tab?.url }));
  notify("LODM", ok ? "Download added." : "Could not reach LODM. Is it running and is the key set?");
});
