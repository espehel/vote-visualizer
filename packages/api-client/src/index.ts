import createFetchClient, {
  type Client,
  type ClientOptions,
} from "openapi-fetch";
import type { paths } from "./generated/schema";

export * from "./generated/schema";

export type ApiClient = Client<paths>;

/**
 * Typed client for the Rust API, e.g.
 * `await client.GET("/votes/{id}", { params: { path: { id } } })`.
 */
export function createApiClient(options: ClientOptions): ApiClient {
  return createFetchClient<paths>(options);
}
