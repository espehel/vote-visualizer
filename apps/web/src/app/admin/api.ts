import { createApiClient, type ErrorBody } from "@repo/api-client";

export const api = createApiClient({
  baseUrl: process.env.API_URL ?? "http://127.0.0.1:8080",
});

/**
 * Extractor rejections are plain text and handler errors are `ErrorBody`.
 * Unmatched routes have an empty body, so fall back to the status.
 */
export function errorMessage(
  error: ErrorBody | string | undefined,
  response: Response,
): string {
  if (typeof error === "string" && error) return error;
  if (typeof error === "object" && error.error) return error.error;
  return `${response.status} ${response.statusText}`.trim();
}
