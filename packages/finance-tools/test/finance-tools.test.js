// Smoke tests for the JavaScript API and the wasm boundary. Numerical
// correctness is covered by the Rust property tests against QuantLib; these
// only check that the classes are wired to the right day counters.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  Actual360,
  Actual364,
  Actual36525,
  Actual365Fixed,
  Actual366,
  ActualActual,
  DayCounter,
  OneDayCounter,
  SimpleDayCounter,
  Thirty360,
  Thirty365,
} from "finance-tools";

test("every class has QuantLib's name", () => {
  const names = [
    [new Actual360(), "Actual/360"],
    [new Actual360(true), "Actual/360 (inc)"],
    [new Actual364(), "Actual/364"],
    [new Actual366(), "Actual/366"],
    [new Actual36525(), "Actual/365.25"],
    [new Actual365Fixed(), "Actual/365 (Fixed)"],
    [new Actual365Fixed(Actual365Fixed.Canadian), "Actual/365 (Fixed) Canadian Bond"],
    [new Actual365Fixed(Actual365Fixed.NoLeap), "Actual/365 (No Leap)"],
    [new ActualActual(ActualActual.ISMA), "Actual/Actual (ISMA)"],
    [new ActualActual(ActualActual.ISDA), "Actual/Actual (ISDA)"],
    [new ActualActual(ActualActual.AFB), "Actual/Actual (AFB)"],
    [new Thirty360(Thirty360.USA), "30/360 (US)"],
    [new Thirty360(Thirty360.BondBasis), "30/360 (Bond Basis)"],
    [new Thirty360(Thirty360.European), "30E/360 (Eurobond Basis)"],
    [new Thirty360(Thirty360.Italian), "30/360 (Italian)"],
    [new Thirty360(Thirty360.ISDA), "30E/360 (ISDA)"],
    [new Thirty360(Thirty360.NASD), "30/360 (NASD)"],
    [new Thirty365(), "30/365"],
    [new OneDayCounter(), "1/1"],
    [new SimpleDayCounter(), "Simple"],
  ];
  for (const [dc, name] of names) {
    assert.ok(dc instanceof DayCounter);
    assert.equal(dc.name(), name);
    assert.equal(String(dc), name);
  }
});

test("computes ISDA paper values", () => {
  const close = (a, b) => assert.ok(Math.abs(a - b) < 1e-11, `${a} vs ${b}`);
  const [d1, d2] = ["2003-11-01", "2004-05-01"];
  close(new ActualActual(ActualActual.ISDA).yearFraction(d1, d2), 0.497724380567);
  close(new ActualActual(ActualActual.AFB).yearFraction(d1, d2), 0.497267759563);
  close(new ActualActual(ActualActual.ISMA).yearFraction(d1, d2, d1, d2), 0.5);
});

test("day counts and the termination date", () => {
  assert.equal(new Thirty360(Thirty360.BondBasis).dayCount("2024-01-31", "2024-03-31"), 60);
  const [d1, maturity] = ["2023-08-31", "2024-02-29"];
  assert.equal(new Thirty360(Thirty360.ISDA).dayCount(d1, maturity), 180);
  assert.equal(new Thirty360(Thirty360.ISDA, maturity).dayCount(d1, maturity), 179);
});

test("aliases are equal", () => {
  assert.ok(new Thirty360(Thirty360.German).equals(new Thirty360(Thirty360.ISDA)));
  assert.ok(!new Thirty360(Thirty360.USA).equals(new Thirty360(Thirty360.NASD)));
});

test("rejects invalid input", () => {
  assert.throws(() => new Actual360().yearFraction("2023-02-29", "2024-01-01"), /invalid start date/);
  assert.throws(() => new Actual360().dayCount("2024-01-01", "not a date"), /invalid end date/);
  assert.throws(() => new Thirty360("Bogus"), /unknown Thirty360 convention/);
  assert.throws(() => new Thirty360(Thirty360.ISDA, "2024-13-01"), /invalid termination date/);
  const canadian = new Actual365Fixed(Actual365Fixed.Canadian);
  assert.throws(() => canadian.yearFraction("2024-01-01", "2024-04-01"), /reference period/);
  assert.throws(
    () => canadian.yearFraction("2024-01-01", "2024-04-01", "2024-01-01"),
    /both reference period dates/,
  );
});
