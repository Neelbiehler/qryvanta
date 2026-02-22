import { Workflow } from "lucide-react";

export function FooterStrip() {
  return (
    <footer className="mt-10 flex items-center justify-between gap-3 border-t border-emerald-100/80 pt-5 text-xs text-slate-500">
      <p>Qryvanta landing app</p>
      <div className="flex items-center gap-2">
        <Workflow className="h-3.5 w-3.5" />
        <span>Designed with @qryvanta/ui primitives</span>
      </div>
    </footer>
  );
}
