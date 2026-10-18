import { Suspense } from "react";
import { ImportForm } from "@/components/admin/import-form";
import { ImportedVotes } from "@/components/admin/imported-votes";
import { api, errorMessage } from "./api";

export default function AdminPage() {
  return (
    <main className="mx-auto w-full max-w-4xl space-y-10 p-8">
      <section className="max-w-md space-y-6">
        <h1 className="font-bold text-2xl">Import vote</h1>
        <ImportForm />
      </section>

      <section className="space-y-4">
        <h2 className="font-bold text-xl">Imported votes</h2>
        <Suspense
          fallback={<div className="skeleton h-32 w-full rounded-box" />}
        >
          <ImportedVotesList />
        </Suspense>
      </section>
    </main>
  );
}

async function ImportedVotesList() {
  try {
    const { data, error, response } = await api.GET("/admin/imports/votes");
    if (data) return <ImportedVotes votes={data} />;
    return <ListError message={errorMessage(error, response)} />;
  } catch (e) {
    return <ListError message={e instanceof Error ? e.message : String(e)} />;
  }
}

function ListError({ message }: { message: string }) {
  return (
    <div role="alert" className="alert alert-error">
      Could not load imported votes: {message}
    </div>
  );
}
