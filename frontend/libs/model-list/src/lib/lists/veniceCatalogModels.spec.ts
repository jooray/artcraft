import { describe, expect, it } from "vitest";
import { GenerationProvider } from "@storyteller/api-enums";
import { CommonAspectRatio } from "../classes/properties/CommonAspectRatio.js";
import { CommonResolution } from "../classes/properties/CommonResolution.js";
import { ModelCreator } from "../classes/metadata/ModelCreator.js";
import { getModelFamilyName } from "../classes/metadata/ModelFamilies.js";
import {
  buildVeniceCatalogImageModels,
  buildVeniceCatalogVideoModels,
} from "./veniceCatalogModels.js";

describe("venice catalog builders", () => {
  it("builds image entries with mapped aspect ratios and resolutions", () => {
    const [model] = buildVeniceCatalogImageModels([
      {
        id: "nano-banana-3",
        name: "Nano Banana 3",
        aspect_ratios: ["1:1", "16:9"],
        resolutions: ["1K", "2K"],
        maybe_prompt_limit: 32768,
      },
    ]);

    expect(model.tauriId).toBe("nano-banana-3");
    expect(model.getProviders()).toEqual([GenerationProvider.Venice]);
    expect(model.canTextToImage).toBe(true);
    expect(model.aspectRatios).toEqual([
      CommonAspectRatio.Square,
      CommonAspectRatio.WideSixteenByNine,
    ]);
    expect(model.resolutions).toEqual([
      CommonResolution.OneK,
      CommonResolution.TwoK,
    ]);
    expect(model.maxPromptLength).toBe(32768);
  });

  it("skips pixel-sized image models", () => {
    const models = buildVeniceCatalogImageModels([
      {
        id: "venice-sd40",
        name: "Venice SD 4.0",
        aspect_ratios: [],
        resolutions: [],
        maybe_width_height_divisor: 16,
      },
    ]);
    expect(models).toEqual([]);
  });

  it("groups video variants into one family entry", () => {
    const models = buildVeniceCatalogVideoModels([
      {
        id: "seedance-3-0-text-to-video",
        name: "Seedance 3.0",
        model_type: "text-to-video",
        aspect_ratios: ["16:9"],
        resolutions: ["720p", "4k"],
        durations: ["8s", "4s", "12s"],
        audio: true,
      },
      {
        id: "seedance-3-0-image-to-video",
        name: "Seedance 3.0",
        model_type: "image-to-video",
        aspect_ratios: ["16:9"],
        resolutions: ["720p"],
        durations: ["4s"],
        audio: true,
      },
      {
        id: "seedance-3-0-reference-to-video",
        name: "Seedance 3.0",
        model_type: "reference-to-video",
        aspect_ratios: ["16:9"],
        resolutions: ["720p"],
        durations: ["4s"],
        audio: true,
      },
    ]);

    expect(models).toHaveLength(1);
    const [model] = models;
    expect(model.tauriId).toBe("seedance-3-0-text-to-video");
    expect(model.startFrame).toBe(true);
    expect(model.endFrame).toBe(false);
    expect(model.requiresImage).toBe(false);
    expect(model.supportsReferenceMode).toBe(true);
    expect(model.durationOptions).toEqual([4, 8, 12]);
    expect(model.defaultDuration).toBe(4);
    expect(model.resolutionOptions).toEqual(["720p", "4K"]);
    expect(model.generateWithSound).toBe(true);
    expect(model.sizeOptions?.map((option) => option.tauriValue)).toEqual([
      "wide_sixteen_by_nine",
    ]);
  });

  it("marks image-to-video-only families as requiring an image", () => {
    const [model] = buildVeniceCatalogVideoModels([
      {
        id: "kling-4-0-image-to-video",
        name: "Kling 4.0",
        model_type: "image-to-video",
        aspect_ratios: [],
        resolutions: [],
        durations: ["5s"],
        audio: false,
      },
    ]);

    expect(model.tauriId).toBe("kling-4-0-image-to-video");
    expect(model.startFrame).toBe(true);
    expect(model.requiresImage).toBe(true);
  });

  it("classifies reference variants by id even when typed image-to-video", () => {
    const [model] = buildVeniceCatalogVideoModels([
      {
        id: "seedance-3-0-text-to-video",
        name: "Seedance 3.0",
        model_type: "text-to-video",
        aspect_ratios: [],
        resolutions: [],
        durations: ["5s"],
        audio: false,
      },
      {
        id: "seedance-3-0-reference-to-video",
        name: "Seedance 3.0",
        model_type: "image-to-video",
        aspect_ratios: [],
        resolutions: [],
        durations: ["5s"],
        audio: false,
      },
    ]);

    expect(model.supportsReferenceMode).toBe(true);
    expect(model.startFrame).toBe(false);
  });

  it("merges -private variant ids into their family", () => {
    const models = buildVeniceCatalogVideoModels([
      {
        id: "seedance-2-5-us-text-to-video-private",
        name: "Seedance 2.5 US",
        model_type: "text-to-video",
        aspect_ratios: [],
        resolutions: [],
        durations: ["5s"],
        audio: false,
      },
      {
        id: "seedance-2-5-us-image-to-video-private",
        name: "Seedance 2.5 US",
        model_type: "image-to-video",
        aspect_ratios: [],
        resolutions: [],
        durations: ["5s"],
        audio: false,
      },
    ]);

    expect(models).toHaveLength(1);
    expect(models[0].tauriId).toBe("seedance-2-5-us-text-to-video-private");
    expect(models[0].startFrame).toBe(true);
  });

  it("disambiguates identical family names by variant", () => {
    const models = buildVeniceCatalogVideoModels([
      {
        id: "runway-gen5",
        name: "Runway Gen-5",
        model_type: "image-to-video",
        aspect_ratios: [],
        resolutions: [],
        durations: ["5s"],
        audio: false,
      },
      {
        id: "runway-gen5-text",
        name: "Runway Gen-5",
        model_type: "text-to-video",
        aspect_ratios: [],
        resolutions: [],
        durations: ["5s"],
        audio: false,
      },
    ]);

    expect(models.map((model) => model.selectorName)).toEqual([
      "Runway Gen-5 (Image)",
      "Runway Gen-5 (Text)",
    ]);
  });
});

