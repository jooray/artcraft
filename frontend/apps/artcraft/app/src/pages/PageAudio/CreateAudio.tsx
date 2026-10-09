import { GenerateAudio, EstimateAudioCost } from "@storyteller/tauri-api";
import type { OmniGenAudioCostResponse, OmniGenAudioRequest } from "@storyteller/api";
import { GenerationProvider } from "@storyteller/api-enums";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { PromptBoxAudio, usePromptAudioStore } from "@storyteller/ui-promptbox";
import {
  UploadAudioMedia,
  UploadImageMedia,
  FilterMediaClasses,
} from "@storyteller/api";
import { useOmniGenAudioModels, useAudioCostEstimate } from "@storyteller/omni-gen";
import {
  galleryModalLightboxImage,
  galleryModalLightboxMediaId,
  galleryModalLightboxVisible,
  galleryModalLightboxNavPrev,
  galleryModalLightboxNavNext,
} from "@storyteller/ui-gallery-modal";
import { HelpMenuButton } from "@storyteller/ui-help-menu";
import {
  useGalleryData,
  useGenerationJobs,
  useLastViewedGenerationStore,
  type GalleryItem,
} from "@storyteller/ui-generation-list";
import { useDesktopUsername } from "~/components/generation-feed/useDesktopUsername";
import { DesktopCreatePageShell } from "~/components/generation-feed/DesktopCreatePageShell";
import { DesktopGenerationGallery } from "~/components/generation-feed/DesktopGenerationGallery";
import { isVeniceAudioModel, withVeniceAudioModels } from "./veniceAudioModels";
import { mergeUniqueGalleryItems, useLocalCompletedTasks } from "~/components/generation-feed/useLocalCompletedTasks";
import { useConfiguredProviders } from "@storyteller/ui-model-selector";

const AUDIO_FILTER = [FilterMediaClasses.AUDIO];

