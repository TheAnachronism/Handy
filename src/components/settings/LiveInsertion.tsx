import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";
import { useModelStore } from "../../stores/modelStore";
import type { ModelInfo } from "@/bindings";

interface LiveInsertionProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const LiveInsertion: React.FC<LiveInsertionProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const { currentModel, models } = useModelStore();

    const currentModelInfo = models.find(
      (m: ModelInfo) => m.id === currentModel,
    );
    const supportsStreaming = currentModelInfo?.supports_streaming ?? false;

    if (!supportsStreaming) {
      return null;
    }

    const enabled = getSetting("live_insertion") ?? false;

    return (
      <ToggleSwitch
        checked={enabled}
        onChange={(enabled) => updateSetting("live_insertion", enabled)}
        isUpdating={isUpdating("live_insertion")}
        label={t("settings.general.liveInsertion.label")}
        description={t("settings.general.liveInsertion.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
    );
  },
);
