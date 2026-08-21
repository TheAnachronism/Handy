import React from "react";
import { useTranslation } from "react-i18next";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { Dropdown } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";
import { useSettings } from "../../hooks/useSettings";
import { useModelStore } from "../../stores/modelStore";
import type { LiveInsertionLookahead, ModelInfo } from "@/bindings";

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
    const lookahead = (getSetting("live_insertion_lookahead") ||
      "fast") as LiveInsertionLookahead;

    const lookaheadOptions = [
      {
        value: "fastest",
        label: t("settings.general.liveInsertion.lookahead.options.fastest"),
      },
      {
        value: "fast",
        label: t("settings.general.liveInsertion.lookahead.options.fast"),
      },
      {
        value: "balanced",
        label: t("settings.general.liveInsertion.lookahead.options.balanced"),
      },
      {
        value: "accurate",
        label: t("settings.general.liveInsertion.lookahead.options.accurate"),
      },
    ];

    return (
      <>
        <ToggleSwitch
          checked={enabled}
          onChange={(enabled) => updateSetting("live_insertion", enabled)}
          isUpdating={isUpdating("live_insertion")}
          label={t("settings.general.liveInsertion.label")}
          description={t("settings.general.liveInsertion.description")}
          descriptionMode={descriptionMode}
          grouped={grouped}
        />
        {enabled && (
          <SettingContainer
            title={t("settings.general.liveInsertion.lookahead.label")}
            description={t(
              "settings.general.liveInsertion.lookahead.description",
            )}
            descriptionMode={descriptionMode}
            grouped={grouped}
          >
            <Dropdown
              options={lookaheadOptions}
              selectedValue={lookahead}
              onSelect={(value) =>
                updateSetting(
                  "live_insertion_lookahead",
                  value as LiveInsertionLookahead,
                )
              }
              disabled={isUpdating("live_insertion_lookahead")}
            />
          </SettingContainer>
        )}
      </>
    );
  },
);
