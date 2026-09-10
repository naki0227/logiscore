const MAX_AUDIO_FILE_BYTES = 64 * 1024 * 1024;
const MAX_DECODED_FRAMES = 32_000_000;

export interface DecodedAudio {
  samples: Float32Array;
  sampleRate: number;
}

export async function decodeAudioFile(file: File): Promise<DecodedAudio> {
  if (file.size === 0 || file.size > MAX_AUDIO_FILE_BYTES) {
    throw new Error("Audio file size is outside the supported range");
  }
  const context = new AudioContext();
  try {
    const buffer = await context.decodeAudioData(await file.arrayBuffer());
    if (buffer.length === 0 || buffer.length > MAX_DECODED_FRAMES) {
      throw new Error("Decoded recording is outside the supported duration");
    }
    const channels = Array.from(
      { length: buffer.numberOfChannels },
      (_, index) => buffer.getChannelData(index),
    );
    return {
      samples: mixChannels(channels),
      sampleRate: buffer.sampleRate,
    };
  } finally {
    await context.close();
  }
}

export function mixChannels(channels: readonly Float32Array[]): Float32Array {
  const length = channels[0]?.length ?? 0;
  if (
    channels.length === 0 ||
    length === 0 ||
    channels.some((channel) => channel.length !== length)
  ) {
    throw new Error("Audio channels must have matching non-empty dimensions");
  }
  const mono = new Float32Array(length);
  for (const channel of channels) {
    for (let index = 0; index < length; index++) {
      mono[index] += channel[index] / channels.length;
    }
  }
  return mono;
}
