import React, { useEffect, useState, useRef, memo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useEntropy } from './hooks/useEntropy';
import { useAudioEngine } from './hooks/useAudioEngine';
import Visualizer from './components/Visualizer';
import type { VisualizerHandle } from './components/Visualizer';
import Panel from './components/Panel';
import './App.css';

/** Tailscale-inspired SVG Icons **/
const Icons = {
    Node: () => (
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="3" />
            <path d="M12 21V15M12 9V3M21 12H15M9 12H3" />
        </svg>
    ),
    Import: () => (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 3v12m0 0l-4-4m4 4l4-4M4 21h16" />
        </svg>
    ),
    Play: () => (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <path d="M8 5v14l11-7z" />
        </svg>
    ),
    Stop: () => (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="6" width="12" height="12" />
        </svg>
    ),
    Download: () => (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
        </svg>
    ),
    Edit: () => (
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
        </svg>
    ),
    Verified: () => (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
        </svg>
    ),
    Check: () => (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="20 6 9 17 4 12" />
        </svg>
    )
};

const SAMPLE_CODE = `fn main() {
    let numbers: Vec<i32> = (1..=10).collect();
    let sum: i32 = numbers.iter().sum();
    println!("Sum: {}", sum);
}
`;

const EXTENSIONS = [
  '.rs', '.py', '.ts', '.js', '.tsx', '.jsx', '.go', '.cpp', '.c', '.h', '.hpp', 
  '.dart', '.swift', '.java', '.kt', '.kts', '.rb', '.sh', '.bash', '.zsh', 
  '.css', '.scss', '.sass', '.less', '.html', '.md', '.json', '.yaml', '.yml', 
  '.toml', '.xml', '.svg', '.sql', '.env', '.txt',
  'Dockerfile', 'Makefile', 'Gemfile', 'go.mod', 'Cargo.toml', 'package.json'
];

