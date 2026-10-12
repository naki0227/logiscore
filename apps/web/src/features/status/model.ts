export type StatusTone = "neutral" | "success" | "error";

export interface StatusMessage {
  text: string;
  tone: StatusTone;
}

export type SetStatus = (text: string, tone?: StatusTone) => void;
