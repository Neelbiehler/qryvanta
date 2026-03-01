import type { Metadata } from "next";
import Link from "next/link";

import { ProtectedEmailLink } from "@/components/legal/protected-email-link";

const LEGAL_EMAIL_LOCAL_PART = [108, 101, 103, 97, 108] as const;
const LEGAL_EMAIL_DOMAIN = [113, 114, 121, 118, 97, 110, 116, 97] as const;
const LEGAL_EMAIL_TLD = [111, 114, 103] as const;

export const metadata: Metadata = {
  title: "Privacy Policy",
  description:
    "Privacy policy for qryvanta.org — what data is processed, why, and your rights under GDPR.",
  robots: { index: false, follow: false, nocache: true },
  alternates: { canonical: "https://qryvanta.org/privacy" },
};

export default function PrivacyPage() {
  return (
    <div className="mx-auto max-w-2xl px-6 py-16">
      <nav className="mb-10 text-xs text-slate-400">
        <Link href="/" className="hover:text-slate-600">
          qryvanta.org
        </Link>
        <span className="mx-2">/</span>
        <span className="text-slate-600">Privacy Policy</span>
      </nav>

      <h1 className="text-2xl font-semibold text-slate-900">Privacy Policy</h1>
      <p className="mt-1 text-xs text-slate-400">
        Last updated: March 2026.{" "}
        <Link
          href="/datenschutz"
          className="text-emerald-700 hover:underline underline-offset-2"
        >
          Deutsche Version (Datenschutzerklärung)
        </Link>
      </p>

      <div className="mt-8 space-y-8 text-sm text-slate-600">

        {/* 1. Controller */}
        <section>
          <h2 className="font-semibold text-slate-900">1. Data Controller</h2>
          <p className="mt-2">
            The controller responsible for data processing on this website is:
          </p>
          <div className="mt-2 space-y-0.5 text-slate-700">
            <p className="font-medium">QRYVANTA UG (haftungsbeschränkt)</p>
            {/* TODO: Replace with actual address */}
            <p>[STREET AND HOUSE NUMBER]</p>
            <p>[POSTAL CODE] [CITY]</p>
            <p>Germany</p>
            <p className="mt-2">
              Email:{" "}
              <ProtectedEmailLink
                localPart={LEGAL_EMAIL_LOCAL_PART}
                domain={LEGAL_EMAIL_DOMAIN}
                tld={LEGAL_EMAIL_TLD}
                className="text-emerald-700 underline-offset-2 hover:underline"
                fallback="legal [at] qryvanta [dot] org"
              />
            </p>
          </div>
        </section>

        {/* 2. What data */}
        <section>
          <h2 className="font-semibold text-slate-900">2. Data We Process</h2>
          <p className="mt-2">
            This website does not use cookies, analytics, tracking pixels, or
            any form that collects personal data. No user accounts exist.
          </p>
          <p className="mt-2">
            When you visit this website, our hosting provider automatically
            records standard server log data, including:
          </p>
          <ul className="mt-2 list-inside list-disc space-y-1 pl-2">
            <li>IP address of the requesting device</li>
            <li>Date and time of the request</li>
            <li>Requested URL and HTTP status code</li>
            <li>Amount of data transferred</li>
            <li>Referring URL (if sent by your browser)</li>
            <li>Browser type and operating system</li>
          </ul>
          <p className="mt-2">
            This data is not combined with any other data source and is used
            only to keep the site running and to identify errors.
          </p>
        </section>

        {/* 3. Legal basis */}
        <section>
          <h2 className="font-semibold text-slate-900">3. Legal Basis</h2>
          <p className="mt-2">
            Server log processing is based on Art. 6(1)(f) GDPR — legitimate
            interest in secure and error-free operation of the website.
          </p>
        </section>

        {/* 4. Retention */}
        <section>
          <h2 className="font-semibold text-slate-900">4. Retention Period</h2>
          <p className="mt-2">
            Server logs are retained by the hosting provider for typically 7 to
            30 days and then deleted automatically, unless a security incident
            requires longer retention.
          </p>
        </section>

        {/* 5. Hosting */}
        <section>
          <h2 className="font-semibold text-slate-900">5. Hosting</h2>
          <p className="mt-2">This website is hosted by:</p>
          {/*
            TODO: Replace with your actual hosting provider name and address.
            A Data Processing Agreement (DPA) with the provider is required under Art. 28 GDPR.
          */}
          <div className="mt-2 rounded-md border border-amber-200 bg-amber-50 p-3 text-xs text-amber-800">
            [HOSTING PROVIDER — add name and address. Ensure a Data Processing
            Agreement (DPA) is in place per Art. 28 GDPR.]
          </div>
          <p className="mt-2">
            A Data Processing Agreement (DPA) has been concluded with the
            hosting provider pursuant to Art. 28 GDPR.
          </p>
        </section>

        {/* 6. Fonts */}
        <section>
          <h2 className="font-semibold text-slate-900">
            6. Web Fonts (Self-Hosted)
          </h2>
          <p className="mt-2">
            This site uses Google Fonts (Fraunces, Space Grotesk). The font
            files are downloaded during the build process and served from our
            own servers. No requests are made to Google servers when you visit
            this site. No data is transmitted to Google.
          </p>
        </section>

        {/* 7. External links */}
        <section>
          <h2 className="font-semibold text-slate-900">7. External Links</h2>
          <p className="mt-2">
            This site links to external services including GitHub
            (github.com) and docs.qryvanta.org. When you follow an external
            link you leave this website. Those services process data under
            their own privacy policies. We have no control over third-party
            data processing.
          </p>
          <p className="mt-2">
            GitHub&apos;s privacy statement:{" "}
            <a
              href="https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement"
              target="_blank"
              rel="noopener noreferrer"
              className="text-emerald-700 underline-offset-2 hover:underline"
            >
              GitHub Privacy Statement
            </a>
          </p>
        </section>

        {/* 8. Your rights */}
        <section>
          <h2 className="font-semibold text-slate-900">
            8. Your Rights Under GDPR
          </h2>
          <p className="mt-2">
            You have the following rights regarding your personal data:
          </p>
          <ul className="mt-2 list-inside list-disc space-y-1 pl-2">
            <li>Right of access (Art. 15 GDPR)</li>
            <li>Right to rectification (Art. 16 GDPR)</li>
            <li>Right to erasure (Art. 17 GDPR)</li>
            <li>Right to restriction of processing (Art. 18 GDPR)</li>
            <li>Right to data portability (Art. 20 GDPR)</li>
            <li>Right to object to processing (Art. 21 GDPR)</li>
          </ul>
          <p className="mt-2">
            To exercise any of these rights, contact us at:{" "}
            <ProtectedEmailLink
              localPart={LEGAL_EMAIL_LOCAL_PART}
              domain={LEGAL_EMAIL_DOMAIN}
              tld={LEGAL_EMAIL_TLD}
              className="text-emerald-700 underline-offset-2 hover:underline"
              fallback="legal [at] qryvanta [dot] org"
            />
          </p>
        </section>

        {/* 9. Right to complain */}
        <section>
          <h2 className="font-semibold text-slate-900">
            9. Right to Lodge a Complaint
          </h2>
          <p className="mt-2">
            You have the right to lodge a complaint with a data protection
            supervisory authority. The competent authority depends on the
            federal state in which the company is registered. A list of German
            supervisory authorities is available at{" "}
            <a
              href="https://www.bfdi.bund.de/DE/Service/Anschriften/Laender/Laender-node.html"
              target="_blank"
              rel="noopener noreferrer"
              className="text-emerald-700 underline-offset-2 hover:underline"
            >
              bfdi.bund.de
            </a>
            .
          </p>
        </section>

        {/* 10. Changes */}
        <section>
          <h2 className="font-semibold text-slate-900">
            10. Changes to This Policy
          </h2>
          <p className="mt-2">
            We may update this privacy policy when the website changes or when
            legal requirements change. The current version is always available
            at{" "}
            <Link
              href="/privacy"
              className="text-emerald-700 underline-offset-2 hover:underline"
            >
              qryvanta.org/privacy
            </Link>
            .
          </p>
        </section>
      </div>

      <div className="mt-12 border-t border-emerald-100 pt-6 text-xs text-slate-400">
        <Link href="/legal-notice" className="hover:text-slate-600">
          Legal Notice
        </Link>
        <span className="mx-3">·</span>
        <Link href="/datenschutz" className="hover:text-slate-600">
          Datenschutzerklärung (Deutsch)
        </Link>
        <span className="mx-3">·</span>
        <Link href="/" className="hover:text-slate-600">
          Back to home
        </Link>
      </div>
    </div>
  );
}
