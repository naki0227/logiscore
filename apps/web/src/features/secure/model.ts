export const MAX_PASSWORD_BYTES = 1024;

export interface SecurePasswordState {
  byteLength: number;
  valid: boolean;
  message: string;
}

export function validateSecurePassword(
  password: string,
  confirmation: string,
): SecurePasswordState {
  const byteLength = new TextEncoder().encode(password).length;
  if (byteLength === 0) {
    return {
      byteLength,
      valid: false,
      message: "パスワードを入力してください",
    };
  }
  if (byteLength > MAX_PASSWORD_BYTES) {
    return {
      byteLength,
      valid: false,
      message: `パスワードは${MAX_PASSWORD_BYTES}バイト（UTF-8）以内にしてください`,
    };
  }
  if (password !== confirmation) {
    return { byteLength, valid: false, message: "パスワードが一致しません" };
  }
  return { byteLength, valid: true, message: "パスワードが一致しました" };
}
