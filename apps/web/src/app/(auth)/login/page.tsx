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
import {
  API_BASE_URL,
  type AuthLoginRequest,
  type AuthLoginResponse,
  type AuthMfaVerifyRequest,
  type AuthRegisterRequest,
  type GenericMessageResponse,
} from "@/lib/api";

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
    }
  } catch {
    return fallback;
  }

  return fallback;
}

export default function LoginPage() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [mfaCode, setMfaCode] = useState("");
  const [mfaMethod, setMfaMethod] = useState<"totp" | "recovery">("totp");

  const [registerEmail, setRegisterEmail] = useState("");
  const [registerPassword, setRegisterPassword] = useState("");
  const [registerDisplayName, setRegisterDisplayName] = useState("");

  const [forgotEmail, setForgotEmail] = useState("");

  const [status, setStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [mfaRequired, setMfaRequired] = useState(false);

  async function handleLogin() {
    if (!email || !password) {
      setStatus("Enter email and password.");
      return;
    }

    setIsLoading(true);
    setStatus("");
    try {
      const payload: AuthLoginRequest = { email, password };
      const response = await fetch(`${API_BASE_URL}/auth/login`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Login failed."));
        return;
      }

      const body = (await response.json()) as AuthLoginResponse;
      if (body.status === "mfa_required") {
        setMfaRequired(true);
        setStatus("Enter your MFA code to continue.");
        return;
      }

      window.location.href = "/";
    } catch {
      setStatus("Login request failed.");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleMfaVerify() {
    if (!mfaCode) {
      setStatus("Enter your MFA code.");
      return;
    }

    setIsLoading(true);
    setStatus("");
    try {
      const payload: AuthMfaVerifyRequest = { code: mfaCode, method: mfaMethod };
      const response = await fetch(`${API_BASE_URL}/auth/login/mfa`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        setStatus(await readErrorMessage(response, "MFA verification failed."));
        return;
      }

      window.location.href = "/";
    } catch {
      setStatus("MFA request failed.");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleRegister() {
    if (!registerEmail || !registerPassword || !registerDisplayName) {
      setStatus("Complete all registration fields.");
      return;
    }

    setIsLoading(true);
    setStatus("");
    try {
      const payload: AuthRegisterRequest = {
        email: registerEmail,
        password: registerPassword,
        display_name: registerDisplayName,
      };
      const response = await fetch(`${API_BASE_URL}/auth/register`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Registration failed."));
        return;
      }

      const body = (await response.json()) as GenericMessageResponse;
      setStatus(body.message);
      setRegisterPassword("");
    } catch {
      setStatus("Registration request failed.");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleForgotPassword() {
    if (!forgotEmail) {
      setStatus("Enter your email for password reset.");
      return;
    }

    setIsLoading(true);
    setStatus("");
    try {
      const response = await fetch(`${API_BASE_URL}/auth/forgot-password`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: forgotEmail }),
      });

      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Password reset request failed."));
        return;
      }

      const body = (await response.json()) as GenericMessageResponse;
      setStatus(body.message);
    } catch {
      setStatus("Password reset request failed.");
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <main className="min-h-screen bg-app px-6 py-10">
      <div className="mx-auto grid w-full max-w-6xl gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-emerald-700">
              Qryvanta
            </p>
            <CardTitle className="font-serif text-3xl">Sign In</CardTitle>
            <CardDescription>
              Use your email and password. If MFA is enabled, you will be prompted for a code.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="login-email">Email</Label>
              <Input
                id="login-email"
                type="email"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                placeholder="you@company.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="login-password">Password</Label>
              <Input
                id="login-password"
                type="password"
                value={password}
                onChange={(event) => setPassword(event.target.value)}
                placeholder="Your password"
              />
            </div>
            <Button onClick={handleLogin} disabled={isLoading} className="w-full">
              Continue
            </Button>

            {mfaRequired ? (
              <div className="space-y-3 rounded-md border border-emerald-100 bg-emerald-50/70 p-4">
                <Label htmlFor="mfa-code">MFA Code</Label>
                <Input
                  id="mfa-code"
                  value={mfaCode}
                  onChange={(event) => setMfaCode(event.target.value)}
                  placeholder="123456"
                />
                <div className="grid grid-cols-2 gap-2">
                  <Button
                    type="button"
                    variant={mfaMethod === "totp" ? "default" : "outline"}
                    onClick={() => setMfaMethod("totp")}
                  >
                    TOTP
                  </Button>
                  <Button
                    type="button"
                    variant={mfaMethod === "recovery" ? "default" : "outline"}
                    onClick={() => setMfaMethod("recovery")}
                  >
                    Recovery Code
                  </Button>
                </div>
                <Button onClick={handleMfaVerify} disabled={isLoading} className="w-full">
                  Verify MFA
                </Button>
              </div>
            ) : null}

            <div className="space-y-2 border-t border-zinc-100 pt-4">
              <Label htmlFor="forgot-email">Forgot password</Label>
              <Input
                id="forgot-email"
                type="email"
                value={forgotEmail}
                onChange={(event) => setForgotEmail(event.target.value)}
                placeholder="you@company.com"
              />
              <Button onClick={handleForgotPassword} disabled={isLoading} variant="outline" className="w-full">
                Send reset link
              </Button>
            </div>

            {status ? (
              <p aria-live="polite" className="rounded-md border border-zinc-200 bg-zinc-50 p-3 text-sm text-zinc-700">
                {status}
              </p>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="font-serif text-2xl">Create Account</CardTitle>
            <CardDescription>
              Register a workspace user. You will receive an email verification link.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="register-name">Display Name</Label>
              <Input
                id="register-name"
                value={registerDisplayName}
                onChange={(event) => setRegisterDisplayName(event.target.value)}
                placeholder="Alex Rivera"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="register-email">Email</Label>
              <Input
                id="register-email"
                type="email"
                value={registerEmail}
                onChange={(event) => setRegisterEmail(event.target.value)}
                placeholder="you@company.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="register-password">Password</Label>
              <Input
                id="register-password"
                type="password"
                value={registerPassword}
                onChange={(event) => setRegisterPassword(event.target.value)}
                placeholder="Choose a strong password"
              />
            </div>

            <Button onClick={handleRegister} disabled={isLoading} className="w-full">
              Register
            </Button>
          </CardContent>
        </Card>
      </div>
    </main>
  );
}
