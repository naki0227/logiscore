import { useCallback, useEffect, useRef, useState } from "react";

export function useWavPlayback() {
  const [playing, setPlaying] = useState(false);
  const [progress, setProgress] = useState(0);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const urlRef = useRef<string | null>(null);

  const stop = useCallback(() => {
    const audio = audioRef.current;
    if (audio) {
      audio.pause();
      audio.currentTime = 0;
    }
    audioRef.current = null;
    if (urlRef.current) URL.revokeObjectURL(urlRef.current);
    urlRef.current = null;
    setPlaying(false);
    setProgress(0);
  }, []);

  const play = useCallback(
    async (wavBytes: Uint8Array, onComplete?: () => void) => {
      stop();
      const blob = new Blob([new Uint8Array(wavBytes)], { type: "audio/wav" });
      const url = URL.createObjectURL(blob);
      const audio = new Audio(url);
      audioRef.current = audio;
      urlRef.current = url;
      audio.ontimeupdate = () => {
        const next =
          audio.duration > 0 ? (audio.currentTime / audio.duration) * 100 : 0;
        setProgress(Math.min(next, 100));
      };
      audio.onended = () => {
        stop();
        onComplete?.();
      };
      audio.onerror = () => stop();
      setPlaying(true);
      try {
        await audio.play();
      } catch (cause) {
        stop();
        throw cause;
      }
    },
    [stop],
  );

  useEffect(() => stop, [stop]);

  return { playing, progress, play, stop };
}