const CreateAudio = () => {
  const { models: omniAudioModels } = useOmniGenAudioModels();
  // Venice.ai BYOK audio models are desktop-only; the backend listing doesn't
  // know about them.
  const allModels = useMemo(() => withVeniceAudioModels(omniAudioModels), [omniAudioModels]);

  // Only offer models whose provider is configured (Venice key, or an ArtCraft
  // session). Nothing configured — show the full catalog for setup discovery.
  const configuredProviders = useConfiguredProviders();
  const models = useMemo(() => {
    if (!configuredProviders || configuredProviders.size === 0) return allModels;
    return allModels.filter((model) =>
      isVeniceAudioModel(model.model)
        ? configuredProviders.has(GenerationProvider.Venice)
        : configuredProviders.has(GenerationProvider.Artcraft),
    );
  }, [allModels, configuredProviders]);
  const promptContentRef = useRef<HTMLDivElement>(null);
  const [promptHeight, setPromptHeight] = useState<number>(138);

  // Tag Venice model requests with the provider so the desktop backend queues
  // them directly against api.venice.ai instead of the ArtCraft backend.
  const generateAudioWithProvider = useCallback(
    async (request: OmniGenAudioRequest) => {
      const isVenice = isVeniceAudioModel(request.model);
      const response = await GenerateAudio(
        isVenice ? { ...request, provider: GenerationProvider.Venice } : request,
      );
      return response.payload;
    },
    [],
  );
  const estimateAudioWithProvider = useCallback(
    async (request: OmniGenAudioRequest) => {
      const isVenice = isVeniceAudioModel(request.model);
      const response = await EstimateAudioCost(
        isVenice ? { ...request, provider: GenerationProvider.Venice } : request,
      );
      return response.payload;
    },
    [],
  );

  // Cost estimate inputs come from the shared audio prompt store (the
  // promptbox owns the settings; the page only observes them).
  const selectedModelId = usePromptAudioStore((s) => s.selectedModelId);
  const referenceAudios = usePromptAudioStore((s) => s.referenceAudios);
  const referenceImages = usePromptAudioStore((s) => s.referenceImages);
  const selectedModel = useMemo(
    () => models.find((m) => m.model === selectedModelId) ?? models[0],
    [models, selectedModelId],
  );
  // BYOK providers (Venice) report a real USD estimate instead of credits.
  const [audioCostUsdCents, setAudioCostUsdCents] = useState<number | null>(null);
  const handleAudioEstimate = useCallback(
    (response: OmniGenAudioCostResponse | null) => {
      setAudioCostUsdCents(response?.cost_in_usd_cents ?? null);
    },
    [],
  );

  const audioCredits = useAudioCostEstimate({
    model: selectedModel?.model ?? "",
    audioReferenceCount: referenceAudios.length,
    hasImageReference: referenceImages.length > 0,
    sampleRateHz: selectedModel?.sample_rate_hz_options?.length
      ? (selectedModel.sample_rate_hz_default ?? undefined)
      : undefined,
  }, estimateAudioWithProvider, handleAudioEstimate);

  // Track the promptbox height so the feed can pad past it.
  useEffect(() => {
    const el = promptContentRef.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const update = () => setPromptHeight(el.offsetHeight);
    update();
    const ro = new ResizeObserver(() => update());
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // Tauri enqueues and persists audio jobs; the shared session poller and
  // library provide the same generation feed used by the webapp.
  const username = useDesktopUsername();
  const feed = useGenerationJobs({ mediaType: "audio", enabled: !!username });
  const gallery = useGalleryData({
    username,
    filterMediaClasses: AUDIO_FILTER,
    excludeUploads: true,
  });

  // Venice BYOK results (uploaded or saved locally) live in the local task
  // database, not in the backend jobs feed.
  const localCompletedItems = useLocalCompletedTasks({ mediaClasses: ["audio"] });
  const mergedGalleryItems = useMemo(
    () => mergeUniqueGalleryItems(gallery.items, localCompletedItems),
    [gallery.items, localCompletedItems],
  );

  // Content only — while the gallery is still loading the shell keeps the
  // hero + background up as a splash and fades them out when items land.
  const hasContent =
    feed.inProgress.length > 0 ||
    feed.failed.length > 0 ||
    feed.newlyCompleted.length > 0 ||
    mergedGalleryItems.length > 0;

  const newlyCompletedTokens = useMemo(
    () => new Set(feed.newlyCompleted.map((item) => item.id)),
    [feed.newlyCompleted],
  );

  // Flat, time-sorted completed list driving lightbox prev/next navigation.
  const flatCompleted = useMemo(() => {
    const seen = new Set<string>();
    const merged: GalleryItem[] = [];
    for (const item of feed.newlyCompleted) {
      if (!seen.has(item.id)) {
        seen.add(item.id);
        merged.push(item);
      }
    }
    for (const item of mergedGalleryItems) {
      if (!seen.has(item.id)) {
        seen.add(item.id);
        merged.push(item);
      }
    }
    merged.sort(
      (a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    );
    return merged;
  }, [feed.newlyCompleted, mergedGalleryItems]);

  const flatCompletedRef = useRef(flatCompleted);
  flatCompletedRef.current = flatCompleted;

  // Open a completed row in the global lightbox (rendered by TopBar's
  // gallery modal); prev/next walk the merged feed order.
  const lastViewedId = useLastViewedGenerationStore((s) => s.id);

  const openInLightbox = useCallback((item: GalleryItem) => {
    useLastViewedGenerationStore.getState().setId(item.id);
    const list = flatCompletedRef.current;
    const index = list.findIndex((i) => i.id === item.id);
    galleryModalLightboxNavPrev.value =
      index > 0 ? () => openInLightbox(list[index - 1]) : null;
    galleryModalLightboxNavNext.value =
      index >= 0 && index < list.length - 1
        ? () => openInLightbox(list[index + 1])
        : null;
    galleryModalLightboxMediaId.value = item.id;
    galleryModalLightboxImage.value = {
      id: item.id,
      label: item.label,
      thumbnail: item.thumbnail,
      fullImage: item.fullImage,
      createdAt: item.createdAt,
      mediaClass: item.mediaClass,
    };
    galleryModalLightboxVisible.value = true;
  }, []);

  return (
    <DesktopCreatePageShell
      hasContent={hasContent}
      emptyStateTitle="Create Audio"
      emptyStateSubtitle="Describe a song, a sound, or a sample"
      bottomOffset={promptHeight + 40}
      listContent={
        <DesktopGenerationGallery
          inProgressJobs={feed.inProgress}
          failedJobs={feed.failed}
          onDismissFailed={feed.dismissFailed}
          newlyCompletedItems={feed.newlyCompleted}
          galleryItems={mergedGalleryItems}
          newlyCompletedTokens={newlyCompletedTokens}
          hasMore={gallery.hasMore}
          isLoading={gallery.isLoading}
          isInitialLoading={gallery.isInitialLoading}
          onLoadMore={gallery.loadMore}
          onGalleryItemClick={openInLightbox}
          lastViewedId={lastViewedId}
        />
      }
      promptBox={
        <div className="fixed bottom-4 left-1/2 z-20 w-full max-w-5xl -translate-x-1/2 px-2 sm:px-4">
          <div ref={promptContentRef}>
            <PromptBoxAudio
              generateAudio={generateAudioWithProvider}
              models={models}
              uploadAudio={UploadAudioMedia}
              uploadImage={UploadImageMedia}
              credits={audioCredits}
              costUsdCents={audioCostUsdCents}
              onEnqueuePressed={async () => {
                // Nudge the shared jobs poller so the pending cards appear
                // immediately (one per created job token).
                window.dispatchEvent(new Event("task-queue-update"));
                window.dispatchEvent(new Event("credits-change"));
              }}
            />
          </div>
        </div>
      }
      bottomRight={<HelpMenuButton />}
    />
  );
};

export default CreateAudio;
