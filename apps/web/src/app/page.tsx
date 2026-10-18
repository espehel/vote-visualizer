import Image from "next/image";
import Link from "next/link";
import icon from "./icon.png";

const STORTINGET_DATA_URL = "https://data.stortinget.no/";

export default function Home() {
  return (
    <>
      <header className="navbar border-base-300 border-b px-4 sm:px-8">
        <div className="navbar-start">
          <Link
            href="/"
            className="flex items-center gap-2 font-semibold text-lg tracking-tight"
          >
            <Image src={icon} alt="" width={28} height={28} priority />
            Vote Visualizer
          </Link>
        </div>
        <nav className="navbar-end gap-2">
          <a href="#how-it-works" className="btn btn-ghost btn-sm">
            How it works
          </a>
          <a
            href={STORTINGET_DATA_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="btn btn-ghost btn-sm"
          >
            Open data
          </a>
        </nav>
      </header>

      <main className="flex flex-1 flex-col">
        <section className="hero flex-1 py-20 sm:py-32">
          <div className="hero-content max-w-3xl text-center">
            <div>
              <h1 className="font-bold text-4xl tracking-tight sm:text-6xl">
                See how the Storting really votes
              </h1>
              <p className="mx-auto mt-6 max-w-xl text-base-content/70 text-lg leading-8">
                Follow every vote in the Norwegian parliament. Compare parties,
                track representatives, and see where they stand, straight from
                Stortinget&apos;s open data.
              </p>
              <a href="#how-it-works" className="btn btn-primary mt-10">
                How it works
              </a>
            </div>
          </div>
        </section>

        <section id="how-it-works" className="bg-base-200 px-4 py-16 sm:px-8">
          <div className="mx-auto grid max-w-5xl gap-10 sm:grid-cols-3">
            <div>
              <h2 className="font-semibold text-lg">Every vote</h2>
              <p className="mt-2 text-base-content/70">
                Votes are imported directly from Stortinget and kept up to date.
              </p>
            </div>
            <div>
              <h2 className="font-semibold text-lg">Party lines</h2>
              <p className="mt-2 text-base-content/70">
                See how each party voted and who broke ranks.
              </p>
            </div>
            <div>
              <h2 className="font-semibold text-lg">Your representatives</h2>
              <p className="mt-2 text-base-content/70">
                Look up a representative and follow their voting record.
              </p>
            </div>
          </div>
        </section>
      </main>
    </>
  );
}