function App() {
  const { ready, processing, midiData, decodedSource, extensionInfo, error, initialize, encodeSource, encodeProjectSource, decodeSource, decodeProjectSource } = useEntropy();
  const { playing, progress, activeFile, play, stop } = useAudioEngine();
  const visualizerRef = useRef<VisualizerHandle>(null);

  // プロジェクトファイルの型定義
  interface ProjectFile {
    name: string;
    extension: string;
    source?: string;
  }

  // リストレンダリング最適化のためのコンポーネント (再生中の1行だけが更新されるようにする)
  const FileListItem = memo(({ 
      file, 
      isActive, 
      rootName 
  }: { 
      file: ProjectFile, 
      isActive: boolean, 
      rootName: string 
  }) => {
      return (
          <div className={`file-item ${isActive ? 'playing' : ''}`}>
              <span className="file-icon">
                  {isActive ? <Icons.Play /> : <Icons.Check />}
              </span>
              <span className="file-name">
                  {formatPath(file.name, rootName)}
              </span>
              <span className="file-ext">{file.extension}</span>
          </div>
      );
  });
  FileListItem.displayName = 'FileListItem';

  const [sourceCode, setSourceCode] = useState(SAMPLE_CODE);
  const [extension, setExtension] = useState('.rs');
  const [status, setStatus] = useState('Initializing WASM...');
  const [uiMode, setUiMode] = useState<'encode' | 'decode'>('encode');
  const [isProjectMode, setIsProjectMode] = useState(false);
  const [projectFiles, setProjectFiles] = useState<{name: string, source: string, extension: string}[]>([]);
  const [filename, setFilename] = useState('logiscore_output');

  useEffect(() => {
    initialize().then(() => setStatus('READY'));
  }, [initialize]);

  const formatPath = (fullPath: string, rootName: string) => {
    let relPath = fullPath;
    if (fullPath.startsWith(rootName + '/')) {
        relPath = fullPath.slice(rootName.length + 1);
    }
    
    const parts = relPath.split('/');
    if (parts.length > 1) {
        const fileName = parts.pop();
        const dirPath = parts.join('/');
        // もしディレクトリ階層が深すぎる場合は後ろの2つだけ残して省略
        const dirParts = dirPath.split('/');
        const displayDir = dirParts.length > 3 
            ? `.../${dirParts.slice(-2).join('/')}` 
            : dirPath;
            
        return (
            <span className="file-name-container">
                <span className="file-path">{displayDir}/</span>
                <span className="file-basename">{fileName}</span>
            </span>
        );
    }
    return <span className="file-basename">{relPath}</span>;
  };

  const handleEncode = () => {
    if (!ready || playing) return;
    setStatus('ENCODING...');
    
    if (isProjectMode && projectFiles.length > 0) {
      const midi = encodeProjectSource(projectFiles);
      if (midi) {
        setStatus(`SYMPHONY CREATED: ${midi.length} BYTES`);
      }
    } else {
      const midi = encodeSource(sourceCode, extension);
      if (midi) {
        setStatus(`ENCODED: ${midi.length} BYTES`);
      }
    }
  };

  const handlePlay = async () => {
    if (!midiData || playing) return;
    setStatus('PLAYING...');
    
    // 再生状況を可視化するために渡すコールバック
    await play(midiData, { noteDuration: 0.2, tickInterval: 0.25 }, 
      (note, vel, dur, isBass) => {
        visualizerRef.current?.addNote(note, vel, dur, isBass);
      },
      () => { // This is the onPlaybackComplete callback
        // プロジェクトモードでなく、かつ midiData が単一ファイルとしてエンコード可能な場合のみ検証
        if (!isProjectMode) {
            setStatus('DECODING...');
            const decoded = decodeSource(midiData);
            if (decoded !== null) {
                setStatus('✅ VERIFIED');
            } else {
                setStatus('❌ DECODING FAILED'); // Added for clarity if decoding fails
            }
        } else {
            setStatus('✅ PLAYBACK COMPLETE');
        }
      }
    );
  };

  const handleStop = () => {
    stop();
    setStatus('STOPPED');
  };

  const handleDownload = () => {
    if (!midiData) return;
    const blob = new Blob([midiData as any], { type: 'audio/midi' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${filename}.mid`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    setStatus('MIDI DOWNLOADED');
  };

    const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file || !ready) return;

    const reader = new FileReader();
    if (file.name.endsWith('.mid') || file.name.endsWith('.midi')) {
      reader.onload = (event) => {
        const arrayBuffer = event.target?.result as ArrayBuffer;
        const uint8Array = new Uint8Array(arrayBuffer);
        stop();
        setStatus('IMPORTING MIDI...');
        
        // プロジェクト形式を先に試行（エラーは無視してフォールバック）
        let imported = false;
        try {
          const project = decodeProjectSource(uint8Array);
          if (project && project.length > 0) {
            setProjectFiles(project);
            setIsProjectMode(true);
            setUiMode('encode');
            setStatus(`✅ PROJECT IMPORTED: ${project.length} FILES`);
            imported = true;
          }
        } catch { /* フォールバックへ */ }
        
        if (!imported) {
          const decoded = decodeSource(uint8Array);
          if (decoded !== null) {
            setSourceCode(decoded.source);
            setExtension(decoded.extension);
            setUiMode('decode');
            setIsProjectMode(false);
            setStatus('✅ IMPORTED & DECODED');
            imported = true;
          }
        }
        
        if (!imported) {
          setStatus('⚠️ This MIDI was not generated by Logiscore');
        }
      };
      reader.readAsArrayBuffer(file);
    } else {
      reader.onload = (event) => {
        const text = event.target?.result as string;
        setSourceCode(text);
        setUiMode('encode');
        const parts = file.name.split('.');
        const ext = '.' + parts.pop();
        if (EXTENSIONS.includes(ext)) {
          setExtension(ext);
        }
        setStatus('✅ SOURCE LOADED');
      };
      reader.readAsText(file);
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  };

  const scanFiles = async (
    entry: FileSystemEntry, 
    path = '', 
    parentIgnores: string[] = ['.git', 'node_modules', 'dist', 'build', 'target', 'pkg', '.DS_Store', '.gemini', '.cursor']
  ): Promise<{name: string, source: string, extension: string}[]> => {
    const files: {name: string, source: string, extension: string}[] = [];
    
    if (entry.isFile) {
        const fileEntry = entry as FileSystemFileEntry;
        const file = await new Promise<File>((resolve) => fileEntry.file(resolve));
        const ext = file.name.includes('.') ? '.' + file.name.split('.').pop() : '';
        if (EXTENSIONS.includes(ext) || EXTENSIONS.includes(file.name)) {
            const source = await file.text();
            files.push({ name: path + file.name, source, extension: ext || file.name });
        }
    } else if (entry.isDirectory) {
        const dirEntry = entry as FileSystemDirectoryEntry;
        const reader = dirEntry.createReader();
        const entries = await new Promise<FileSystemEntry[]>((resolve) => reader.readEntries(resolve));
        
        let currentIgnores = [...parentIgnores];
        const gitignoreEntry = entries.find(e => e.name === '.gitignore') as FileSystemFileEntry | undefined;
        if (gitignoreEntry) {
            try {
                const file = await new Promise<File>((resolve) => gitignoreEntry.file(resolve));
                const text = await file.text();
                const newRules = text.split('\n')
                    .filter(line => line && !line.startsWith('#'));
                currentIgnores = Array.from(new Set([...currentIgnores, ...newRules]));
            } catch (e) {
                console.warn('Failed to read .gitignore', e);
            }
        }

        for (const child of entries) {
            // 無視対象かチェック (簡易実装: 完全一致またはプレフィックス一致)
            const isIgnored = currentIgnores.some(pattern => {
                const p = pattern.endsWith('/') ? pattern.slice(0, -1) : pattern;
                if (p === '*') return true;
                return child.name === p || child.name.startsWith(p + '/');
            });

            if (isIgnored && child.name !== '.gitignore') continue;

            const childFiles = await scanFiles(child, path + entry.name + '/', currentIgnores);
            files.push(...childFiles);
        }
    }
    return files;
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    const items = e.dataTransfer.items;
    if (items && items.length > 0) {
        const item = items[0].webkitGetAsEntry();
        if (item && item.isDirectory) {
            setStatus('SCANNING PROJECT...');
            const results = await scanFiles(item);
            if (results.length > 0) {
                setProjectFiles(results);
                setIsProjectMode(true);
                setFilename(item.name);
                setStatus(`PROJECT LOADED: ${results.length} FILES`);
            }
            return;
        }
    }

    const file = e.dataTransfer.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    if (file.name.endsWith('.mid') || file.name.endsWith('.midi')) {
      reader.onload = (event) => {
        const arrayBuffer = event.target?.result as ArrayBuffer;
        const uint8Array = new Uint8Array(arrayBuffer);
        stop();
        setStatus('IMPORTING MIDI...');
        
        // プロジェクト形式を先に試行（エラーは無視してフォールバック）
        let imported = false;
        try {
          const project = decodeProjectSource(uint8Array);
          if (project && project.length > 0) {
            setProjectFiles(project);
            setIsProjectMode(true);
            setUiMode('encode');
            setStatus(`✅ PROJECT DROPPED: ${project.length} FILES`);
            imported = true;
          }
        } catch { /* フォールバックへ */ }
        
        if (!imported) {
          const decoded = decodeSource(uint8Array);
          if (decoded !== null) {
            setSourceCode(decoded.source);
            setExtension(decoded.extension);
            setUiMode('decode');
            setIsProjectMode(false);
            setStatus('✅ DROP & DECODED SUCCESS');
            imported = true;
          }
        }
        
        if (!imported) {
          setStatus('⚠️ This MIDI was not generated by Logiscore');
        }
      };
      reader.readAsArrayBuffer(file);
    } else {
      // ソースコードとしてのドロップ
      reader.onload = (event) => {
        const text = event.target?.result as string;
        setSourceCode(text);
        setUiMode('encode'); 
        setIsProjectMode(false);
        const parts = file.name.split('.');
        const ext = '.' + parts.pop();
        setFilename(parts.join('.') || 'logiscore_output');
        if (EXTENSIONS.includes(ext)) {
          setExtension(ext);
        }
        setStatus('✅ SOURCE DROPPED');
      };
      reader.readAsText(file);
    }
  };

  const verified = decodedSource !== null && decodedSource === sourceCode;

  return (
    <div 
        className="app"
        onDragOver={handleDragOver}
        onDrop={handleDrop}
    >
      <header className="header">
        <motion.div 
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            className="logo"
        >
          <span className="logo-text">L O G I S C O R E</span>
          <span className="logo-version">v1.7</span>
        </motion.div>
        <div className="status-bar-container">
            <motion.div 
                key={status}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                className="status-bar"
            >
              <div className="status-indicator" />
              {status}
            </motion.div>
        </div>

        <div className="header-meta">
            {isProjectMode && (
                <div className="project-badge">
                    <Icons.Node />
                    <span>{projectFiles.length} FILES</span>
                </div>
            )}
            <div className="filename-badge">
                <Icons.Check />
                <span>{filename}</span>
            </div>
        </div>
      </header>

      <main className="main">
        {/* Visualizer Background */}
        <div className="visualizer-bg">
            <Visualizer ref={visualizerRef} />
        </div>

        <div className="layout-grid">
            {/* --- LEFT PANEL --- */}
            <motion.div 
                key={`left-${uiMode}`}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.1 }}
            >
                {uiMode === 'encode' ? (
                    <Panel 
                        title={isProjectMode ? "PROJECT RADIUS" : "SOURCE CODE"} 
                        className="editor-panel"
                        headerAction={
                            <div className="header-actions">
                                <button className={`btn-icon ${isProjectMode ? 'active' : ''}`} title="Project Mode" onClick={() => setIsProjectMode(!isProjectMode)}>
                                    <Icons.Node />
                                </button>
                                <label className="btn-icon" title="Import File">
                                    <Icons.Import />
                                    <input 
                                        type="file" 
                                        accept=".mid,.midi,.rs,.py,.ts,.go,.cpp,.rb,.css,.md,.json,.yaml,.toml" 
                                        style={{ display: 'none' }} 
                                        onChange={handleImport}
                                    />
                                </label>
                                {!isProjectMode && (
                                    <select
                                        className="ext-select"
                                        value={extension}
                                        onChange={(e) => setExtension(e.target.value)}
                                    >
                                        {EXTENSIONS.map(ext => (
                                            <option key={ext} value={ext}>{ext}</option>
                                        ))}
                                    </select>
                                )}
                            </div>
                        }
                    >
                        {isProjectMode ? (
                            <div className="project-file-list">
                                <div className="project-header">
                                    <span className="project-name">{filename}</span>
                                    <span className="project-count">{projectFiles.length} tracks</span>
                                </div>
                                <div className="file-items">
                                    {projectFiles.map((f) => (
                                        <FileListItem 
                                            key={f.name}
                                            file={f}
                                            isActive={activeFile === f.name}
                                            rootName={filename}
                                        />
                                    ))}
                                    {projectFiles.length === 0 && (
                                        <div className="project-placeholder">
                                            Drop a folder here to start Symphony
                                        </div>
                                    )}
                                </div>
                            </div>
                        ) : (
                            <textarea
                                className="code-input"
                                value={sourceCode}
                                onChange={(e) => {
                                    setSourceCode(e.target.value);
                                    setUiMode('encode');
                                }}
                                spellCheck={false}
                                placeholder="Paste your source code here..."
                            />
                        )}
                    </Panel>
                ) : (
                    <Panel 
                        title="INPUT MIDI" 
                        className="input-panel-midi"
                        headerAction={
                            <label className="btn-icon" title="Import File">
                                <Icons.Import />
                                <input 
                                    type="file" 
                                    accept=".mid,.midi,.rs,.py,.ts,.go,.cpp,.rb,.css,.md,.json,.yaml,.toml" 
                                    style={{ display: 'none' }} 
                                    onChange={handleImport}
                                />
                            </label>
                        }
                    >
                        <div className="midi-output-container">
                             <div className="midi-card input-version">
                                <div className="midi-card-icon"><Icons.Node /></div>
                                <div className="midi-card-info">
                                    <div className="midi-filename">Imported MIDI Data</div>
                                    <div className="midi-filesize">{midiData?.length.toLocaleString() ?? '0'} BYTES</div>
                                </div>
                                <div className="input-label">READY TO DECODE</div>
                                <button className="btn-icon circle-large" onClick={() => setUiMode('encode')}>
                                    <Icons.Edit />
                                </button>
                                <p className="hint-text">Click to edit source code</p>
                            </div>
                        </div>
                    </Panel>
                )}
            </motion.div>

            {/* --- CENTER PANEL --- */}
            <motion.div 
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.2 }}
            >
                <Panel title="CONTROLS" className="controls-panel">
                    {extensionInfo && (
                        <div className="info-display">
                        <div className="info-row">
                            <span className="info-label">Language</span>
                            <span className="info-value">{extensionInfo.name}</span>
                        </div>
                        <div className="info-row">
                            <span className="info-label">Scale</span>
                            <span className="info-value">{extensionInfo.scale_name}</span>
                        </div>
                        <div className="info-row">
                            <span className="info-label">Root</span>
                            <span className="info-value">{['C','C#','D','D#','E','F','F#','G','G#','A','A#','B'][extensionInfo.root_key]}</span>
                        </div>
                        </div>
                    )}

                    <div className="button-group">
                        <button
                        className={`btn btn-encode ${processing ? 'shimmer' : ''}`}
                        onClick={handleEncode}
                        disabled={!ready || playing || processing}
                        >
                        {processing ? <div className="spinner-small" /> : 'ENCODE'}
                        </button>
                        <button
                        className="btn btn-play"
                        onClick={handlePlay}
                        disabled={!midiData || playing}
                        >
                        <Icons.Play /> PLAY
                        </button>
                        <button
                        className="btn btn-stop"
                        onClick={handleStop}
                        disabled={!playing}
                        >
                        <Icons.Stop /> STOP
                        </button>
                    </div>

                    <div className="progress-container">
                        <div className="progress-label">PROGRESS</div>
                        <div className="progress-bar-container">
                            <motion.div 
                                className="progress-bar" 
                                animate={{ width: `${progress}%` }}
                                transition={{ type: 'spring', bounce: 0, duration: 0.3 }}
                            />
                        </div>
                    </div>

                    {midiData && (
                        <div className="midi-info">
                        MIDI: {midiData.length.toLocaleString()} BYTES
                        </div>
                    )}
                </Panel>
            </motion.div>

            {/* --- RIGHT PANEL --- */}
            <motion.div 
                key={`right-${uiMode}`}
                initial={{ opacity: 0, x: 20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.3 }}
            >
                {uiMode === 'encode' ? (
                    <Panel 
                        title="MIDI MASTER" 
                        className="output-panel" 
                        verified={verified}
                    >
                        <div className="midi-output-container">
                            <AnimatePresence mode="wait">
                                {midiData ? (
                                    <motion.div 
                                        key="midi-card"
                                        initial={{ opacity: 0, y: 10 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, scale: 0.9 }}
                                        className="midi-card"
                                    >
                                        <div className="midi-card-icon">🎼</div>
                                        <div className="midi-card-info">
                                            <div className="midi-filename">{filename}.mid</div>
                                            <div className="midi-filesize">{midiData.length.toLocaleString()} BYTES</div>
                                        </div>
                                        <button className="btn-download-main" onClick={handleDownload}>
                                            DOWNLOAD MIDI
                                        </button>
                                        
                                        <div className="verification-status">
                                            {verified ? '✅ LOSSLESS VERIFIED' : '... UNVERIFIED'}
                                        </div>
                                        
                                        {decodedSource && (
                                            <div className="mini-preview">
                                                <div className="preview-header">DECODED VERIFICATION</div>
                                                <pre className="preview-content">{decodedSource.slice(0, 100)}...</pre>
                                            </div>
                                        )}
                                    </motion.div>
                                ) : (
                                    <motion.div 
                                        key="placeholder"
                                        initial={{ opacity: 0 }}
                                        animate={{ opacity: 1 }}
                                        className="midi-placeholder"
                                    >
                                        <div className="placeholder-icon">🎵</div>
                                        <p>ENCODE YOUR CODE TO MUSIC</p>
                                    </motion.div>
                                )}
                            </AnimatePresence>
                        </div>
                    </Panel>
                ) : (
                    <Panel 
                        title="DECODED SOURCE" 
                        className="editor-panel"
                        verified={verified}
                        headerAction={
                             <button className="btn-icon" onClick={handleDownload} title="Download MIDI">
                                <Icons.Download />
                            </button>
                        }
                    >
                        <textarea
                            className="code-input"
                            value={sourceCode}
                            onChange={(e) => setSourceCode(e.target.value)}
                            spellCheck={false}
                        />
                    </Panel>
                )}
            </motion.div>
        </div>
      </main>

      <AnimatePresence>
          {error && (
            <motion.div 
                initial={{ opacity: 0, y: 20, x: '-50%' }}
                animate={{ opacity: 1, y: 0, x: '-50%' }}
                exit={{ opacity: 0, y: 20, x: '-50%' }}
                className="error-toast"
            >
              笞��� {error}
            </motion.div>
          )}
      </AnimatePresence>
    </div>
  );
}

export default App;
