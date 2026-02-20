"use client";

import { useState } from "react";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Label,
} from "@qryvanta/ui";
import { API_BASE_URL } from "@/lib/api";

function decodeBase64Url(input: string): ArrayBuffer {
  const normalized = input.replace(/-/g, "+").replace(/_/g, "/");
  const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, "=");
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes.buffer;
}

function encodeBase64Url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }

  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}

type CredentialDescriptor = {
  id: string;
  type: PublicKeyCredentialType;
};

type LoginChallengeResponse = {
  publicKey: {
    challenge: string;
    timeout?: number;
    userVerification?: UserVerificationRequirement;
    allowCredentials?: CredentialDescriptor[];
    rpId?: string;
  };
};

type RegistrationChallengeResponse = {
  publicKey: {
    challenge: string;
    rp: { id: string; name: string };
    user: { id: string; name: string; displayName: string };
    pubKeyCredParams: Array<{ type: PublicKeyCredentialType; alg: number }>;
    timeout?: number;
    excludeCredentials?: CredentialDescriptor[];
    authenticatorSelection?: AuthenticatorSelectionCriteria;
    attestation?: AttestationConveyancePreference;
  };
};

type ErrorResponse = {
  message?: string;
};

async function readErrorMessage(response: Response, fallback: string): Promise<string> {
  try {
    const contentType = response.headers.get("content-type") ?? "";
    if (contentType.includes("application/json")) {
      const payload = (await response.json()) as ErrorResponse;
      if (payload.message?.trim()) {
        return payload.message;
      }

      return fallback;
    }

    const body = (await response.text()).trim();
    if (body) {
      return body;
    }
  } catch {
    return fallback;
  }

  return fallback;
}

