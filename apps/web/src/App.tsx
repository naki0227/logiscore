import { useCallback, useEffect, useRef, useState } from "react";
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
import { QuickStartGuide } from "./features/workspace/QuickStartGuide";
import { downloadBlob } from "./lib/downloadBlob";
import { SAMPLE_CODE, SAMPLE_TEXT } from "./features/workspace/sampleCode";
import type { SetStatus, StatusMessage } from "./features/status/model";
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
  const [status, setStatusMessage] = useState<StatusMessage>({
    text: "エンジンを読み込み中…",
    tone: "neutral",
  });
  const setStatus = useCallback<SetStatus>(
    (text, tone = "neutral") => setStatusMessage({ text, tone }),
    [],
  );
  const [uiMode, setUiMode] = useState<"encode" | "decode">("encode");
  const [payload, setPayload] = useState<PayloadSelection>("source-file");
  const [mode, setMode] = useState<ModeSelection>("auto");
  const [projectFiles, setProjectFiles] = useState<ProjectFile[]>([]);
  const [projectVerified, setProjectVerified] = useState(false);
  const [acousticVerified, setAcousticVerified] = useState(false);
  const [filename, setFilename] = useState("logiscore_output");
  const [delivered, setDelivered] = useState(false);
  const [restored, setRestored] = useState(false);
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
    void initialize().then(() => setStatus("準備完了"));
  }, [initialize, setStatus]);

  const imports = usePayloadImport({
    ready: entropy.ready,
    payload,
    mode,
    decodeMidi: (bytes) => {
      const decoded = entropy.decodeImportedMidi(bytes);
      setRestored(true);
      return decoded;
    },
    decodeAudio: async (file) => {
      const decoded = await acoustic.decodeFile(
        file,
        mode === "secure" ? secure.password : undefined,
      );
      setAcousticVerified(decoded.type !== "project");
      if (decoded.type === "project") setProjectVerified(true);
      setRestored(true);
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
    setStatus("再生中…");
    setDelivered(true);
    if (isAcoustic) {
      try {
        await wavPlayback.play(encodedData, () =>
          setStatus("再生が終わりました", "success"),
        );
      } catch {
        setStatus("再生できませんでした", "error");
      }
      return;
    }
    await audio.play(
      encodedData,
      { noteDuration: 0.2, tickInterval: 0.25 },
      (note, velocity, duration, bass) =>
        visualizerRef.current?.addNote(note, velocity, duration, bass),
      undefined,
      () => verifyPlayback(encodedData),
    );
  };

  const verifyPlayback = (midi: Uint8Array) => {
    if (isProject) return setStatus("再生が終わりました", "success");
    setStatus("復元を確認中…");
    try {
      const decoded = isText
        ? entropy.decodeText(midi)
        : entropy.decodeSourceFile(midi);
      setStatus(
        decoded !== null ? "復元を確認しました" : "復元できませんでした",
        decoded !== null ? "success" : "error",
      );
    } catch {
      setStatus("復元できませんでした", "error");
    }
  };

  const handleStop = () => {
    audio.stop();
    wavPlayback.stop();
    setStatus("停止しました");
  };

  const handleDownload = () => {
    if (!encodedData) return;
    downloadBlob(
      new Blob([new Uint8Array(encodedData)], {
        type: isAcoustic ? "audio/wav" : "audio/midi",
      }),
      `${filename}.${isAcoustic ? "wav" : "mid"}`,
    );
    setDelivered(true);
    setStatus(`${encodedFormat}をダウンロードしました`);
  };

  const handleDownloadZip = async () => {
    if (!isProject || projectFiles.length === 0) return;
    setStatus("ZIPを作成中…");
    try {
      const { default: JSZip } = await import("jszip");
      const zip = new JSZip();
      projectFiles.forEach((file) => zip.file(file.name, file.source));
      downloadBlob(
        await zip.generateAsync({ type: "blob" }),
        `${filename}_source.zip`,
      );
      setStatus("ZIPをダウンロードしました", "success");
    } catch {
      setStatus("ZIPを作成できませんでした", "error");
    }
  };

  /** 入力や設定が変わったら、前の変換結果は古くなるので消す */
  const clearOutput = () => {
    handleStop();
    entropy.clearEncoded();
    acoustic.clear();
    setAcousticVerified(false);
    setProjectVerified(false);
  };

  const selectPayload = (selection: PayloadSelection) => {
    clearOutput();
    setPayload(selection);
    setUiMode("encode");
    setStatus("準備完了");
  };

  const selectMode = (selection: ModeSelection) => {
    clearOutput();
    setMode(selection);
    setStatus(
      selection === "secure"
        ? "準備完了（暗号化 WAV）"
        : selection === "reliable"
          ? "準備完了（空気伝送 WAV）"
          : "準備完了",
    );
  };

  const editSource = (next: string) => {
    if (encodedData) clearOutput();
    setSource(next);
  };

  const selectExtension = (next: string) => {
    if (encodedData) clearOutput();
    setExtension(next);
  };

  const loadDemo = () => {
    clearOutput();
    setPayload("text");
    setMode("reliable");
    setSource(SAMPLE_TEXT);
    setFilename("logiscore_demo");
    setUiMode("encode");
    setAcousticVerified(false);
    setProjectVerified(false);
    setRestored(false);
    setDelivered(false);
    setStatus("デモを読み込みました。「変換する」を押してください");
  };

  return (
    <div
      className="app"
      onDragOver={imports.handleDragOver}
      onDrop={imports.handleDrop}
    >
      <AppHeader
        status={status}
        filename={filename}
        projectFileCount={isProject ? projectFiles.length : null}
      />
      <main className="main">
        <QuickStartGuide
          progress={{
            hasInput: isProject ? projectFiles.length > 0 : source.length > 0,
            encoded: encodedData !== null,
            played: delivered,
            restored,
          }}
          onLoadDemo={loadDemo}
        />
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
            onSourceChange={editSource}
            onExtensionChange={selectExtension}
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
            visualizer={<Visualizer ref={visualizerRef} />}
            onDownload={handleDownload}
            onSourceChange={setSource}
          />
        </div>
      </main>
      {(entropy.error || acoustic.error) && (
        <div className="error-toast" role="alert">
          {entropy.error || acoustic.error}
        </div>
      )}
    </div>
  );
}

export default App;
