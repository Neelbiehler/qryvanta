"use client";

import Link from "next/link";
import {
  type FormEvent,
  useEffect,
  useMemo,
  useRef,
  useState,
  useTransition,
} from "react";
import { usePathname, useRouter } from "next/navigation";
import { ChevronDown, LogOut, Search, ChevronRight } from "lucide-react";

import {
  Avatar,
  AvatarFallback,
  Button,
  Input,
  StatusBadge,
} from "@qryvanta/ui";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@qryvanta/ui/dropdown-menu";
import { apiFetch, type UserIdentityResponse } from "@/lib/api";
import {
  readAccessibleSurfaces,
  type SurfaceId,
  SURFACES,
  SURFACE_ORDER,
} from "@/lib/surfaces";

import { cn } from "@/lib/utils";

type HeaderProps = {
  user: UserIdentityResponse;
  surfaceId?: SurfaceId;
};

type HeaderCommandTarget = {
  label: string;
  href: string;
};

function segmentToLabel(segment: string): string {
  return segment
    .replace(/[-_]/g, " ")
    .split(" ")
    .filter((word) => word.length > 0)
    .map((word) => `${word.charAt(0).toUpperCase()}${word.slice(1)}`)
    .join(" ");
}

export function Header({ user, surfaceId }: HeaderProps) {
  const pathname = usePathname();
  const router = useRouter();
  const [isPending, startTransition] = useTransition();
  const [commandText, setCommandText] = useState("");
  const commandInputRef = useRef<HTMLInputElement | null>(null);

  const name = user.display_name.trim();
  const initials = name
    ? name
        .split(" ")
        .slice(0, 2)
        .map((part) => part.at(0)?.toUpperCase() ?? "")
        .join("")
    : "U";

  const surfaceLabel = surfaceId ? SURFACES[surfaceId].label : "Workspace";
  const resolvedAccessibleSurfaces = readAccessibleSurfaces(user);
  const accessibleSurfaces = SURFACE_ORDER.filter((id) =>
    resolvedAccessibleSurfaces.includes(id),
  );

  const commandTargets = useMemo<HeaderCommandTarget[]>(() => {
    const candidates = accessibleSurfaces.flatMap((id) => {
      const surface = SURFACES[id];
      return [
        { label: `${surface.label} Home`, href: surface.basePath },
        ...surface.navigationItems.map((item) => ({
          label: `${surface.label}: ${item.label}`,
          href: item.href,
        })),
      ];
    });

    const uniqueTargets = new Map<string, HeaderCommandTarget>();
    for (const candidate of candidates) {
      uniqueTargets.set(candidate.href, candidate);
    }

    return [...uniqueTargets.values()];
  }, [accessibleSurfaces]);

  const breadcrumbs = useMemo(() => {
    if (!surfaceId) {
      return [{ label: "Workspace", href: "/" }];
    }

    const surface = SURFACES[surfaceId];
    const currentPath = pathname ?? surface.basePath;
    const relativePath = currentPath.startsWith(surface.basePath)
      ? currentPath.slice(surface.basePath.length)
      : "";

    const segments = relativePath
      .split("/")
      .filter((segment) => segment.length > 0);
    const nextBreadcrumbs: Array<{ label: string; href: string }> = [
      { label: surface.label, href: surface.basePath },
    ];

    let runningPath = surface.basePath;
    for (const segment of segments) {
      runningPath = `${runningPath}/${segment}`;
      const matchingNav = surface.navigationItems.find(
        (item) => item.href === runningPath,
      );

      nextBreadcrumbs.push({
        label: matchingNav?.label ?? segmentToLabel(segment),
        href: runningPath,
      });
    }

    return nextBreadcrumbs;
  }, [pathname, surfaceId]);

  const pageTitle = breadcrumbs.at(-1)?.label ?? surfaceLabel;

  useEffect(() => {
    function handleKeyboardShortcut(event: KeyboardEvent) {
      if (event.key !== "/" || event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      const target = event.target as HTMLElement | null;
      if (
        target?.tagName === "INPUT" ||
        target?.tagName === "TEXTAREA" ||
        target?.isContentEditable
      ) {
        return;
      }

      event.preventDefault();
      commandInputRef.current?.focus();
    }

    window.addEventListener("keydown", handleKeyboardShortcut);
    return () => {
      window.removeEventListener("keydown", handleKeyboardShortcut);
    };
  }, []);

  async function handleLogout() {
    startTransition(() => {
      void (async () => {
        await apiFetch("/auth/logout", { method: "POST" });
        router.replace("/login");
        router.refresh();
      })();
    });
  }

  function handleOpenCommandTarget(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalized = commandText.trim();
    if (!normalized) {
      return;
    }

    const matched = commandTargets.find((candidate) => {
      const normalizedCandidateLabel = candidate.label.toLowerCase();
      const normalizedCandidateHref = candidate.href.toLowerCase();
      const normalizedInput = normalized.toLowerCase();

      return (
        normalizedCandidateLabel === normalizedInput ||
        normalizedCandidateHref === normalizedInput
      );
    });

    if (matched) {
      router.push(matched.href);
      setCommandText("");
      return;
    }

    if (normalized.startsWith("/")) {
      router.push(normalized);
      setCommandText("");
    }
  }

  return (
    <header className="border-b border-emerald-100/80 bg-white/60 backdrop-blur-sm">
      <div className="flex items-center justify-between gap-4 px-4 py-2.5 md:px-6">
        {/* Left: Breadcrumbs & Title */}
        <div className="flex min-w-0 flex-1 items-center gap-3">
          {/* Breadcrumbs */}
          <nav className="hidden text-[11px] text-zinc-500 md:flex md:items-center">
            {breadcrumbs.map((breadcrumb, index) => (
              <div key={breadcrumb.href} className="flex items-center">
                {index > 0 && (
                  <ChevronRight className="mx-1 h-3 w-3 text-zinc-300" />
                )}
                <Link
                  href={breadcrumb.href}
                  className="transition-colors hover:text-zinc-700"
                >
                  {breadcrumb.label}
                </Link>
              </div>
            ))}
          </nav>
        </div>

        {/* Center: Command Bar */}
        <form
          className="flex max-w-md flex-1 items-center"
          onSubmit={handleOpenCommandTarget}
        >
          <div className="relative flex-1">
            <Search className="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-zinc-400" />
            <Input
              ref={commandInputRef}
              value={commandText}
              onChange={(event) => setCommandText(event.target.value)}
              placeholder="Jump to... (press /)"
              list="surface-command-targets"
              className="h-8 border-emerald-100 pl-8 pr-16 text-sm placeholder:text-zinc-400"
            />
            <div className="absolute right-2 top-1/2 -translate-y-1/2">
              <kbd className="hidden rounded border border-zinc-200 bg-zinc-50 px-1.5 py-0.5 text-[10px] font-medium text-zinc-400 sm:inline">
                /
              </kbd>
            </div>
            <datalist id="surface-command-targets">
              {commandTargets.map((target) => (
                <option key={target.href} value={target.href}>
                  {target.label}
                </option>
              ))}
            </datalist>
          </div>
        </form>

        {/* Right: User Actions */}
        <div className="flex items-center gap-2">
          <StatusBadge tone="success" className="hidden sm:inline-flex">
            Live
          </StatusBadge>

          <DropdownMenu>
            <DropdownMenuTrigger>
              <Button variant="ghost" size="sm" className="gap-2 pl-2 pr-3">
                <Avatar className="h-6 w-6">
                  <AvatarFallback className="bg-emerald-100 text-[10px] font-medium text-emerald-700">
                    {initials}
                  </AvatarFallback>
                </Avatar>
                <span className="hidden max-w-28 truncate text-xs font-medium text-zinc-700 sm:inline">
                  {user.display_name}
                </span>
                <ChevronDown className="h-3.5 w-3.5 text-zinc-400" />
              </Button>
            </DropdownMenuTrigger>

            <DropdownMenuContent className="w-56">
              <div className="border-b border-zinc-100 px-3 py-2">
                <p className="text-xs font-medium text-zinc-900">
                  {user.display_name}
                </p>
                <p className="text-[10px] text-zinc-500">
                  {user.email ?? user.subject}
                </p>
              </div>

              <div className="px-3 py-1.5">
                <p className="mb-1 text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
                  Switch Surface
                </p>
                {accessibleSurfaces.map((id) => {
                  const target = SURFACES[id];
                  const isCurrent = surfaceId === id;
                  return (
                    <DropdownMenuItem
                      key={id}
                      disabled={isCurrent}
                      onClick={() => {
                        router.push(target.basePath);
                      }}
                      className={cn(
                        "text-xs",
                        isCurrent && "bg-zinc-50 text-zinc-500",
                      )}
                    >
                      {target.label}
                      {isCurrent && (
                        <span className="ml-auto text-[10px] text-zinc-400">
                          Current
                        </span>
                      )}
                    </DropdownMenuItem>
                  );
                })}
              </div>

              <div className="border-t border-zinc-100 px-3 py-1.5">
                <DropdownMenuItem
                  onClick={handleLogout}
                  disabled={isPending}
                  className="text-xs text-red-600 focus:text-red-600"
                >
                  <LogOut className="mr-2 h-3.5 w-3.5" />
                  {isPending ? "Logging out..." : "Logout"}
                </DropdownMenuItem>
              </div>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>
    </header>
  );
}