export default function LoginPage() {
  const [subject, setSubject] = useState("");
  const [bootstrapToken, setBootstrapToken] = useState("");
  const [status, setStatus] = useState<string>("");
  const [loading, setLoading] = useState(false);

  async function bootstrapSession() {
    if (!subject || !bootstrapToken) {
      setStatus("Enter subject and bootstrap token first.");
      return;
    }

    setLoading(true);
    try {
      const response = await fetch(`${API_BASE_URL}/auth/bootstrap`, {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ subject, token: bootstrapToken }),
      });

      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Bootstrap failed."));
        return;
      }

      setStatus("Bootstrap successful. You can enroll a passkey from settings after login.");
    } catch {
      setStatus("Bootstrap request failed.");
    } finally {
      setLoading(false);
    }
  }

  async function signInWithPasskey() {
    if (!subject) {
      setStatus("Enter your subject first.");
      return;
    }

    setLoading(true);
    try {
      const startResponse = await fetch(
        `${API_BASE_URL}/auth/webauthn/login/start?subject=${encodeURIComponent(subject)}`,
        {
          method: "GET",
          credentials: "include",
        },
      );

      if (!startResponse.ok) {
        setStatus(await readErrorMessage(startResponse, "Unable to start passkey login."));
        return;
      }

      const challenge = (await startResponse.json()) as LoginChallengeResponse;
      const publicKey = challenge.publicKey;

      const assertion = (await navigator.credentials.get({
        publicKey: {
          ...publicKey,
          challenge: decodeBase64Url(publicKey.challenge),
          allowCredentials: publicKey.allowCredentials?.map((credential) => ({
            ...credential,
            id: decodeBase64Url(credential.id),
          })),
        },
      })) as PublicKeyCredential | null;

      if (!assertion) {
        setStatus("Passkey authentication was cancelled.");
        return;
      }

      const response = assertion.response as AuthenticatorAssertionResponse;

      const finishPayload = {
        id: assertion.id,
        rawId: encodeBase64Url(assertion.rawId),
        type: assertion.type,
        response: {
          authenticatorData: encodeBase64Url(response.authenticatorData),
          clientDataJSON: encodeBase64Url(response.clientDataJSON),
          signature: encodeBase64Url(response.signature),
          userHandle: response.userHandle
            ? encodeBase64Url(response.userHandle)
            : null,
        },
      };

      const finishResponse = await fetch(`${API_BASE_URL}/auth/webauthn/login/finish`, {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(finishPayload),
      });

      if (!finishResponse.ok) {
        setStatus(await readErrorMessage(finishResponse, "Passkey verification failed."));
        return;
      }

      window.location.href = "/";
    } catch {
      setStatus("Passkey login failed.");
    } finally {
      setLoading(false);
    }
  }

  async function enrollPasskey() {
    setLoading(true);
    try {
      const startResponse = await fetch(`${API_BASE_URL}/auth/webauthn/register/start`, {
        method: "POST",
        credentials: "include",
      });

      if (!startResponse.ok) {
        setStatus(
          await readErrorMessage(startResponse, "Unable to start passkey enrollment."),
        );
        return;
      }

      const challenge = (await startResponse.json()) as RegistrationChallengeResponse;
      const publicKey = challenge.publicKey;

      const credential = (await navigator.credentials.create({
        publicKey: {
          ...publicKey,
          challenge: decodeBase64Url(publicKey.challenge),
          user: {
            ...publicKey.user,
            id: decodeBase64Url(publicKey.user.id),
          },
          excludeCredentials: publicKey.excludeCredentials?.map((descriptor) => ({
            ...descriptor,
            id: decodeBase64Url(descriptor.id),
          })),
        },
      })) as PublicKeyCredential | null;

      if (!credential) {
        setStatus("Passkey enrollment was cancelled.");
        return;
      }

      const response = credential.response as AuthenticatorAttestationResponse;

      const finishPayload = {
        id: credential.id,
        rawId: encodeBase64Url(credential.rawId),
        type: credential.type,
        response: {
          clientDataJSON: encodeBase64Url(response.clientDataJSON),
          attestationObject: encodeBase64Url(response.attestationObject),
        },
      };

      const finishResponse = await fetch(`${API_BASE_URL}/auth/webauthn/register/finish`, {
        method: "POST",
        credentials: "include",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(finishPayload),
      });

      if (!finishResponse.ok) {
        setStatus(await readErrorMessage(finishResponse, "Passkey enrollment failed."));
        return;
      }

      setStatus("Passkey enrolled. You can now sign in with passkey.");
    } catch {
      setStatus("Passkey enrollment failed.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="grid min-h-screen place-items-center px-6 py-12">
      <Card className="w-full max-w-md">
        <CardHeader>
          <p className="text-xs font-semibold uppercase tracking-[0.18em] text-emerald-700">
            Qryvanta
          </p>
          <CardTitle className="font-serif text-3xl">Sign In</CardTitle>
          <CardDescription>
            Use your passkey to access your tenant workspace.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="subject">Subject</Label>
            <Input
              id="subject"
              value={subject}
              onChange={(event) => setSubject(event.target.value)}
              placeholder="your-subject"
            />
          </div>

          <Button onClick={signInWithPasskey} disabled={loading} className="w-full">
            Sign in with Passkey
          </Button>

          <div className="space-y-2 border-t border-zinc-100 pt-4">
            <Label htmlFor="bootstrap-token">Bootstrap Token</Label>
            <Input
              id="bootstrap-token"
              type="password"
              value={bootstrapToken}
              onChange={(event) => setBootstrapToken(event.target.value)}
              placeholder="only for first-time setup"
            />
            <Button
              onClick={bootstrapSession}
              disabled={loading}
              variant="outline"
              className="w-full"
            >
              Bootstrap Session
            </Button>
            <Button
              onClick={enrollPasskey}
              disabled={loading}
              variant="outline"
              className="w-full"
            >
              Enroll Passkey
            </Button>
          </div>

          {status ? <p className="text-sm text-zinc-600">{status}</p> : null}
        </CardContent>
      </Card>
    </main>
  );
}
