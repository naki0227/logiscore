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
    return { byteLength, valid: false, message: "Password is required" };
  }
  if (byteLength > MAX_PASSWORD_BYTES) {
    return {
      byteLength,
      valid: false,
      message: `Password must be at most ${MAX_PASSWORD_BYTES} UTF-8 bytes`,
    };
  }
  if (password !== confirmation) {
    return { byteLength, valid: false, message: "Passwords do not match" };
  }
  return { byteLength, valid: true, message: "Passwords match" };
}
