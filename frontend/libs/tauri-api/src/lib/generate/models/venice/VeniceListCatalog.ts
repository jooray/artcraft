import { invoke } from "@tauri-apps/api/core";
import { CommandResult } from "../common/CommandStatus";

export interface VeniceCatalogImageModel {
  id: string;
  name: string;
  aspect_ratios: string[];
  resolutions: string[];
  maybe_prompt_limit?: number;
  maybe_width_height_divisor?: number;
}

export interface VeniceCatalogVideoModel {
  id: string;
  name: string;
  model_type: string;
  aspect_ratios: string[];
  resolutions: string[];
  durations: string[];
  audio: boolean;
}

export interface VeniceCatalogPayload {
  success: boolean;
  image_models: VeniceCatalogImageModel[];
  video_models: VeniceCatalogVideoModel[];
}

export interface VeniceCatalogSuccess extends CommandResult {
  payload: VeniceCatalogPayload;
}

/**
 * Fetch Venice.ai's live model catalog (desktop command).
 *
 * Returns only the models this app build does not already know about; the
 * model pickers merge them in as Venice-provider entries. Empty when no
 * Venice API key is configured.
 */
export const VeniceListCatalog = async (): Promise<VeniceCatalogSuccess> => {
  const result = await invoke("venice_catalog_command");
  return result as VeniceCatalogSuccess;
};
