import { useCallback, useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { GetTaskQueue } from "@storyteller/tauri-api";
import { getModelDisplayName, getProviderDisplayName } from "@storyteller/model-list";
import type { GalleryItem } from "@storyteller/ui-generation-list";

/// Completed generations from the local task database.
///
/// Generation results always land in the local task database, so this is the
/// desktop's "local-first" library source: it works without an account (Venice
/// BYOK saves files locally) and complements the cloud library when logged in
/// (dedupe by media token, see `mergeUniqueGalleryItems`).
export function useLocalCompletedTasks(options: { mediaClasses?: string[] } = {}): GalleryItem[] {
  const { mediaClasses } = options;
  const mediaClassesKey = mediaClasses ? [...mediaClasses].sort().join(",") : "";
  const [items, setItems] = useState<GalleryItem[]>([]);

  const load = useCallback(async () => {
    try {
      const { tasks } = await GetTaskQueue();
      const wantedClasses = mediaClassesKey ? mediaClassesKey.split(",") : null;

      const completed: GalleryItem[] = [];
      for (const task of tasks) {
        if (String(task.task_status) !== "complete_success") continue;
        const completedItem = task.completed_item;
        if (!completedItem) continue;

        const mediaClass = String(completedItem.media_file_class ?? "");
        if (!mediaClass) continue;
        if (wantedClasses && !wantedClasses.includes(mediaClass)) continue;

        const cdnUrl = completedItem.primary_media_file?.cdn_url;
        if (!cdnUrl) continue;

        const modelId = task.model_type ? String(task.model_type) : undefined;
        const providerKey = task.provider ? String(task.provider).toLowerCase() : undefined;
        const label = [modelId ? getModelDisplayName(modelId) : undefined, providerKey ? getProviderDisplayName(providerKey) : undefined].filter(Boolean).join(" · ");

        completed.push({
          id: completedItem.primary_media_file.token,
          label: label || "Generation",
          thumbnail: null,
          fullImage: cdnUrl,
          createdAt: (task.completed_at ?? task.updated_at ?? task.created_at).toISOString(),
          mediaClass,
          modelId,
          isLocalTask: true,
        });
      }

      // Newest first.
      completed.sort(
        (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
      );

      setItems(completed);
    } catch (error) {
      console.warn("Could not load locally completed tasks", error);
    }
  }, [mediaClassesKey]);

  useEffect(() => {
    load();

    const onQueueUpdate = () => load();
    window.addEventListener("task-queue-update", onQueueUpdate);

    let unlisten: (() => void) | undefined;
    listen("generation-complete-event", () => load()).then((fn) => {
      unlisten = fn;
    });

    return () => {
      window.removeEventListener("task-queue-update", onQueueUpdate);
      unlisten?.();
    };
  }, [load]);

  return items;
}

/** Merge gallery items, keeping the first entry per id. */
export function mergeUniqueGalleryItems(
  primary: GalleryItem[],
  extra: GalleryItem[],
): GalleryItem[] {
  const seen = new Set(primary.map((item) => item.id));
  const merged = [...primary];
  for (const item of extra) {
    if (seen.has(item.id)) continue;
    seen.add(item.id);
    merged.push(item);
  }
  return merged;
}

/** Merge + sort by createdAt (descending), keeping the first entry per id. */
export function mergeSortedGalleryItems(
  primary: GalleryItem[],
  extra: GalleryItem[],
): GalleryItem[] {
  const merged = mergeUniqueGalleryItems(primary, extra);
  merged.sort(
    (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
  );
  return merged;
}

/** Convenience hook: local completed items merged before cloud gallery items. */
export function useLocalFirstGalleryItems(
  cloudItems: GalleryItem[],
  mediaClasses: string[],
): GalleryItem[] {
  const localItems = useLocalCompletedTasks({ mediaClasses });
  return useMemo(
    () => mergeUniqueGalleryItems(cloudItems, localItems),
    [cloudItems, localItems],
  );
}
