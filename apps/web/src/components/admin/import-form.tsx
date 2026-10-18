"use client";

import { useActionState } from "react";
import { importVote } from "@/app/admin/actions";

export function ImportForm() {
  const [state, action, pending] = useActionState(importVote, undefined);

  return (
    <div className="space-y-4">
      <form action={action} className="space-y-4">
        <label className="floating-label">
          <span>Case ID</span>
          <input
            name="case_id"
            type="number"
            required
            defaultValue={83657}
            className="input w-full"
          />
        </label>
        <label className="floating-label">
          <span>Vote ID</span>
          <input
            name="vote_id"
            type="number"
            required
            defaultValue={16897}
            className="input w-full"
          />
        </label>
        <button type="submit" disabled={pending} className="btn btn-primary">
          {pending && <span className="loading loading-spinner" />}
          Import
        </button>
      </form>

      {state && "report" in state && (
        <div role="alert" className="alert alert-success">
          Imported vote {state.report.vote_id} (case {state.report.case_id}):{" "}
          {state.report.ballot_count} ballots, {state.report.proposal_count}{" "}
          proposals.
        </div>
      )}
      {state && "error" in state && (
        <div role="alert" className="alert alert-error">
          {state.error}
        </div>
      )}
    </div>
  );
}
