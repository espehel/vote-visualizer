"use client";

import type { ImportedVote, ImportedVoteSummary } from "@repo/api-client";
import { JsonViewer } from "@repo/ui/json-viewer";
import { Modal } from "@repo/ui/modal";
import { useRef, useState, useTransition } from "react";
import { getImportedVote, type ImportedVoteResult } from "@/app/admin/actions";

// Fixed locale and time zone, so server and client render the same text.
const dateFormat = new Intl.DateTimeFormat("nb-NO", {
  dateStyle: "short",
  timeStyle: "medium",
  timeZone: "Europe/Oslo",
});

function formatDate(value: string) {
  return dateFormat.format(new Date(value));
}

export function ImportedVotes({ votes }: { votes: ImportedVoteSummary[] }) {
  const [open, setOpen] = useState(false);
  const openId = useRef<number | null>(null);
  const [result, setResult] = useState<ImportedVoteResult>();
  const [pending, startTransition] = useTransition();

  function show(id: number) {
    openId.current = id;
    setResult(undefined);
    setOpen(true);
    startTransition(async () => {
      const next = await getImportedVote(id);
      // Ignore a response for a vote that is no longer open.
      if (openId.current === id) setResult(next);
    });
  }

  function close() {
    setOpen(false);
    openId.current = null;
    setResult(undefined);
  }

  if (votes.length === 0) {
    return <p className="text-base-content/60">No votes imported yet.</p>;
  }

  return (
    <>
      <div className="overflow-x-auto rounded-box border border-base-300">
        <table className="table">
          <thead>
            <tr>
              <th>Vote ID</th>
              <th>Subject</th>
              <th>Imported</th>
            </tr>
          </thead>
          <tbody>
            {votes.map((vote) => (
              <tr
                key={vote.id}
                onClick={() => show(vote.id)}
                className="cursor-pointer hover:bg-base-200"
              >
                <td className="tabular-nums">{vote.id}</td>
                <td>
                  <button type="button" className="link link-hover text-left">
                    {vote.subject ?? (
                      <span className="text-base-content/60">No subject</span>
                    )}
                  </button>
                </td>
                <td className="whitespace-nowrap tabular-nums">
                  {formatDate(vote.imported_at)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <Modal open={open} onClose={close} className="w-11/12 max-w-5xl">
        {pending && !result && (
          <div className="flex justify-center py-12">
            <span className="loading loading-spinner loading-lg" />
          </div>
        )}
        {result && "error" in result && (
          <div role="alert" className="alert alert-error">
            {result.error}
          </div>
        )}
        {result && "vote" in result && (
          <VoteDetails key={result.vote.vote_id} vote={result.vote} />
        )}
      </Modal>
    </>
  );
}

const sources = [
  { key: "case_votes", label: "Case votes" },
  { key: "results", label: "Results" },
  { key: "proposals", label: "Proposals" },
] as const;

function VoteDetails({ vote }: { vote: ImportedVote }) {
  return (
    <div className="space-y-4">
      <h3 className="pr-8 font-bold text-lg">
        {vote.subject ?? `Vote ${vote.vote_id}`}
      </h3>
      <dl className="grid grid-cols-1 gap-x-6 gap-y-1 text-sm sm:grid-cols-[auto_1fr]">
        <dt className="text-base-content/60">Vote ID</dt>
        <dd className="tabular-nums">{vote.vote_id}</dd>
        <dt className="text-base-content/60">Case ID</dt>
        <dd className="tabular-nums">{vote.case_id}</dd>
        <dt className="text-base-content/60">Imported</dt>
        <dd className="tabular-nums">{formatDate(vote.imported_at)}</dd>
      </dl>

      <div role="tablist" className="tabs tabs-lift">
        {sources.map((source, index) => (
          <SourceTab
            key={source.key}
            label={source.label}
            body={vote[source.key]}
            defaultChecked={index === 0}
          />
        ))}
      </div>
    </div>
  );
}

function SourceTab({
  label,
  body,
  defaultChecked,
}: {
  label: string;
  body: unknown;
  defaultChecked: boolean;
}) {
  return (
    <>
      <input
        type="radio"
        name="imported_vote_source"
        className="tab"
        aria-label={label}
        defaultChecked={defaultChecked}
      />
      <div className="tab-content border-base-300 bg-base-100 p-4">
        {body == null ? (
          <p className="text-base-content/60">No snapshot stored.</p>
        ) : (
          <JsonViewer value={body} className="max-h-[60vh]" />
        )}
      </div>
    </>
  );
}
