/**
 * Day count conventions for JavaScript.
 *
 * This module is the package's public API. The `wasm-bindgen` output in
 * `./wasm/` is an implementation detail: it may change shape between releases
 * without a major version bump, so nothing from it is exported directly
 * except through the declarations below.
 *
 * @packageDocumentation
 */

import * as wasm from "./wasm/finance_tools.js";

export { DayCountConvention } from "./wasm/finance_tools.js";

/**
 * An ISO 8601 calendar date, `YYYY-MM-DD`.
 *
 * JavaScript `Date` objects are deliberately not accepted: they are instants,
 * and converting one to a calendar date depends on a time zone.
 */
export type IsoDate = string;

/**
 * Computes the year fraction between two dates under a day count convention.
 *
 * Dates do not have to be ordered; a reversed pair yields what the
 * convention's formula produces (negative for the actual-day conventions).
 *
 * @throws {Error} If either date is not a valid ISO 8601 calendar date.
 */
export function yearFraction(
  convention: wasm.DayCountConvention,
  start: IsoDate,
  end: IsoDate,
): number {
  return wasm.yearFraction(convention, start, end);
}

/**
 * Returns the conventional market name of a day count convention, for
 * example `"Act/365F"`.
 */
export function conventionName(convention: wasm.DayCountConvention): string {
  return wasm.conventionName(convention);
}
