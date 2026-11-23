// Smoke tests for the generated bindings. Numerical correctness is covered by
// the Rust test suite (including parity with RustQuant); these tests only
// check that the wasm boundary behaves.
import assert from "node:assert/strict";
import { test } from "node:test";

import { DayCountConvention, conventionName, yearFraction } from "finance-tools";

test("computes a year fraction from ISO dates", () => {
  const yf = yearFraction(DayCountConvention.ActualActualIsda, "2003-11-01", "2004-05-01");
  assert.ok(Math.abs(yf - 0.497724380567) < 1e-12, `got ${yf}`);
});

test("exposes every convention with a market name", () => {
  const names = Object.keys(DayCountConvention)
    .filter((key) => Number.isNaN(Number(key)))
    .map((key) => conventionName(DayCountConvention[key]));
  assert.equal(names.length, 18);
  assert.ok(names.includes("Act/365F"));
  assert.ok(names.includes("30E+/360"));
});

test("throws on invalid dates", () => {
  assert.throws(
    () => yearFraction(DayCountConvention.Actual360, "2023-02-29", "2024-01-01"),
    /invalid start date/,
  );
  assert.throws(
    () => yearFraction(DayCountConvention.Actual360, "2024-01-01", "not a date"),
    /invalid end date/,
  );
});
