import { useMemo, useState } from "react";
import {
  DEFAULT_ACOUSTIC_SETTINGS,
  type AcousticEnvironment,
  type AcousticProfileDescription,
} from "../features/acoustic/model";
import type { PayloadSelection } from "../features/codec/model";
import { describeAcousticProfile } from "../lib/acoustic-codec";
import type { ProjectFile } from "./useEntropy";

interface AdaptiveProfileInput {
  ready: boolean;
  payload: PayloadSelection;
  source: string;
  filename: string;
  extension: string;
  projectFiles: ProjectFile[];
}

export function useAdaptiveProfile(input: AdaptiveProfileInput) {
  const [environment, setEnvironment] = useState<AcousticEnvironment>(
    DEFAULT_ACOUSTIC_SETTINGS.environment,
  );
  const [reliabilityPriority, setReliabilityPriority] = useState(
    DEFAULT_ACOUSTIC_SETTINGS.reliabilityPriority,
  );
  const settings = useMemo(
    () => ({ environment, reliabilityPriority }),
    [environment, reliabilityPriority],
  );
  const payloadBytes = useMemo(() => estimatePayloadBytes(input), [input]);
  const profile = useMemo<AcousticProfileDescription | null>(() => {
    if (!input.ready) return null;
    try {
      return describeAcousticProfile(settings, payloadBytes);
    } catch {
      return null;
    }
  }, [input.ready, payloadBytes, settings]);
  return {
    settings,
    profile,
    setEnvironment,
    setReliabilityPriority,
  };
}

function estimatePayloadBytes(input: AdaptiveProfileInput): number {
  const encoder = new TextEncoder();
  if (input.payload === "project") {
    return input.projectFiles.reduce(
      (total, file) =>
        total +
        encoder.encode(file.name).length +
        encoder.encode(file.extension).length +
        encoder.encode(file.source).length,
      0,
    );
  }
  const sourceBytes = encoder.encode(input.source).length;
  if (input.payload === "text") return sourceBytes;
  return (
    sourceBytes +
    encoder.encode(input.filename).length +
    encoder.encode(input.extension).length
  );
}
