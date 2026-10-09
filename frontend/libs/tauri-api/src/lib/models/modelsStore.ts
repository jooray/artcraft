// Shared model store: the single source of truth for the model dropdowns.
//
// Seeded synchronously from the frontend OVERLAY (`IMAGE_MODELS` / `VIDEO_MODELS`
// in `@storyteller/model-list`) so the UI is never empty. On app boot,
// `loadModelsFromBackend()` fetches the authoritative omni listing via the Tauri
// commands and rebuilds the lists FROM that response (membership + order come
// from the backend; the overlay only enriches UI metadata; backend models with
// no overlay entry are built minimally so NEW models appear). If the fetch fails
// the store keeps the overlay.

import { create } from "zustand";
import {
  ImageModel,
  VideoModel,
  IMAGE_MODELS,
  VIDEO_MODELS,
  ALL_MODELS_LIST,
  buildImageModelsFromListing,
  buildVideoModelsFromListing,
  buildVeniceCatalogImageModels,
  buildVeniceCatalogVideoModels,
} from "@storyteller/model-list";
import { ListImageModels } from "../generate/models/image/ListImageModels.js";
import { ListVideoModels } from "../generate/models/video/ListVideoModels.js";
import { VeniceListCatalog } from "../generate/models/venice/VeniceListCatalog.js";

export interface ModelsStoreState {
  imageModels: ImageModel[];
  videoModels: VideoModel[];
  // True once a backend reconciliation has completed at least once.
  loaded: boolean;
  isLoading: boolean;
  loadModelsFromBackend: () => Promise<void>;
}

// Venice catalog models appended by the previous load, so a rebuild can drop
// stale ones (e.g. after the API key is cleared) before appending fresh ones.
let lastDynamicImageIds = new Set<string>();
let lastDynamicVideoIds = new Set<string>();

const appendDynamic = <T extends { tauriId: string }>(
  base: T[],
  dynamic: T[],
  lastIds: Set<string>,
): T[] => {
  const existing = base.filter((model) => !lastIds.has(model.tauriId));
  const known = new Set(existing.map((model) => model.tauriId));
  return [...existing, ...dynamic.filter((model) => !known.has(model.tauriId))];
};

export const useModelsStore = create<ModelsStoreState>((set, get) => ({
  imageModels: IMAGE_MODELS,
  videoModels: VIDEO_MODELS,
  loaded: false,
  isLoading: false,
  loadModelsFromBackend: async () => {
    if (get().isLoading) return;
    console.log("[models] loadModelsFromBackend() — calling Tauri commands…");
    set({ isLoading: true });

    const [imageResult, videoResult, veniceResult] = await Promise.allSettled([
      ListImageModels(),
      ListVideoModels(),
      VeniceListCatalog(),
    ]);

    const next: Partial<ModelsStoreState> = { isLoading: false, loaded: true };

    if (imageResult.status === "fulfilled") {
      const { models, providers } = imageResult.value.payload;
      const offered = providers.flatMap((p) => p.models.map((pm) => pm.model));
      const built = buildImageModelsFromListing(IMAGE_MODELS, models, offered);
      next.imageModels = built;
      console.log(
        `[models] image: backend returned ${models.length} → ${built.length} models shown`,
        built.map((m) => m.tauriId),
      );
    } else {
      console.error("[models] failed to load image models from backend:", imageResult.reason);
    }

    if (videoResult.status === "fulfilled") {
      const { models, providers } = videoResult.value.payload;
      const offered = providers.flatMap((p) => p.models.map((pm) => pm.model));
      const built = buildVideoModelsFromListing(VIDEO_MODELS, models, offered);
      next.videoModels = built;
      console.log(
        `[models] video: backend returned ${models.length} → ${built.length} models shown`,
        built.map((m) => m.tauriId),
      );
    } else {
      console.error("[models] failed to load video models from backend:", videoResult.reason);
    }

    // Merge the RUNTIME Venice catalog: models Venice added after this app
    // release. Generation passes their raw ids through the Venice handler.
    if (veniceResult.status === "fulfilled") {
      const payload = veniceResult.value.payload;
      const dynamicImages = buildVeniceCatalogImageModels(payload.image_models ?? []);
      const dynamicVideos = buildVeniceCatalogVideoModels(payload.video_models ?? []);

      next.imageModels = appendDynamic(
        next.imageModels ?? get().imageModels,
        dynamicImages,
        lastDynamicImageIds,
      );
      next.videoModels = appendDynamic(
        next.videoModels ?? get().videoModels,
        dynamicVideos,
        lastDynamicVideoIds,
      );
      lastDynamicImageIds = new Set(dynamicImages.map((model) => model.tauriId));
      lastDynamicVideoIds = new Set(dynamicVideos.map((model) => model.tauriId));

      if (dynamicImages.length > 0 || dynamicVideos.length > 0) {
        console.log(
          `[models] Venice runtime catalog: +${dynamicImages.length} image, +${dynamicVideos.length} video model(s)`,
        );
      }
    } else {
      console.warn("[models] Venice catalog unavailable:", veniceResult.reason);
    }

    set(next);
  },
}));

// Venice adds models between releases; re-check when provider credentials
// change (a Venice key saved or cleared). The event name matches
// `PROVIDER_CREDENTIALS_CHANGED_EVENT` in `@storyteller/ui-model-selector`
// (imported as a literal to avoid a dependency cycle; that lib depends on
// this one). Boot and periodic refreshes call `loadModelsFromBackend`
// directly.
if (typeof window !== "undefined") {
  window.addEventListener("provider-credentials-changed", () => {
    void useModelsStore.getState().loadModelsFromBackend();
  });
}

// Hook selectors for React consumers.
export const useImageModels = (): ImageModel[] => useModelsStore((s) => s.imageModels);
export const useVideoModels = (): VideoModel[] => useModelsStore((s) => s.videoModels);

// History/recreate must consult the hydrated catalog too, not just the build's overlay.
export function findDesktopModel(modelId: string) {
  const { imageModels, videoModels } = useModelsStore.getState();
  return [...imageModels, ...videoModels, ...ALL_MODELS_LIST]
    .find((model) => model.tauriId === modelId || model.id === modelId);
}
