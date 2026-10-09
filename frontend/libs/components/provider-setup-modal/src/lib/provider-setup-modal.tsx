import { Modal } from "@storyteller/ui-modal";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRightIcon, WandSparklesIcon } from "lucide-react";
import { Button } from "@storyteller/ui-button";
import { Input } from "@storyteller/ui-input";
import { useState } from "react";
import { useShowProviderLoginModalEvent } from "@storyteller/tauri-events";
import { GenerationProvider } from "@storyteller/common";
import { ProviderCredentialKey, ProviderSetApiKey } from "@storyteller/tauri-api";
import { notifyProviderCredentialsChanged } from "@storyteller/ui-model-selector";

interface ProviderSetupModalProps {
}

export function ProviderSetupModal({
}: ProviderSetupModalProps) {
  const [showModal, setShowModal] = useState(false);
  const [provider, setProvider] = useState<GenerationProvider>(GenerationProvider.Artcraft);
  const [veniceApiKey, setVeniceApiKey] = useState("");
  const [isSavingKey, setIsSavingKey] = useState(false);

  useShowProviderLoginModalEvent(async (event) => {
    // Retained native handlers can still emit these events for old tasks.
    if (
      event.provider === GenerationProvider.Sora ||
      event.provider === GenerationProvider.WorldLabs
    ) {
      return;
    }
    console.log("Show provider login modal event received from Tauri:", event);
    setProvider(event.provider);
    setShowModal(true);
  });

  const serviceProviderName = getServiceProviderName(provider);
  const isVenice = provider === GenerationProvider.Venice;

  const modalTitle = `Set up ${serviceProviderName}`;
  const modalSubTitle = `Add your ${serviceProviderName} account to ArtCraft!`;

  let modalDescription;
  switch (provider) {
    case GenerationProvider.Grok:
      modalDescription = `You can add your ${serviceProviderName} account to ArtCraft by simply logging in. Use can then use it directly within Artcraft. You can add all of your AI accounts to Artcraft to use them all in one place and build the ultimate AI art tool.`;
      break;
    case GenerationProvider.Venice:
      modalDescription = `Add your ${serviceProviderName} API key to use your own Venice.ai account directly within Artcraft. You pay Venice.ai directly — no ArtCraft credits are used. Create a key at venice.ai/settings/api.`;
      break;
    default:
      modalDescription = `You can add your ${serviceProviderName} account to ArtCraft by simply logging in. Use your credits and account directly within Artcraft. You can add all of your AI accounts to Artcraft to use them all in one place and build the ultimate AI art tool.`;
      break;
  }

  const modalButtonText = isVenice ? "Save API key" : `Set up ${serviceProviderName}`;

  const buttonOnClick = async () => {
    if (isVenice) {
      const trimmedKey = veniceApiKey.trim();
      if (!trimmedKey) return;
      setIsSavingKey(true);
      try {
        await ProviderSetApiKey(ProviderCredentialKey.VeniceApiKey, trimmedKey);
        setVeniceApiKey("");
        setShowModal(false);
        notifyProviderCredentialsChanged();
      } catch (e) {
        console.error("Error saving the Venice.ai API key", e);
      } finally {
        setIsSavingKey(false);
      }
      return;
    }

    switch (provider) {
      case GenerationProvider.Grok:
        await invoke("grok_open_login_command");
        break;
      case GenerationProvider.Midjourney:
        await invoke("midjourney_open_login_command");
        break;
      case GenerationProvider.Fal:
        break; // TODO: None yet.
      default:
        break;
    }
    setShowModal(false);
  };

  return (
    <Modal
      //title={modalTitle}
      isOpen={showModal}
      onClose={() => {
        setShowModal(false);
      }}
      className="max-w-xl p-6"
      showClose={true}
    >
      <div className="flex flex-col gap-6 text-base-fg">
        <div className="flex flex-col gap-3">
          <p className="hud-label flex items-center gap-2 text-base-fg/55">
            <WandSparklesIcon aria-hidden className="h-3.5 w-3.5" />
            Connect account
          </p>
          <h1 className="font-display text-3xl font-medium leading-[1.05] tracking-[-0.03em]">
            {modalTitle}
          </h1>
          <p className="text-base font-medium text-base-fg/80">
            {modalSubTitle}
          </p>
          <p className="text-sm leading-relaxed text-base-fg/60">
            {modalDescription}
          </p>
        </div>

        {isVenice && (
          <Input
            value={veniceApiKey}
            onChange={(e) => setVeniceApiKey(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") buttonOnClick();
            }}
            placeholder="Paste your Venice.ai API key"
            type="password"
          />
        )}

        <Button
          className="h-10 px-4"
          icon={ArrowRightIcon}
          iconFlip={true}
          disabled={isVenice && (!veniceApiKey.trim() || isSavingKey)}
          onClick={() => {
            buttonOnClick();
          }}
        >
          {isSavingKey ? "Saving..." : modalButtonText}
        </Button>
      </div>
    </Modal>
  );
}

function getServiceProviderName(provider: GenerationProvider) : string {
  switch (provider) {
    case GenerationProvider.Grok:
      return "Grok";
    case GenerationProvider.Fal:
      return "Fal";
    case GenerationProvider.Midjourney:
      return "Midjourney";
    case GenerationProvider.Higgsfield:
      return "Higgsfield";
    case GenerationProvider.Krea:
      return "Krea";
    case GenerationProvider.Leonardo:
      return "Leonardo";
    case GenerationProvider.Magnific:
      return "Magnific";
    case GenerationProvider.Openart:
      return "OpenArt";
    case GenerationProvider.Picsart:
      return "Picsart";
    case GenerationProvider.Pixverse:
      return "PixVerse";
    case GenerationProvider.Runway:
      return "Runway";
    case GenerationProvider.Venice:
      return "Venice.ai";
    case GenerationProvider.Artcraft:
    default:
      return "Artcraft";
  }
}

export default ProviderSetupModal;
