import type { Metadata } from "next";
import Link from "next/link";

import { ProtectedEmailLink } from "@/components/legal/protected-email-link";

const LEGAL_EMAIL_LOCAL_PART = [108, 101, 103, 97, 108] as const;
const LEGAL_EMAIL_DOMAIN = [113, 114, 121, 118, 97, 110, 116, 97] as const;
const LEGAL_EMAIL_TLD = [111, 114, 103] as const;

export const metadata: Metadata = {
  title: "Impressum",
  description: "Gesetzliche Anbieterkennzeichnung gemäß § 5 TMG für qryvanta.org.",
  robots: { index: false, follow: false, nocache: true },
  alternates: { canonical: "https://qryvanta.org/impressum" },
};

export default function ImpressumPage() {
  return (
    <div className="mx-auto max-w-2xl px-6 py-16">
      <nav className="mb-10 text-xs text-slate-400">
        <Link href="/" className="hover:text-slate-600">
          qryvanta.org
        </Link>
        <span className="mx-2">/</span>
        <span className="text-slate-600">Impressum</span>
      </nav>

      <h1 className="text-2xl font-semibold text-slate-900">Impressum</h1>
      <p className="mt-1 text-xs text-slate-400">
        Angaben gemäß § 5 TMG
      </p>

      <section className="mt-8 space-y-1 text-sm text-slate-700">
        <p className="font-semibold">QRYVANTA UG (haftungsbeschränkt)</p>
        {/* TODO: Replace the three lines below with your registered address */}
        <p>[STRASSE UND HAUSNUMMER]</p>
        <p>[PLZ] [STADT]</p>
        <p>Deutschland</p>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">Geschäftsführer</p>
        {/* TODO: Replace with the managing director's full name */}
        <p className="mt-1">[VORNAME NACHNAME]</p>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">Registereintrag</p>
        <div className="mt-1 space-y-0.5">
          {/* TODO: Replace with the correct Amtsgericht city and HRB number */}
          <p>Eingetragen im Handelsregister.</p>
          <p>Registergericht: Amtsgericht [STADT]</p>
          <p>Registernummer: HRB [NUMMER]</p>
        </div>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">Kontakt</p>
        <p className="mt-1">
          E-Mail:{" "}
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
        <p className="font-semibold text-slate-900">
          Umsatzsteuer-Identifikationsnummer
        </p>
        {/*
          TODO: Add your VAT ID once issued by the Bundeszentralamt für Steuern.
          If not yet assigned, replace the line below with:
          "Umsatzsteuer-Identifikationsnummer noch nicht zugeteilt."
        */}
        <p className="mt-1">
          Gemäß § 27a Umsatzsteuergesetz: [DE-NUMMER oder &quot;noch nicht zugeteilt&quot;]
        </p>
      </section>

      <section className="mt-6 text-sm text-slate-700">
        <p className="font-semibold text-slate-900">
          Verantwortlich für den Inhalt nach § 55 Abs. 2 RStV
        </p>
        {/* TODO: Full name and address of the person responsible for editorial content */}
        <div className="mt-1 space-y-0.5">
          <p>[VORNAME NACHNAME]</p>
          <p>[STRASSE UND HAUSNUMMER]</p>
          <p>[PLZ] [STADT]</p>
        </div>
      </section>

      <section className="mt-10 space-y-4 text-sm text-slate-600">
        <div>
          <p className="font-semibold text-slate-900">Streitschlichtung</p>
          <p className="mt-1">
            Die Europäische Kommission stellt eine Plattform zur
            Online-Streitbeilegung (OS) bereit:{" "}
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
            Wir sind nicht bereit oder verpflichtet, an
            Streitbeilegungsverfahren vor einer Verbraucherschlichtungsstelle
            teilzunehmen.
          </p>
        </div>

        <div>
          <p className="font-semibold text-slate-900">Haftung für Inhalte</p>
          <p className="mt-1">
            Als Diensteanbieter sind wir gemäß § 7 Abs. 1 TMG für eigene
            Inhalte auf diesen Seiten nach den allgemeinen Gesetzen
            verantwortlich. Nach §§ 8 bis 10 TMG sind wir als Diensteanbieter
            nicht verpflichtet, übermittelte oder gespeicherte fremde
            Informationen zu überwachen oder nach Umständen zu forschen, die
            auf eine rechtswidrige Tätigkeit hinweisen. Verpflichtungen zur
            Entfernung oder Sperrung der Nutzung von Informationen nach den
            allgemeinen Gesetzen bleiben hiervon unberührt. Eine Haftung ist
            jedoch erst ab dem Zeitpunkt der Kenntnis einer konkreten
            Rechtsverletzung möglich.
          </p>
        </div>

        <div>
          <p className="font-semibold text-slate-900">Haftung für Links</p>
          <p className="mt-1">
            Unser Angebot enthält Links zu externen Websites Dritter, auf deren
            Inhalte wir keinen Einfluss haben. Für die Inhalte der verlinkten
            Seiten ist stets der jeweilige Anbieter oder Betreiber der Seiten
            verantwortlich. Die verlinkten Seiten wurden zum Zeitpunkt der
            Verlinkung auf mögliche Rechtsverstöße überprüft. Rechtswidrige
            Inhalte waren zum Zeitpunkt der Verlinkung nicht erkennbar. Bei
            Bekanntwerden von Rechtsverletzungen werden wir derartige Links
            umgehend entfernen.
          </p>
        </div>

        <div>
          <p className="font-semibold text-slate-900">Urheberrecht</p>
          <p className="mt-1">
            Die durch die Seitenbetreiber erstellten Inhalte und Werke auf
            diesen Seiten unterliegen dem deutschen Urheberrecht. Der
            Quellcode des Qryvanta-Projekts steht unter der jeweiligen
            Open-Source-Lizenz des Projekts zur Verfügung. Soweit die Inhalte
            auf dieser Seite nicht vom Betreiber erstellt wurden, werden die
            Urheberrechte Dritter beachtet.
          </p>
        </div>
      </section>

      <div className="mt-12 border-t border-emerald-100 pt-6 text-xs text-slate-400">
        <Link href="/datenschutz" className="hover:text-slate-600">
          Datenschutzerklärung
        </Link>
        <span className="mx-3">·</span>
        <Link href="/legal-notice" className="hover:text-slate-600">
          Legal Notice (English)
        </Link>
        <span className="mx-3">·</span>
        <Link href="/" className="hover:text-slate-600">
          Zurück zur Startseite
        </Link>
      </div>
    </div>
  );
}
