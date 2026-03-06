"use client";

import { useEffect, useState } from "react";

import {
  Button,
  Input,
  Label,
  Notice,
  SegmentedControl,
} from "@qryvanta/ui";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@qryvanta/ui/dialog";

import { type AuthStepUpRequest, apiFetch } from "@/lib/api";
import { apiErrorMessage, readApiError } from "@/lib/api-error";

type StepUpMethod = "password" | "totp" | "recovery";

type StepUpDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onVerified: () => void;
  description?: string;
};

export function StepUpDialog({
  open,
  onOpenChange,
  onVerified,
  description,
}: StepUpDialogProps) {
  const [method, setMethod] = useState<StepUpMethod>("password");
  const [password, setPassword] = useState("");
  const [code, setCode] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      setMethod("password");
      setPassword("");
      setCode("");
      setErrorMessage(null);
    }
  }, [open]);

  async function handleSubmit() {
    setErrorMessage(null);

    const payload: AuthStepUpRequest =
      method === "password"
        ? { password: password.trim(), code: null, method: null }
        : { password: null, code: code.trim(), method };

    if (method === "password" && !payload.password) {
      setErrorMessage("Enter your current password.");
      return;
    }

    if (method !== "password" && !payload.code) {
      setErrorMessage("Enter your MFA code.");
      return;
    }

    setIsSubmitting(true);
    try {
      const response = await apiFetch("/auth/step-up", {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const error = await readApiError(response);
        setErrorMessage(apiErrorMessage(error, "Verification failed."));
        return;
      }

      onOpenChange(false);
      onVerified();
    } catch {
      setErrorMessage("Verification failed.");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent size="sm">
        <DialogHeader>
          <DialogTitle>Confirm your identity</DialogTitle>
          <DialogDescription>
            {description ??
              "Verify your current password or MFA code before continuing."}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-2">
            <p className="text-sm font-medium text-zinc-900">
              Verification method
            </p>
            <SegmentedControl
              value={method}
              onChange={(value) => setMethod(value as StepUpMethod)}
              options={[
                { value: "password", label: "Password" },
                { value: "totp", label: "Authenticator" },
                { value: "recovery", label: "Recovery Code" },
              ]}
            />
          </div>

          {method === "password" ? (
            <div className="space-y-2">
              <Label htmlFor="step_up_password">Current password</Label>
              <Input
                id="step_up_password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
              />
            </div>
          ) : (
            <div className="space-y-2">
              <Label htmlFor="step_up_code">
                {method === "recovery" ? "Recovery code" : "Authenticator code"}
              </Label>
              <Input
                id="step_up_code"
                value={code}
                onChange={(event) => setCode(event.target.value)}
              />
            </div>
          )}

          {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
        </div>

        <DialogFooter className="mt-6">
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
          >
            Cancel
          </Button>
          <Button disabled={isSubmitting} onClick={handleSubmit} type="button">
            {isSubmitting ? "Verifying..." : "Verify"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
