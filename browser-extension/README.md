# LODM browser extension

Chrome / Edge / Brave (Manifest V3):

1. Open `chrome://extensions`, enable Developer mode, **Load unpacked**, pick this folder.
2. In LODM open Settings > Browser extension, copy the **Key**.
3. Click the extension icon, paste the key (and port if changed), Save. It should say *Connected to LODM*.

Downloads started in the browser are forwarded to LODM (with cookies and referer) while LODM is running; if LODM is closed the browser keeps handling them. Right-click any link > **Download with LODM**.

The local API only accepts requests that carry the secret key, and only extension pages may read its responses.

