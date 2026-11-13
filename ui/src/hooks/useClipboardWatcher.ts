import { useEffect, useRef } from "react";
import { api } from "../lib/api";
import { extractUrl, looksDownloadable } from "../lib/jobs";

/**
 * When the window regains focus, look at the clipboard and offer to download
 * a direct file link (once per distinct link).
 */
export function useClipboardWatcher(
  enabled: boolean,
  isKnown: (url: string) => boolean,
  onFound: (url: string) => void,
) {
  const last = useRef<string | null>(null);
  const knownRef = useRef(isKnown);
  const foundRef = useRef(onFound);
  knownRef.current = isKnown;
  foundRef.current = onFound;

  useEffect(() => {
    if (!enabled) return;
    let busy = false;

    const check = async () => {
      if (busy) return;
      busy = true;
      try {
        const text = await api.getClipboard();
        const url = extractUrl(text);
        if (url && url !== last.current && looksDownloadable(url) && !knownRef.current(url)) {
          last.current = url;
          foundRef.current(url);
        }
      } catch {
        /* clipboard unavailable */
      } finally {
        busy = false;
      }
    };

    // Remember what is already in the clipboard at startup so it is not offered.
    api
      .getClipboard()
      .then((t) => {
        last.current = extractUrl(t);
      })
      .catch(() => undefined);

    window.addEventListener("focus", check);
    return () => window.removeEventListener("focus", check);
  }, [enabled]);
}
