"use server";

import type { ImportedVote, ImportReport } from "@repo/api-client";
import { refresh } from "next/cache";
import { api, errorMessage } from "./api";

export type ImportState =
  | { report: ImportReport }
  | { error: string }
  | undefined;

export async function importVote(
  _prev: ImportState,
  formData: FormData,
): Promise<ImportState> {
  const body = {
    case_id: Number(formData.get("case_id")),
    vote_id: Number(formData.get("vote_id")),
  };
  try {
    const { data, error, response } = await api.POST("/admin/imports/votes", {
      body,
    });
    if (data) {
      // Re-render the page so the imported votes list includes this vote.
      refresh();
      return { report: data };
    }
    return { error: errorMessage(error, response) };
  } catch (e) {
    return { error: e instanceof Error ? e.message : String(e) };
  }
}

export type ImportedVoteResult = { vote: ImportedVote } | { error: string };

export async function getImportedVote(id: number): Promise<ImportedVoteResult> {
  try {
    const { data, error, response } = await api.GET(
      "/admin/imports/votes/{id}",
      {
        params: { path: { id } },
      },
    );
    if (data) return { vote: data };
    return { error: errorMessage(error, response) };
  } catch (e) {
    return { error: e instanceof Error ? e.message : String(e) };
  }
}
