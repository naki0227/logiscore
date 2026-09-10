import { useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import JSZip from "jszip";
import Visualizer, { type VisualizerHandle } from "./components/Visualizer";
import { useAudioEngine } from "./hooks/useAudioEngine";
import { useAcousticCodec } from "./hooks/useAcousticCodec";
import { useEntropy, type ProjectFile } from "./hooks/useEntropy";
import { useWavPlayback } from "./hooks/useWavPlayback";
import { useAdaptiveProfile } from "./hooks/useAdaptiveProfile";
import { useSecureMode } from "./hooks/useSecureMode";
import {
  type ModeSelection,
  type PayloadSelection,
} from "./features/codec/model";
import { createEncodingActions } from "./features/codec/createEncodingActions";
import { usePayloadImport } from "./features/files/usePayloadImport";
import { AppHeader } from "./features/workspace/AppHeader";
import { ControlsPanel } from "./features/workspace/ControlsPanel";
import { OutputPanel } from "./features/workspace/OutputPanel";
import { SourcePanel } from "./features/workspace/SourcePanel";
import { downloadBlob } from "./lib/downloadBlob";
import { SAMPLE_CODE } from "./features/workspace/sampleCode";
import "./App.css";

function App() {
  const entropy = useEntropy();
  const acoustic = useAcousticCodec();
  const audio = useAudioEngine();
  const wavPlayback = useWavPlayback();
  const secure = useSecureMode();
  const visualizerRef = useRef<VisualizerHandle>(null);
  const [source, setSource] = useState(SAMPLE_CODE);
  const [extension, setExtension] = useState(".rs");
  const [status, setStatus] = useState("Initializing WASM...");
  const [uiMode, setUiMode] = useState<"encode" | "decode">("encode");
  const [payload, setPayload] = useState<PayloadSelection>("source-file");
  const [mode, setMode] = useState<ModeSelection>("auto");
  const [projectFiles, setProjectFiles] = useState<ProjectFile[]>([]);
  const [projectVerified, setProjectVerified] = useState(false);
  const [acousticVerified, setAcousticVerified] = useState(false);
  const [filename, setFilename] = useState("logiscore_output");
  const adaptive = useAdaptiveProfile({
    ready: entropy.ready,
    payload,
    source,
    filename,
    extension,
    projectFiles,
  });

  const isProject = payload === "project";
  const isText = payload === "text";
  const isAcoustic = mode === "reliable" || mode === "secure";
  const encodedData = isAcoustic ? acoustic.wavData : entropy.midiData;
  const encodedFormat = isAcoustic ? "WAV" : "MIDI";
  const playing = isAcoustic ? wavPlayback.playing : audio.playing;
  const progress = isAcoustic ? wavPlayback.progress : audio.progress;
  const processing = isAcoustic ? acoustic.processing : entropy.processing;
  const verified = isAcoustic
    ? acousticVerified
    : entropy.decodedSource !== null && entropy.decodedSource === source;
  const encodedSize = encodedData?.length ?? null;
  const initialize = entropy.initialize;

  useEffect(() => {
    void initialize().then(() => setStatus("READY"));
  }, [initialize]);

  const imports = usePayloadImport({
    ready: entropy.ready,
    payload,
    mode,
    decodeMidi: entropy.decodeImportedMidi,
    decodeAudio: async (file) => {
      const decoded = await acoustic.decodeFile(
        file,
        mode === "secure" ? secure.password : undefined,
      );
      setAcousticVerified(decoded.type !== "project");
      if (decoded.type === "project") setProjectVerified(true);
      return decoded;
    },
    stop: () => {
      audio.stop();
      wavPlayback.stop();
    },
    setPayload,
    setMode,
    setProjectFiles,
    setSource,
    setExtension,
    setFilename,
    setUiMode,
    setStatus,
  });

  const { handleEncode } = createEncodingActions({
    ready: entropy.ready,
    playing,
    payload,
    mode,
    source,
    filename,
    extension,
    projectFiles,
    acousticSettings: adaptive.settings,
    securePassword: mode === "secure" ? secure.password : undefined,
    entropy,
    acoustic,
    setStatus,
    setAcousticVerified,
    setProjectVerified,
  });

  const handlePlay = async () => {
    if (!encodedData || playing) return;
    setStatus("PLAYING...");
    if (isAcoustic) {
      try {
        await wavPlayback.play(encodedData, () =>
          setStatus("✅ WAV PLAYBACK COMPLETE"),
        );
      } catch {
        setStatus("❌ WAV PLAYBACK FAILED");
      }
      return;
    }
    await audio.play(
      encodedData,
      { noteDuration: 0.2, tickInterval: 0.25 },
      (note, velocity, duration, bass) =>
        visualizerRef.current?.addNote(note, velocity, duration, bass),
      () => verifyPlayback(encodedData),
    );
  };

  const verifyPlayback = (midi: Uint8Array) => {
    if (isProject) return setStatus("✅ PLAYBACK COMPLETE");
    setStatus("DECODING...");
    try {
      const decoded = isText
        ? entropy.decodeText(midi)
        : entropy.decodeSourceFile(midi);
      setStatus(decoded !== null ? "✅ VERIFIED" : "❌ DECODING FAILED");
    } catch {
      setStatus("❌ DECODING FAILED");
    }
  };

  const handleStop = () => {
    audio.stop();
    wavPlayback.stop();
    setStatus("STOPPED");
  };

  const handleDownload = () => {
    if (!encodedData) return;
    downloadBlob(
      new Blob([new Uint8Array(encodedData)], {
        type: isAcoustic ? "audio/wav" : "audio/midi",
      }),
      `${filename}.${isAcoustic ? "wav" : "mid"}`,
    );
    setStatus(`${encodedFormat} DOWNLOADED`);
  };

  const handleDownloadZip = async () => {
    if (!isProject || projectFiles.length === 0) return;
    setStatus("CREATING ZIP...");
    try {
      const zip = new JSZip();
      projectFiles.forEach((file) => zip.file(file.name, file.source));
      downloadBlob(
        await zip.generateAsync({ type: "blob" }),
        `${filename}_source.zip`,
      );
      setStatus("✅ ZIP DOWNLOADED");
    } catch {
      setStatus("❌ ZIP CREATION FAILED");
    }
  };

  const selectPayload = (selection: PayloadSelection) => {
    setPayload(selection);
    setUiMode("encode");
    setStatus(selection === "text" ? "V2 TEXT READY" : "READY");
  };

  const selectMode = (selection: ModeSelection) => {
    handleStop();
    setMode(selection);
    setAcousticVerified(false);
    setProjectVerified(false);
    setStatus(
      selection === "secure"
        ? "SECURE WAV READY"
        : selection === "reliable"
          ? "RELIABLE WAV READY"
          : "READY",
    );
  };

  return (
    <div
      className="app"
      onDragOver={imports.handleDragOver}
      onDrop={imports.handleDrop}
    >
      <AppHeader
        status={status}
        systemVersion={entropy.systemVersion}
        filename={filename}
        projectFileCount={isProject ? projectFiles.length : null}
      />
      <main className="main">
        <div className="visualizer-bg">
          <Visualizer ref={visualizerRef} />
        </div>
        <div className="layout-grid">
          <SourcePanel
            uiMode={uiMode}
            payload={payload}
            source={source}
            extension={extension}
            filename={filename}
            systemVersion={entropy.systemVersion}
            projectFiles={projectFiles}
            activeFile={audio.activeFile}
            inputSize={encodedSize ?? 0}
            inputFormat={encodedFormat}
            onPayloadChange={selectPayload}
            onSourceChange={setSource}
            onExtensionChange={setExtension}
            onUiModeChange={setUiMode}
            onImport={imports.handleImport}
            onDownloadZip={handleDownloadZip}
          />
          <ControlsPanel
            payload={payload}
            mode={mode}
            v2Version={entropy.v2Version}
            extensionInfo={entropy.extensionInfo}
            processing={processing}
            ready={entropy.ready}
            playing={playing}
            progress={progress}
            encodedSize={encodedSize}
            encodedFormat={encodedFormat}
            acousticEnvironment={adaptive.settings.environment}
            reliabilityPriority={adaptive.settings.reliabilityPriority}
            acousticProfile={adaptive.profile}
            securePassword={secure.password}
            secureConfirmation={secure.confirmation}
            secureValidation={secure.validation}
            onPayloadChange={selectPayload}
            onModeChange={selectMode}
            onEnvironmentChange={adaptive.setEnvironment}
            onPriorityChange={adaptive.setReliabilityPriority}
            onSecurePasswordChange={secure.setPassword}
            onSecureConfirmationChange={secure.setConfirmation}
            onEncode={handleEncode}
            onPlay={handlePlay}
            onStop={handleStop}
          />
          <OutputPanel
            uiMode={uiMode}
            filename={filename}
            encodedData={encodedData}
            encodedFormat={encodedFormat}
            decodedSource={entropy.decodedSource}
            source={source}
            verified={verified}
            projectVerified={projectVerified}
            isProject={isProject}
            onDownload={handleDownload}
            onSourceChange={setSource}
          />
        </div>
      </main>
      <AnimatePresence>
        {(entropy.error || acoustic.error) && (
          <motion.div
            initial={{ opacity: 0, y: 20, x: "-50%" }}
            animate={{ opacity: 1, y: 0, x: "-50%" }}
            exit={{ opacity: 0, y: 20, x: "-50%" }}
            className="error-toast"
          >
            ⚠️ {entropy.error || acoustic.error}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

export default App;
