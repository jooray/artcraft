// Runtime Venice.ai catalog → picker models.
//
// The desktop app ships a compiled set of Venice models; Venice adds models
// between app releases. `venice_catalog_command` returns the live catalog
// minus the compiled set, and these builders turn those entries into picker
// models. Generation sends the raw Venice id (`model_id`), and the desktop
// Venice handlers pass it through, so new models work without an app update.

import { GenerationProvider } from "@storyteller/api-enums";
import { ImageModel } from "../classes/ImageModel.js";
import { VideoModel } from "../classes/VideoModel.js";
import { ModelCreator } from "../classes/metadata/ModelCreator.js";
import { CommonAspectRatio } from "../classes/properties/CommonAspectRatio.js";
import { CommonResolution } from "../classes/properties/CommonResolution.js";
import { SizeIconOption, SizeOption } from "../classes/metadata/SizeOption.js";

/** Shape of `venice_catalog_command` payload entries (structural on purpose). */
export interface VeniceCatalogImageEntry {
  id: string;
  name: string;
  aspect_ratios: string[];
  resolutions: string[];
  maybe_prompt_limit?: number | null;
  maybe_width_height_divisor?: number | null;
}

export interface VeniceCatalogVideoEntry {
  id: string;
  name: string;
  model_type: string;
  aspect_ratios: string[];
  resolutions: string[];
  durations: string[];
  audio: boolean;
}

// ── Field maps ──

const ASPECT_RATIO_LABELS: Record<string, CommonAspectRatio> = {
  "auto": CommonAspectRatio.Auto,
  "1:1": CommonAspectRatio.Square,
  "3:2": CommonAspectRatio.WideThreeByTwo,
  "2:3": CommonAspectRatio.TallTwoByThree,
  "16:9": CommonAspectRatio.WideSixteenByNine,
  "9:16": CommonAspectRatio.TallNineBySixteen,
  "4:3": CommonAspectRatio.WideFourByThree,
  "3:4": CommonAspectRatio.TallThreeByFour,
  "5:4": CommonAspectRatio.WideFiveByFour,
  "4:5": CommonAspectRatio.TallFourByFive,
  "21:9": CommonAspectRatio.WideTwentyOneByNine,
  "9:21": CommonAspectRatio.TallNineByTwentyOne,
};

/** Venice resolution labels are case-sloppy (`4k` / `4K` / `768P`). */
const RESOLUTION_LABELS: Record<string, CommonResolution> = {
  "0.5K": CommonResolution.HalfK,
  "1K": CommonResolution.OneK,
  "2K": CommonResolution.TwoK,
  "3K": CommonResolution.ThreeK,
  "4K": CommonResolution.FourK,
  "480P": CommonResolution.FourEightyP,
  "720P": CommonResolution.SevenTwentyP,
  "1080P": CommonResolution.TenEightyP,
};

/** Video resolution labels the video promptbox can map back to an enum. */
const VIDEO_RESOLUTION_LABELS: Record<string, string> = {
  "480P": "480p",
  "720P": "720p",
  "1080P": "1080p",
  "1K": "1K",
  "2K": "2K",
  "3K": "3K",
  "4K": "4K",
};

const commonAspectRatiosFor = (labels: string[]): CommonAspectRatio[] =>
  labels
    .map((label) => ASPECT_RATIO_LABELS[label.trim().toLowerCase()])
    .filter((value): value is CommonAspectRatio => value !== undefined);

const commonResolutionsFor = (labels: string[]): CommonResolution[] =>
  labels
    .map((label) => RESOLUTION_LABELS[label.trim().toUpperCase()])
    .filter((value): value is CommonResolution => value !== undefined);

const videoResolutionLabelsFor = (labels: string[]): string[] =>
  labels
    .map((label) => VIDEO_RESOLUTION_LABELS[label.trim().toUpperCase()])
    .filter((value): value is string => value !== undefined);

// ── Image models ──

/**
 * Venice image models this build does not know about, as text-to-image picker
 * entries. Pixel-sized models (width/height sizing) are skipped: the runtime
 * passthrough sends aspect ratio + resolution, which those models reject.
 */
