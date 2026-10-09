import { useEffect, useState } from "react";
import { GenerationProvider } from "@storyteller/api-enums";
import { ProviderCredentialKey, ProviderList } from "@storyteller/tauri-api";
import { UsersApi } from "@storyteller/api";

const usersApi = new UsersApi();

const CREDENTIAL_TO_PROVIDER: Partial<
  Record<ProviderCredentialKey, GenerationProvider>
> = {
  [ProviderCredentialKey.FalApiKey]: GenerationProvider.Fal,
  [ProviderCredentialKey.VeniceApiKey]: GenerationProvider.Venice,
  [ProviderCredentialKey.GrokWebLogin]: GenerationProvider.Grok,
  [ProviderCredentialKey.MidjourneyLogin]: GenerationProvider.Midjourney,
  [ProviderCredentialKey.RunwayWebLogin]: GenerationProvider.Runway,
  [ProviderCredentialKey.HiggsfieldWebLogin]: GenerationProvider.Higgsfield,
};

/** Window event fired after provider credentials are saved/cleared. */
export const PROVIDER_CREDENTIALS_CHANGED_EVENT = "provider-credentials-changed";

/** Tell every live model picker to re-check which providers are configured. */
export function notifyProviderCredentialsChanged(): void {
  try {
    window.dispatchEvent(new Event(PROVIDER_CREDENTIALS_CHANGED_EVENT));
  } catch {
    // ignore (non-window environments)
  }
}

/**
 * The providers this install can actually use right now:
 * - ArtCraft, when signed in (managed generations + credits)
 * - a BYOK provider, when its credential is saved (Venice key, Fal key, …)
 *
 * Returns `null` while loading; an empty set means nothing is configured.
 */
export function useConfiguredProviders(): Set<GenerationProvider> | null {
  const [configured, setConfigured] =
    useState<Set<GenerationProvider> | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      const providers = new Set<GenerationProvider>();

      // ArtCraft-managed generations require a signed-in session.
      try {
        const session = await usersApi.GetSession();
        if (session.success && session.data?.user) {
          providers.add(GenerationProvider.Artcraft);
        }
      } catch {
        // Not signed in.
      }

      // BYOK credentials.
      try {
        const result = await ProviderList();
        for (const entry of result.payload?.providers ?? []) {
          if (!entry.has_credentials) continue;
          const provider = CREDENTIAL_TO_PROVIDER[entry.provider_credential];
          if (provider) providers.add(provider);
        }
      } catch {
        // Credentials not reachable (e.g. not the desktop app).
      }

      if (!cancelled) {
        setConfigured(providers);
      }
    };

    load();

    const onChanged = () => load();
    window.addEventListener(PROVIDER_CREDENTIALS_CHANGED_EVENT, onChanged);

    return () => {
      cancelled = true;
      window.removeEventListener(PROVIDER_CREDENTIALS_CHANGED_EVENT, onChanged);
    };
  }, []);

  return configured;
}
