import { useEffect, useRef, useState } from "react";
import { getInspectionPhotoPreview } from "../lib/files-api";

type Props = {
  photoId: string;
  alt: string;
};

type PreviewState = "idle" | "loading" | "missing" | "limited" | "ready" | "error";

export function InspectionPhotoPreview({ photoId, alt }: Props) {
  const rootRef = useRef<HTMLDivElement>(null);
  const [state, setState] = useState<PreviewState>("idle");
  const [url, setUrl] = useState<string | null>(null);

  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    let cancelled = false;
    let objectUrl: string | null = null;

    const load = async () => {
      if (cancelled) return;
      setState("loading");
      try {
        const preview = await getInspectionPhotoPreview(photoId);
        if (cancelled) return;
        if (!preview.fileExists) {
          setState("missing");
          return;
        }
        if (!preview.bytes?.length) {
          setState(preview.previewLimited ? "limited" : "error");
          return;
        }
        const blob = new Blob([new Uint8Array(preview.bytes)], { type: preview.mimeType || "application/octet-stream" });
        objectUrl = URL.createObjectURL(blob);
        setUrl(objectUrl);
        setState("ready");
      } catch {
        if (!cancelled) setState("error");
      }
    };

    if (typeof IntersectionObserver === "undefined") {
      void load();
    } else {
      const observer = new IntersectionObserver((entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          observer.disconnect();
          void load();
        }
      }, { rootMargin: "120px" });
      observer.observe(root);
      return () => {
        cancelled = true;
        observer.disconnect();
        if (objectUrl) URL.revokeObjectURL(objectUrl);
      };
    }

    return () => {
      cancelled = true;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
  }, [photoId]);

  return <div ref={rootRef} className={`photo-thumbnail photo-thumbnail-${state}`}>
    {state === "ready" && url ? <img src={url} alt={alt} loading="lazy" /> : <span>{state === "missing" ? "Arquivo não encontrado" : state === "limited" ? "Prévia não carregada" : state === "error" ? "Prévia indisponível" : "Carregando prévia…"}</span>}
  </div>;
}
