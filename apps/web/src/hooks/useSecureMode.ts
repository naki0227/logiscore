import { useMemo, useState } from "react";
import { validateSecurePassword } from "../features/secure/model";

export function useSecureMode() {
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const validation = useMemo(
    () => validateSecurePassword(password, confirmation),
    [password, confirmation],
  );

  return {
    password,
    confirmation,
    validation,
    setPassword,
    setConfirmation,
  };
}
