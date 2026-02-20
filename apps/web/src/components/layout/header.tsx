"use client";

import { useMemo, useState } from "react";
import { LogOut, ChevronDown } from "lucide-react";

import {
  Avatar,
  AvatarFallback,
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@qryvanta/ui";
import { apiFetch, type UserIdentityResponse } from "@/lib/api";

type HeaderProps = {
  user: UserIdentityResponse;
};

export function Header({ user }: HeaderProps) {
  const [open, setOpen] = useState(false);
  const initials = useMemo(() => {
    const name = user.display_name.trim();
    if (!name) return "U";

    return name
      .split(" ")
      .slice(0, 2)
      .map((part) => part.at(0)?.toUpperCase() ?? "")
      .join("");
  }, [user.display_name]);

  async function handleLogout() {
    await apiFetch("/auth/logout", { method: "POST" });
    window.location.href = "/login";
  }

  return (
    <header className="flex items-center justify-between border-b border-emerald-100 bg-white/90 px-6 py-3 backdrop-blur">
      <div>
        <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">Workspace</p>
        <h1 className="font-serif text-xl text-zinc-900">Metadata Builder</h1>
      </div>

      <DropdownMenu>
        <DropdownMenuTrigger>
          <Button
            variant="outline"
            className="gap-2"
            onClick={() => setOpen((current) => !current)}
            type="button"
          >
            <Avatar>
              <AvatarFallback>{initials}</AvatarFallback>
            </Avatar>
            <span className="max-w-36 truncate text-left">{user.display_name}</span>
            <ChevronDown className="h-4 w-4" />
          </Button>
        </DropdownMenuTrigger>

        {open ? (
          <DropdownMenuContent>
            <p className="px-2 py-1 text-xs uppercase tracking-[0.14em] text-zinc-500">
              {user.email ?? user.subject}
            </p>
            <DropdownMenuItem onClick={handleLogout}>
              <LogOut className="mr-2 h-4 w-4" />
              Logout
            </DropdownMenuItem>
          </DropdownMenuContent>
        ) : null}
      </DropdownMenu>
    </header>
  );
}
