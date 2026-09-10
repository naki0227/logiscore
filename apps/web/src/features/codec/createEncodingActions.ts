import type { ProjectFile, useEntropy } from "../../hooks/useEntropy";
import type { useAcousticCodec } from "../../hooks/useAcousticCodec";
import { areProjectFilesEqual, resolveCodecRoute } from "./model";
import type { ModeSelection, PayloadSelection } from "./model";
import type { AcousticSettings } from "../acoustic/model";

interface EncodingOptions {
  ready: boolean;
  playing: boolean;
  payload: PayloadSelection;
  mode: ModeSelection;
  source: string;
  filename: string;
  extension: string;
  projectFiles: ProjectFile[];
  acousticSettings: AcousticSettings;
  securePassword?: string;
  entropy: Pick<
    ReturnType<typeof useEntropy>,
    | "encodeText"
    | "encodeSourceFile"
    | "encodeProjectSource"
    | "decodeProjectSource"
  >;
  acoustic: Pick<
    ReturnType<typeof useAcousticCodec>,
    "encodeText" | "encodeSourceFile" | "encodeProject" | "decodeWav"
  >;
  setStatus: (status: string) => void;
  setAcousticVerified: (verified: boolean) => void;
  setProjectVerified: (verified: boolean) => void;
}

export function createEncodingActions(options: EncodingOptions) {
  const encodeProject = (musical: boolean, acoustic: boolean) => {
    if (options.projectFiles.length === 0) return;
    if (acoustic) {
      const wav = options.acoustic.encodeProject(
        options.projectFiles,
        options.acousticSettings,
        options.securePassword,
      );
      if (!wav) return;
      try {
        const decoded = options.acoustic.decodeWav(wav, options.securePassword);
        const matched =
          decoded.type === "project" &&
          areProjectFilesEqual(decoded.files, options.projectFiles);
        options.setProjectVerified(matched);
        options.setStatus(
          matched
            ? `✅ ${options.securePassword ? "SECURE" : "RELIABLE"} SYMPHONY VERIFIED: 100% BIT-PERFECT`
            : "⚠️ ACOUSTIC PROJECT VERIFICATION FAILED",
        );
      } catch {
        options.setProjectVerified(false);
        options.setStatus("⚠️ ACOUSTIC PROJECT VERIFICATION FAILED");
      }
      return;
    }
    const midi = options.entropy.encodeProjectSource(
      options.projectFiles,
      musical,
    );
    if (!midi) return;
    options.setStatus(`SYMPHONY CREATED: ${midi.length} BYTES`);
    setTimeout(() => {
      try {
        const decoded = options.entropy.decodeProjectSource(midi);
        const matched = decoded
          ? areProjectFilesEqual(decoded, options.projectFiles)
          : false;
        options.setProjectVerified(matched);
        options.setStatus(
          matched
            ? "✅ SYMPHONY VERIFIED: 100% BIT-PERFECT"
            : "⚠️ VERIFICATION FAILED: CONTENT MISMATCH",
        );
      } catch {
        options.setProjectVerified(false);
        options.setStatus("⚠️ PROJECT VERIFICATION FAILED");
      }
    }, 100);
  };

  const handleEncode = () => {
    if (!options.ready || options.playing) return;
    options.setStatus("ENCODING...");
    const route = resolveCodecRoute(options.payload, options.mode);
    const reliable = route.endsWith("-reliable");
    const secure = route.endsWith("-secure");
    const acoustic = reliable || secure;
    const musical = route.endsWith("-musical");
    if (route.startsWith("v2-project-")) {
      encodeProject(musical, acoustic);
      return;
    }
    if (acoustic) {
      const wav = route.startsWith("v2-text-")
        ? options.acoustic.encodeText(
            options.source,
            options.acousticSettings,
            options.securePassword,
          )
        : options.acoustic.encodeSourceFile(
            options.filename,
            options.extension,
            options.source,
            options.acousticSettings,
            options.securePassword,
          );
      if (!wav) return;
      try {
        const decoded = options.acoustic.decodeWav(wav, options.securePassword);
        const matched =
          decoded.type === "text"
            ? decoded.text === options.source
            : decoded.type === "source-file" &&
              decoded.source === options.source &&
              decoded.extension === options.extension &&
              decoded.filename === options.filename;
        options.setAcousticVerified(matched);
        options.setStatus(
          matched
            ? `✅ V2 ${secure ? "SECURE" : "RELIABLE"} ${options.payload === "text" ? "TEXT" : "SOURCE FILE"} VERIFIED: ${wav.length} BYTES`
            : "⚠️ ACOUSTIC VERIFICATION FAILED",
        );
      } catch {
        options.setAcousticVerified(false);
        options.setStatus("⚠️ ACOUSTIC VERIFICATION FAILED");
      }
      return;
    }
    const midi = route.startsWith("v2-text-")
      ? options.entropy.encodeText(options.source, musical)
      : options.entropy.encodeSourceFile(
          options.filename,
          options.extension,
          options.source,
          musical,
        );
    if (midi) {
      options.setStatus(
        `V2 ${musical ? "MUSICAL " : ""}${route.startsWith("v2-text-") ? "TEXT" : "SOURCE FILE"} ENCODED: ${midi.length} BYTES`,
      );
    }
  };

  return { handleEncode };
}
