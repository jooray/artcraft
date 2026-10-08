import { Button } from "@storyteller/ui-button";
import { Input } from "@storyteller/ui-input";
import { useEffect, useState } from "react";
import { LoaderCircleIcon } from "lucide-react";
import {
  ProviderClear,
  ProviderCredentialKey,
  ProviderList,
  ProviderSetApiKey,
} from "@storyteller/tauri-api";

/// API key management for the Venice.ai BYOK provider.
export const VeniceAccountBlock = () => {
  const [savedKeyStart, setSavedKeyStart] = useState<string | undefined>(undefined);
  const [draftKey, setDraftKey] = useState("");
  const [isCheckingKey, setIsCheckingKey] = useState(false);
  const [isSavingKey, setIsSavingKey] = useState(false);

  const fetchCredentials = async () => {
    setIsCheckingKey(true);
    try {
      const result = await ProviderList();
      const entry = result.payload?.providers?.find(
        (provider) => provider.provider_credential === ProviderCredentialKey.VeniceApiKey,
      );
      setSavedKeyStart(entry?.has_credentials ? entry.maybe_details?.maybe_key_start : undefined);
    } catch (e) {
      console.error("Error fetching the Venice.ai API key", e);
      setSavedKeyStart(undefined);
    } finally {
      setIsCheckingKey(false);
    }
  };

  useEffect(() => {
    fetchCredentials();
  }, []);

  const hasSavedKey = !!savedKeyStart;

  const saveKey = async () => {
    const trimmedKey = draftKey.trim();
    if (!trimmedKey) return;
    setIsSavingKey(true);
    try {
      await ProviderSetApiKey(ProviderCredentialKey.VeniceApiKey, trimmedKey);
      setDraftKey("");
      await fetchCredentials();
    } catch (e) {
      console.error("Error saving the Venice.ai API key", e);
    } finally {
      setIsSavingKey(false);
    }
  };

  const clearKey = async () => {
    try {
      await ProviderClear(ProviderCredentialKey.VeniceApiKey);
      await fetchCredentials();
    } catch (e) {
      console.error("Error clearing the Venice.ai API key", e);
    }
  };

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center justify-between gap-3">
        <div className="flex min-w-0 flex-col gap-0.5">
          <p className="font-mono text-[11px] font-semibold uppercase tracking-[0.12em]">
            Venice.ai API key
          </p>
          <p className="truncate text-sm font-medium text-base-fg/80">
            {hasSavedKey ? savedKeyStart : "Not connected"}
          </p>
        </div>
        {hasSavedKey && (
          <Button
            variant="destructive"
            className="h-9 shrink-0 px-3"
            onClick={clearKey}
            disabled={isCheckingKey}
          >
            {isCheckingKey ? <LoaderCircleIcon className="animate-spin text-sm" /> : "Disconnect"}
          </Button>
        )}
      </div>

      {!hasSavedKey && (
        <div className="flex items-center gap-2">
          <Input
            value={draftKey}
            onChange={(e) => setDraftKey(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") saveKey();
            }}
            placeholder="Enter your Venice.ai API key"
            type="password"
            className="flex-1"
          />
          <Button
            variant="secondary"
            className="h-9 shrink-0 px-3"
            onClick={saveKey}
            disabled={!draftKey.trim() || isSavingKey}
          >
            {isSavingKey ? <LoaderCircleIcon className="animate-spin text-sm" /> : "Save"}
          </Button>
        </div>
      )}

      <p className="text-xs leading-relaxed text-base-fg/50">
        Generate with your own Venice.ai account — no ArtCraft credits are used. Create a key at
        venice.ai/settings/api.
      </p>
    </div>
  );
};