export const buildVeniceCatalogImageModels = (
  entries: VeniceCatalogImageEntry[],
): ImageModel[] =>
  entries
    .filter((entry) => (entry.maybe_width_height_divisor ?? 1) <= 1)
    .map((entry) => {
      const aspectRatios = commonAspectRatiosFor(entry.aspect_ratios ?? []);
      const resolutions = commonResolutionsFor(entry.resolutions ?? []);

      return new ImageModel({
        id: entry.id,
        tauriId: entry.id,
        fullName: entry.name,
        category: "image",
        creator: ModelCreator.Venice,
        providers: [GenerationProvider.Venice],
        selectorName: entry.name,
        selectorDescription: "Venice catalog model",
        selectorBadges: [],
        maxGenerationCount: 4,
        defaultGenerationCount: 1,
        canTextToImage: true,
        canEditImages: false,
        canUseImagePrompt: false,
        canChangeAspectRatio: aspectRatios.length > 0,
        aspectRatios,
        defaultAspectRatio: aspectRatios[0],
        canChangeResolution: resolutions.length > 0,
        resolutions,
        defaultResolution: resolutions[0],
        maxPromptLength: entry.maybe_prompt_limit ?? 4000,
        progressBarTime: 30000,
      });
    });

// ── Video models ──

const VIDEO_VARIANT_SUFFIXES = [
  "-first-last-frame-to-video",
  "-reference-to-video-basic",
  "-text-to-video-basic",
  "-image-to-video-basic",
  "-reference-to-video",
  "-text-to-video",
  "-image-to-video",
];

/** Per-key private variants ("…-private") group with their public family. */
const PRIVATE_SUFFIX = "-private";

interface VeniceVideoFamily {
  baseId: string;
  name: string;
  textToVideo?: VeniceCatalogVideoEntry;
  imageToVideo?: VeniceCatalogVideoEntry;
  referenceToVideo?: VeniceCatalogVideoEntry;
  firstLastFrame?: VeniceCatalogVideoEntry;
  standalone?: VeniceCatalogVideoEntry;
}

const variantKind = (entry: VeniceCatalogVideoEntry): keyof VeniceVideoFamily | null => {
  // Venice types reference-to-video models as "image-to-video", so the id
  // suffix is authoritative; model_type is only a fallback.
  if (entry.id.includes("-first-last-frame-to-video")) return "firstLastFrame";
  if (entry.id.includes("-reference-to-video")) return "referenceToVideo";
  if (entry.id.includes("-image-to-video")) return "imageToVideo";
  if (entry.id.includes("-text-to-video")) return "textToVideo";

  const modelType = entry.model_type.replaceAll("_", "-");
  if (modelType.includes("first-last")) return "firstLastFrame";
  if (modelType.includes("reference")) return "referenceToVideo";
  if (modelType.includes("image-to-video")) return "imageToVideo";
  if (modelType.includes("text-to-video")) return "textToVideo";
  return null;
};

const baseIdFor = (id: string): { baseId: string; suffix: string } | null => {
  const normalized = id.endsWith(PRIVATE_SUFFIX)
    ? id.slice(0, -PRIVATE_SUFFIX.length)
    : id;
  for (const suffix of VIDEO_VARIANT_SUFFIXES) {
    if (normalized.endsWith(suffix)) {
      return { baseId: normalized.slice(0, -suffix.length), suffix };
    }
  }
  return null;
};

const groupVideoFamilies = (
  entries: VeniceCatalogVideoEntry[],
): VeniceVideoFamily[] => {
  const families = new Map<string, VeniceVideoFamily>();

  for (const entry of entries) {
    const base = baseIdFor(entry.id);
    const key = base?.baseId ?? entry.id;
    const family = families.get(key) ?? { baseId: key, name: entry.name };
    family.name = family.name || entry.name;

    const kind = variantKind(entry);
    if (kind === null) {
      // Unrecognized variant naming: keep it as its own text-to-video entry.
      families.set(key, { ...family, name: entry.name, standalone: entry });
      continue;
    }
    if (kind === "textToVideo") family.textToVideo = entry;
    else if (kind === "imageToVideo") family.imageToVideo = entry;
    else if (kind === "referenceToVideo") family.referenceToVideo = entry;
    else family.firstLastFrame = entry;

    families.set(key, family);
  }

  return [...families.values()];
};

const durationSeconds = (labels: string[]): number[] =>
  labels
    .map((label) => Number.parseInt(label, 10))
    .filter((value) => Number.isFinite(value) && value > 0)
    .sort((a, b) => a - b);

/**
 * Variant hint for a family whose display name collides with another family's
 * (e.g. Runway lists separate text and image variants under one name).
 */
const familyNameHint = (family: VeniceVideoFamily): string | null => {
  const hasText = family.textToVideo !== undefined;
  const hasImage =
    family.imageToVideo !== undefined || family.firstLastFrame !== undefined;
  if (!hasText && hasImage) return "Image";
  if (hasText && !hasImage && family.referenceToVideo === undefined) return "Text";
  return null;
};

