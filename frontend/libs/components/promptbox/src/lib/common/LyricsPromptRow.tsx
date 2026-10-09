import type { ChangeEvent, ReactNode } from "react";
import { AlignLeftIcon, XIcon } from "lucide-react";

export interface LyricsPromptRowProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  maxLength?: number | null;
  rightSlot?: ReactNode;
  className?: string;
}

// Multiline lyrics input for lyric-capable music models (Venice MiniMax /
// ACE-Step). Renders beneath the main prompt, like StylePromptRow.
export function LyricsPromptRow({
  value,
  onChange,
  placeholder = "Lyrics — leave empty to let the model write them",
  maxLength,
  rightSlot,
  className = "",
}: LyricsPromptRowProps) {
  const handleChange = (event: ChangeEvent<HTMLTextAreaElement>) => {
    onChange(event.target.value);
  };

  return (
    <div className={`flex items-start gap-2 border-t border-white/10 px-1 pt-2 ${className}`}>
      <AlignLeftIcon className="mt-1 h-3.5 w-3.5 shrink-0 text-base-fg/40" />
      <textarea
        rows={3}
        value={value}
        onChange={handleChange}
        placeholder={placeholder}
        maxLength={maxLength ?? undefined}
        className="promptbox-scrollbar min-h-[3.5em] w-full resize-y overflow-y-auto bg-transparent text-sm text-base-fg placeholder:text-base-fg/35 focus:outline-none"
      />
      <div className="flex shrink-0 flex-col items-end gap-1 pt-0.5">
        {value.length > 0 && (
          <button
            type="button"
            aria-label="Clear lyrics"
            onClick={() => onChange("")}
            className="flex h-5 w-5 items-center justify-center rounded-[3px] text-base-fg/40 transition-colors hover:bg-white/10 hover:text-base-fg"
          >
            <XIcon className="h-3 w-3" />
          </button>
        )}
        {maxLength != null && value.length > 0 && (
          <span className="text-[10px] tabular-nums text-base-fg/35">
            {value.length}/{maxLength}
          </span>
        )}
        {rightSlot}
      </div>
    </div>
  );
}
