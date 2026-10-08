import { invoke } from "@tauri-apps/api/core";
import { CommandResult } from "../common/CommandStatus";

/** Mirrors the Rust `ProviderCredentialKey` enum (snake_case). */
export enum ProviderCredentialKey {
  FalApiKey = "fal_api_key",
  ReplicateApiKey = "replicate_api_key",
  VeniceApiKey = "venice_api_key",
  GrokWebLogin = "grok_web_login",
  HiggsfieldWebLogin = "higgsfield_web_login",
  MidjourneyLogin = "midjourney_login",
  RunwayWebLogin = "runway_web_login",
}

export type ProviderCredentialType = "api_key" | "web_login";

export interface ProviderCredentialDetails {
  maybe_key_start?: string;
  maybe_full_key?: string;
  maybe_email_address?: string;
  maybe_username?: string;
}

export interface ProviderListEntry {
  provider_credential: ProviderCredentialKey;
  credential_type: ProviderCredentialType;
  has_credentials: boolean;
  maybe_details?: ProviderCredentialDetails;
}

export interface ProviderListPayload {
  providers: ProviderListEntry[];
}

export interface ProviderListSuccess extends CommandResult {
  payload: ProviderListPayload;
}

/** List the known provider credentials and whether they are configured. */
export const ProviderList = async (): Promise<ProviderListSuccess> => {
  const result = await invoke("provider_list_command");
  return result as ProviderListSuccess;
};

/** Save an API key for a provider (e.g. the Venice.ai key). */
export const ProviderSetApiKey = async (
  providerCredential: ProviderCredentialKey,
  apiKey: string,
): Promise<CommandResult> => {
  const result = await invoke("provider_set_api_key_command", {
    request: { provider_credential: providerCredential, api_key: apiKey },
  });
  return result as CommandResult;
};

/** Remove the stored credential for a provider. */
export const ProviderClear = async (
  providerCredential: ProviderCredentialKey,
): Promise<CommandResult> => {
  const result = await invoke("provider_clear_command", {
    request: { provider_credential: providerCredential },
  });
  return result as CommandResult;
};