const sizeOptionForAspectRatio = (value: string): SizeOption => {
  switch (value) {
    case "square":
      return { tauriValue: value, textLabel: "1:1", icon: SizeIconOption.Square };
    case "wide_sixteen_by_nine":
      return { tauriValue: value, textLabel: "16:9", icon: SizeIconOption.Landscape16x9 };
    case "tall_nine_by_sixteen":
      return { tauriValue: value, textLabel: "9:16", icon: SizeIconOption.Portrait9x16 };
    case "wide_four_by_three":
      return { tauriValue: value, textLabel: "4:3", icon: SizeIconOption.Standard4x3 };
    case "tall_three_by_four":
      return { tauriValue: value, textLabel: "3:4", icon: SizeIconOption.Portrait3x4 };
    case "wide_three_by_two":
      return { tauriValue: value, textLabel: "3:2", icon: SizeIconOption.Landscape };
    case "tall_two_by_three":
      return { tauriValue: value, textLabel: "2:3", icon: SizeIconOption.Portrait };
    case "wide_five_by_four":
      return { tauriValue: value, textLabel: "5:4", icon: SizeIconOption.Landscape };
    case "tall_four_by_five":
      return { tauriValue: value, textLabel: "4:5", icon: SizeIconOption.Portrait };
    case "wide_twenty_one_by_nine":
      return { tauriValue: value, textLabel: "21:9", icon: SizeIconOption.Landscape16x9 };
    case "tall_nine_by_twenty_one":
      return { tauriValue: value, textLabel: "9:21", icon: SizeIconOption.Portrait9x16 };
    case "auto":
      return { tauriValue: value, textLabel: "Auto", icon: SizeIconOption.Square };
    default:
      return { tauriValue: value, textLabel: value, icon: SizeIconOption.Square };
  }
};

/**
 * Venice video models this build does not know about, grouped into one picker
 * entry per model family (text/image/reference/first-last variants share a
 * name). The entry id is the representative variant's full Venice id
 * (text-to-video when present); the desktop handler switches variants by id
 * suffix when frames or references are attached.
 */
export const buildVeniceCatalogVideoModels = (
  entries: VeniceCatalogVideoEntry[],
): VideoModel[] => {
  const families = groupVideoFamilies(entries);

  // Disambiguate identical family names ("Runway Gen-4.5" text + image).
  const nameCounts = new Map<string, number>();
  for (const family of families) {
    nameCounts.set(family.name, (nameCounts.get(family.name) ?? 0) + 1);
  }

  return families.map((family) => {
    const representative =
      family.textToVideo ??
      family.imageToVideo ??
      family.referenceToVideo ??
      family.firstLastFrame ??
      family.standalone;

    const aspectRatios = commonAspectRatiosFor(representative?.aspect_ratios ?? []);
    const resolutions = videoResolutionLabelsFor(representative?.resolutions ?? []);
    const durations = durationSeconds(representative?.durations ?? []);
    const hasTextToVideo = family.textToVideo !== undefined;
    const hint = familyNameHint(family);
    const fullName =
      (nameCounts.get(family.name) ?? 0) > 1 && hint !== null
        ? `${family.name} (${hint})`
        : family.name;

    return new VideoModel({
      id: representative?.id ?? family.baseId,
      tauriId: representative?.id ?? family.baseId,
      fullName,
      category: "video",
      creator: ModelCreator.Venice,
      providers: [GenerationProvider.Venice],
      selectorName: fullName,
      selectorDescription: "Venice catalog model",
      selectorBadges: [],
      startFrame: family.imageToVideo !== undefined || family.firstLastFrame !== undefined,
      endFrame: family.firstLastFrame !== undefined,
      requiresImage: !hasTextToVideo && family.imageToVideo !== undefined,
      generateWithSound: representative?.audio ?? false,
      durationOptions: durations.length > 0 ? durations : undefined,
      defaultDuration:
        durations.length > 0
          ? (durations.includes(5) ? 5 : durations[0])
          : undefined,
      minDuration: durations[0],
      maxDuration: durations[durations.length - 1],
      resolutionOptions: resolutions.length > 0 ? resolutions : undefined,
      defaultResolution: resolutions[0],
      sizeOptions: aspectRatios.map(sizeOptionForAspectRatio),
      supportsCommonAspectRatio: aspectRatios.length > 0,
      supportsReferenceMode: family.referenceToVideo !== undefined,
      maxReferenceImages: family.referenceToVideo ? 3 : undefined,
      progressBarTime: 3 * 60 * 1000,
      maxPromptLength: 3000,
    });
  });
};
