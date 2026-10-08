import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { GetTaskQueue } from "@storyteller/tauri-api";
import { getModelDisplayName } from "@storyteller/model-list";
import type { GalleryItem } from "@storyteller/ui-generation-list";

/// Completed Venice audio tasks from the local task database.
///
/// The audio page's jobs feed is an HTTP backend feed, so BYOK (Venice)
/// results never appear there. The local task rows keep the result (uploaded
/// CDN URL or a local asset URL), which also makes history persist across
/// restarts without an ArtCraft account.
export function useLocalCompletedAudioTasks(): GalleryItem[] {
  const [items, setItems] = useState<GalleryItem[]>([]);

  const load = useCallback(async () => {
    try {
      const { tasks } = await GetTaskQueue();

      const completed: GalleryItem[] = [];
      for (const task of tasks) {
        if (String(task.task_status) !== "complete_success") continue;
        if (String(task.provider) !== "venice") continue;
        const completedItem = task.completed_item;
        if (!completedItem || String(completedItem.media_file_class) !== "audio") continue;
        const cdnUrl = completedItem.primary_media_file?.cdn_url;
        if (!cdnUrl) continue;

        const modelId = task.model_type ? String(task.model_type) : undefined;
        completed.push({
          id: completedItem.primary_media_file.token,
          label: modelId ? `Venice · ${getModelDisplayName(modelId)}` : "Venice audio",
          thumbnail: null,
          fullImage: cdnUrl,
          createdAt: (
            task.completed_at ??
            task.updated_at ??
            task.created_at
          ).toISOString(),
          mediaClass: "audio",
          modelId,
        });
      }

      setItems(completed);
    } catch (error) {
      console.warn("Could not load locally completed audio tasks", error);
    }
  }, []);

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
export function mergeAudioGalleryItems(
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
