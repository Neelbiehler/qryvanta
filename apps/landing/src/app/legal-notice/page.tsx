import type { Metadata } from "next";
import Link from "next/link";

import { ProtectedEmailLink } from "@/components/legal/protected-email-link";

const LEGAL_EMAIL_LOCAL_PART = [108, 101, 103, 97, 108] as const;
const LEGAL_EMAIL_DOMAIN = [113, 114, 121, 118, 97, 110, 116, 97] as const;
const LEGAL_EMAIL_TLD = [111, 114, 103] as const;

export const metadata: Metadata = {
  title: "Legal Notice",
  description:
    "Legal disclosure for qryvanta.org pursuant to § 5 TMG (German Telemedia Act).",
  robots: { index: false, follow: false, nocache: true },
  alternates: {
    canonical: "https://qryvanta.org/legal-notice",
  },
};

export default function LegalNoticePage() {
  return (
    <div className="mx-auto max-w-2xl px-6 py-16">
      <nav className="mb-10 text-xs text-slate-400">
        <Link href="/" className="hover:text-slate-600">
          qryvanta.org
        </Link>
        <span className="mx-2">/</span>
        <span className="text-slate-600">Legal Notice</span>
      </nav>

      <h1 className="text-2xl font-semibold text-slate-900">Legal Notice</h1>
      <p className="mt-1 text-xs text-slate-400">
        Pursuant to § 5 TMG (German Telemedia Act).{" "}
        <Link href="/impressum" className="text-emerald-700 hover:underline underline-offset-2">
          Deutsche Version (Impressum)
        </Link>
      </p>

      <section className="mt-8 space-y-1 text-sm text-slate-700">
        <p className="font-semibold">QRYVANTA UG (haftungsbeschränkt)</p>
        {/* TODO: Replace with actual registered address */}
        <p>[STREET AND HOUSE NUMBER]</p>
        <p>[POSTAL CODE] [CITY]</p>
        <p>Germany</p>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">Managing Director</p>
        {/* TODO: Replace with full name of managing director */}
        <p className="mt-1">[FIRST NAME LAST NAME]</p>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">Commercial Register</p>
        <div className="mt-1 space-y-0.5">
          {/* TODO: Replace with correct local court and registration number */}
          <p>Registered in the commercial register.</p>
          <p>Registration court: Local court (Amtsgericht) [CITY]</p>
          <p>Registration number: HRB [NUMBER]</p>
        </div>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">Contact</p>
        <p className="mt-1">
          Email:{" "}
          <ProtectedEmailLink
            localPart={LEGAL_EMAIL_LOCAL_PART}
            domain={LEGAL_EMAIL_DOMAIN}
            tld={LEGAL_EMAIL_TLD}
            className="text-emerald-700 underline-offset-2 hover:underline"
            fallback="legal [at] qryvanta [dot] org"
          />
        </p>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">VAT Identification Number</p>
        {/*
          TODO: Add your VAT ID once issued.
          If not yet assigned, use: "VAT ID not yet assigned."
        */}
        <p className="mt-1">
          Pursuant to § 27a of the German VAT Act: [DE-NUMBER or &quot;not yet assigned&quot;]
        </p>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">
          Person Responsible for Content (§ 55 para. 2 RStV)
        </p>
        {/* TODO: Full name and address */}
        <div className="mt-1 space-y-0.5">
          <p>[FIRST NAME LAST NAME]</p>
          <p>[STREET AND HOUSE NUMBER]</p>
          <p>[POSTAL CODE] [CITY]</p>
        </div>
      </section>

      <section className="mt-10 space-y-4 text-sm text-slate-600">
        <div>
          <p className="font-semibold text-slate-900">Dispute Resolution</p>
          <p className="mt-1">
            The European Commission provides a platform for online dispute
            resolution (ODR):{" "}
            <a
              href="https://ec.europa.eu/consumers/odr/"
              target="_blank"
              rel="noopener noreferrer"
              className="text-emerald-700 underline-offset-2 hover:underline"
            >
              https://ec.europa.eu/consumers/odr/
            </a>
          </p>
          <p className="mt-1">
            We are not willing or obliged to participate in dispute resolution
            proceedings before a consumer arbitration board.
          </p>
        </div>

        <div>
          <p className="font-semibold text-slate-900">
            Liability for Content
          </p>
          <p className="mt-1">
            As a service provider we are responsible for our own content on
            these pages under general law pursuant to § 7 para. 1 TMG. Under
            §§ 8–10 TMG we are not obliged to monitor transmitted or stored
            third-party information or to investigate circumstances indicating
            illegal activity. Obligations to remove or block the use of
            information under general law remain unaffected. Liability in this
            regard is only possible from the point in time at which a specific
            legal violation becomes known.
          </p>
        </div>

        <div>
          <p className="font-semibold text-slate-900">Liability for Links</p>
          <p className="mt-1">
            Our website contains links to external third-party websites over
            whose content we have no control. We cannot accept any liability for
            that external content. The respective provider or operator of the
            linked pages is always responsible for the content of those pages.
            The linked pages were checked for possible legal violations at the
            time of linking. If we become aware of any legal violations, we will
            remove such links immediately.
          </p>
        </div>

        <div>
          <p className="font-semibold text-slate-900">Copyright</p>
          <p className="mt-1">
            Content created by the site operator is subject to German copyright
            law. The source code of the Qryvanta project is available under the
            applicable open-source licence. Where content on this site was not
            created by the operator, third-party copyrights are observed.
          </p>
        </div>
      </section>

      <div className="mt-12 border-t border-emerald-100 pt-6 text-xs text-slate-400">
        <Link href="/privacy" className="hover:text-slate-600">
          Privacy Policy
        </Link>
        <span className="mx-3">·</span>
        <Link href="/impressum" className="hover:text-slate-600">
          Impressum (Deutsch)
        </Link>
        <span className="mx-3">·</span>
        <Link href="/" className="hover:text-slate-600">
          Back to home
        </Link>
      </div>
    </div>
  );
}
