import type { Metadata } from "next";
import Link from "next/link";

import { ProtectedEmailLink } from "@/components/legal/protected-email-link";

const LEGAL_EMAIL_LOCAL_PART = [108, 101, 103, 97, 108] as const;
const LEGAL_EMAIL_DOMAIN = [113, 114, 121, 118, 97, 110, 116, 97] as const;
const LEGAL_EMAIL_TLD = [111, 114, 103] as const;

export const metadata: Metadata = {
  title: "Datenschutzerklärung",
  description:
    "Informationen zur Verarbeitung personenbezogener Daten auf qryvanta.org gemäß DSGVO.",
  robots: { index: false, follow: false, nocache: true },
  alternates: { canonical: "https://qryvanta.org/datenschutz" },
};

export default function DatenschutzPage() {
  return (
    <div className="mx-auto max-w-2xl px-6 py-16">
      <nav className="mb-10 text-xs text-slate-400">
        <Link href="/" className="hover:text-slate-600">
          qryvanta.org
        </Link>
        <span className="mx-2">/</span>
        <span className="text-slate-600">Datenschutzerklärung</span>
      </nav>

      <h1 className="text-2xl font-semibold text-slate-900">
        Datenschutzerklärung
      </h1>
      <p className="mt-1 text-xs text-slate-400">Stand: März 2026</p>

      <div className="mt-8 space-y-8 text-sm text-slate-600">

        {/* 1. Verantwortlicher */}
        <section>
          <h2 className="font-semibold text-slate-900">1. Verantwortlicher</h2>
          <p className="mt-2">
            Verantwortlicher im Sinne der DSGVO für diese Website ist:
          </p>
          <div className="mt-2 space-y-0.5 text-slate-700">
            <p className="font-medium">QRYVANTA UG (haftungsbeschränkt)</p>
            {/* TODO: Replace with actual address */}
            <p>[STRASSE UND HAUSNUMMER]</p>
            <p>[PLZ] [STADT]</p>
            <p>Deutschland</p>
            <p className="mt-2">
              E-Mail:{" "}
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

        {/* 2. Welche Daten werden verarbeitet */}
        <section>
          <h2 className="font-semibold text-slate-900">
            2. Welche Daten werden verarbeitet
          </h2>
          <p className="mt-2">
            Diese Website erhebt keine personenbezogenen Daten durch Formulare,
            Nutzerkonten oder Tracking-Technologien. Es werden keine Cookies
            gesetzt.
          </p>
          <p className="mt-2">
            Beim Abruf dieser Website werden durch den Hosting-Anbieter
            automatisch technische Zugriffsdaten in Server-Logfiles erfasst.
            Dazu gehören:
          </p>
          <ul className="mt-2 list-inside list-disc space-y-1 pl-2">
            <li>IP-Adresse des anfragenden Geräts (anonymisiert oder vollständig, je nach Konfiguration)</li>
            <li>Datum und Uhrzeit des Zugriffs</li>
            <li>Aufgerufene URL und HTTP-Statuscode</li>
            <li>Übertragene Datenmenge</li>
            <li>Referrer-URL (sofern übermittelt)</li>
            <li>Browser-Typ und Betriebssystem</li>
          </ul>
          <p className="mt-2">
            Diese Daten werden nicht mit anderen Datenquellen zusammengeführt
            und dienen ausschließlich dem sicheren Betrieb der Website sowie der
            Fehleranalyse.
          </p>
        </section>

        {/* 3. Rechtsgrundlage */}
        <section>
          <h2 className="font-semibold text-slate-900">3. Rechtsgrundlage</h2>
          <p className="mt-2">
            Die Verarbeitung der Server-Logfiles erfolgt auf Grundlage von Art.
            6 Abs. 1 lit. f DSGVO (berechtigtes Interesse). Das berechtigte
            Interesse liegt im sicheren und fehlerfreien Betrieb der Website.
          </p>
        </section>

        {/* 4. Speicherdauer */}
        <section>
          <h2 className="font-semibold text-slate-900">4. Speicherdauer</h2>
          <p className="mt-2">
            Server-Logfiles werden durch den Hosting-Anbieter für in der Regel
            7 bis 30 Tage gespeichert und anschließend automatisch gelöscht,
            sofern kein Sicherheitsvorfall eine längere Aufbewahrung
            erfordert.
          </p>
        </section>

        {/* 5. Hosting */}
        <section>
          <h2 className="font-semibold text-slate-900">5. Hosting</h2>
          <p className="mt-2">
            Diese Website wird bei folgendem Anbieter gehostet:
          </p>
          {/*
            TODO: Replace with your actual hosting provider name and address.
            Examples: Vercel Inc., Hetzner Online GmbH, etc.
            Include their data processing agreement (DPA) reference if available.
          */}
          <div className="mt-2 rounded-md border border-amber-200 bg-amber-50 p-3 text-xs text-amber-800">
            [HOSTING-ANBIETER — Name und Adresse eintragen. Auftragsverarbeitungsvertrag
            (AVV) mit dem Anbieter gemäß Art. 28 DSGVO abschließen.]
          </div>
          <p className="mt-2">
            Mit dem Hosting-Anbieter wurde ein Auftragsverarbeitungsvertrag
            (AVV) gemäß Art. 28 DSGVO abgeschlossen.
          </p>
        </section>

        {/* 6. Schriftarten */}
        <section>
          <h2 className="font-semibold text-slate-900">
            6. Webfonts (selbst gehostet)
          </h2>
          <p className="mt-2">
            Diese Website verwendet Schriftarten (Google Fonts: Fraunces,
            Space Grotesk). Die Schriftdateien werden beim Build-Prozess
            heruntergeladen und auf unseren eigenen Servern ausgeliefert. Beim
            Besuch dieser Website werden keine Anfragen an Google-Server
            gestellt. Es werden keine Daten an Google übermittelt.
          </p>
        </section>

        {/* 7. Externe Links */}
        <section>
          <h2 className="font-semibold text-slate-900">7. Externe Links</h2>
          <p className="mt-2">
            Diese Website enthält Links zu externen Diensten, darunter GitHub
            (github.com) und die Dokumentations-Domain docs.qryvanta.org.
            Wenn Sie auf externe Links klicken, verlassen Sie diese Website.
            Für die Datenverarbeitung durch diese Dienste gelten deren eigene
            Datenschutzerklärungen. Wir haben keinen Einfluss auf die
            Datenverarbeitung durch Dritte.
          </p>
          <p className="mt-2">
            Informationen zur Datenverarbeitung durch GitHub finden Sie unter:{" "}
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

        {/* 8. Ihre Rechte */}
        <section>
          <h2 className="font-semibold text-slate-900">
            8. Ihre Rechte nach der DSGVO
          </h2>
          <p className="mt-2">
            Sie haben gegenüber dem Verantwortlichen folgende Rechte bezüglich
            Ihrer personenbezogenen Daten:
          </p>
          <ul className="mt-2 list-inside list-disc space-y-1 pl-2">
            <li>Recht auf Auskunft (Art. 15 DSGVO)</li>
            <li>Recht auf Berichtigung (Art. 16 DSGVO)</li>
            <li>Recht auf Löschung (Art. 17 DSGVO)</li>
            <li>Recht auf Einschränkung der Verarbeitung (Art. 18 DSGVO)</li>
            <li>Recht auf Datenübertragbarkeit (Art. 20 DSGVO)</li>
            <li>Recht auf Widerspruch gegen die Verarbeitung (Art. 21 DSGVO)</li>
          </ul>
          <p className="mt-2">
            Zur Geltendmachung Ihrer Rechte wenden Sie sich an:{" "}
            <ProtectedEmailLink
              localPart={LEGAL_EMAIL_LOCAL_PART}
              domain={LEGAL_EMAIL_DOMAIN}
              tld={LEGAL_EMAIL_TLD}
              className="text-emerald-700 underline-offset-2 hover:underline"
              fallback="legal [at] qryvanta [dot] org"
            />
          </p>
        </section>

        {/* 9. Beschwerderecht */}
        <section>
          <h2 className="font-semibold text-slate-900">
            9. Beschwerderecht bei einer Aufsichtsbehörde
          </h2>
          <p className="mt-2">
            Sie haben das Recht, sich bei einer Datenschutz-Aufsichtsbehörde
            über die Verarbeitung Ihrer personenbezogenen Daten zu beschweren.
            Die zuständige Aufsichtsbehörde richtet sich nach dem Bundesland
            des Unternehmensitzes.
          </p>
          <p className="mt-2">
            Eine Liste der deutschen Datenschutzbehörden finden Sie unter:{" "}
            <a
              href="https://www.bfdi.bund.de/DE/Service/Anschriften/Laender/Laender-node.html"
              target="_blank"
              rel="noopener noreferrer"
              className="text-emerald-700 underline-offset-2 hover:underline"
            >
              bfdi.bund.de
            </a>
          </p>
        </section>

        {/* 10. Aktualität */}
        <section>
          <h2 className="font-semibold text-slate-900">
            10. Aktualität dieser Erklärung
          </h2>
          <p className="mt-2">
            Wir behalten uns vor, diese Datenschutzerklärung bei Änderungen der
            Website oder bei geänderten rechtlichen Anforderungen anzupassen.
            Die aktuelle Version ist stets unter{" "}
            <Link
              href="/datenschutz"
              className="text-emerald-700 underline-offset-2 hover:underline"
            >
              qryvanta.org/datenschutz
            </Link>{" "}
            abrufbar.
          </p>
        </section>
      </div>

      <div className="mt-12 border-t border-emerald-100 pt-6 text-xs text-slate-400">
        <Link href="/impressum" className="hover:text-slate-600">
          Impressum
        </Link>
        <span className="mx-3">·</span>
        <Link href="/privacy" className="hover:text-slate-600">
          Privacy Policy (English)
        </Link>
        <span className="mx-3">·</span>
        <Link href="/" className="hover:text-slate-600">
          Zurück zur Startseite
        </Link>
      </div>
    </div>
  );
}