describe("family grouping", () => {
  it("groups dashed Venice ids with their compiled families", () => {
    expect(getModelFamilyName("nano_banana_2", ModelCreator.Google)).toBe("Nano Banana");
    expect(getModelFamilyName("nano-banana-2-1", ModelCreator.Google)).toBe("Nano Banana");
    expect(getModelFamilyName("nano-banana-2-lite", ModelCreator.Venice)).toBe("Nano Banana");
    expect(getModelFamilyName("krea-v2-medium", ModelCreator.Venice)).toBe("Krea");
    expect(getModelFamilyName("luma-uni-1", ModelCreator.Venice)).toBe("Luma");
    expect(getModelFamilyName("ideogram-v4", ModelCreator.Venice)).toBe("Ideogram");
    expect(getModelFamilyName("z-image-turbo", ModelCreator.Venice)).toBe("Z-Image");
    expect(getModelFamilyName("hunyuan-image-v3", ModelCreator.Venice)).toBe("Hunyuan");
    expect(getModelFamilyName("hunyuan_3d_2", ModelCreator.Tencent)).toBe("Hunyuan 3D");
    expect(getModelFamilyName("gemini-omni-flash-1-1-text-to-video", ModelCreator.Google)).toBe("Gemini");
    expect(getModelFamilyName("gemini_25_flash", ModelCreator.Google)).toBe("Nano Banana");
    expect(getModelFamilyName("runway-gen4-5", ModelCreator.Runway)).toBe("Runway");
    expect(getModelFamilyName("happyhorse-1-1-text-to-video", ModelCreator.Alibaba)).toBe("Happy Horse");
  });

  it("infers creators for catalog models from the id", () => {
    const [imageModel] = buildVeniceCatalogImageModels([
      { id: "nano-banana-9", name: "Nano Banana 9", aspect_ratios: [], resolutions: [] },
    ]);
    expect(imageModel.creator).toBe(ModelCreator.Google);

    const [videoModel] = buildVeniceCatalogVideoModels([
      {
        id: "kling-4-0-text-to-video",
        name: "Kling 4.0",
        model_type: "text-to-video",
        aspect_ratios: [],
        resolutions: [],
        durations: ["5s"],
        audio: false,
      },
    ]);
    expect(videoModel.creator).toBe(ModelCreator.Kling);

    const [unknownModel] = buildVeniceCatalogImageModels([
      { id: "mystery-model", name: "Mystery", aspect_ratios: [], resolutions: [] },
    ]);
    expect(unknownModel.creator).toBe(ModelCreator.Venice);
  });
});
