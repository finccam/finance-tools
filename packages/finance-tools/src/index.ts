/**
 * QuantLib's day counters for JavaScript.
 *
 * The classes mirror QuantLib (and its Python bindings): construct a day
 * counter such as `new Thirty360(Thirty360.BondBasis)`, then call `name()`,
 * `dayCount(d1, d2)` or `yearFraction(d1, d2)`. Results match QuantLib; the
 * underlying Rust implementation is property-tested against it.
 *
 * The `wasm-bindgen` glue in `./wasm/` is an implementation detail and may
 * change shape between releases without a major version bump.
 *
 * @packageDocumentation
 */

import * as wasm from "./wasm/finance_tools.js";

/**
 * An ISO 8601 calendar date, `YYYY-MM-DD`.
 *
 * JavaScript `Date` objects are deliberately not accepted: they are instants,
 * and converting one to a calendar date depends on a time zone.
 */
export type IsoDate = string;

/**
 * A day count convention (`QuantLib::DayCounter`). Use one of the subclasses.
 */
export abstract class DayCounter {
  readonly #kind: string;
  readonly #convention: string | undefined;
  readonly #includeLastDay: boolean;
  readonly #terminationDate: IsoDate | undefined;
  readonly #name: string;

  protected constructor(
    kind: string,
    convention?: string,
    includeLastDay = false,
    terminationDate?: IsoDate,
  ) {
    this.#kind = kind;
    this.#convention = convention;
    this.#includeLastDay = includeLastDay;
    this.#terminationDate = terminationDate;
    // Also validates the convention and termination date.
    this.#name = wasm.dayCounterName(kind, convention, includeLastDay, terminationDate);
  }

  /** QuantLib's name for this day counter, e.g. `"Actual/365 (Fixed)"`. */
  name(): string {
    return this.#name;
  }

  /**
   * The number of days between two dates according to this convention.
   *
   * @throws {Error} If a date is not a valid ISO 8601 calendar date.
   */
  dayCount(d1: IsoDate, d2: IsoDate): number {
    return wasm.dayCount(
      this.#kind,
      this.#convention,
      this.#includeLastDay,
      this.#terminationDate,
      d1,
      d2,
    );
  }

  /**
   * The year fraction between two dates.
   *
   * The reference (coupon) period is only used by `ActualActual.ISMA` (and
   * `ActualActual.Bond`), and is required by `Actual365Fixed.Canadian`. Give
   * both of its dates or neither.
   *
   * @throws {Error} If a date is invalid, or the reference period is missing
   *   or does not fit the dates.
   */
  yearFraction(
    d1: IsoDate,
    d2: IsoDate,
    refPeriodStart?: IsoDate,
    refPeriodEnd?: IsoDate,
  ): number {
    return wasm.yearFraction(
      this.#kind,
      this.#convention,
      this.#includeLastDay,
      this.#terminationDate,
      d1,
      d2,
      refPeriodStart,
      refPeriodEnd,
    );
  }

  /** Day counters are equal if their names are, as in QuantLib. */
  equals(other: DayCounter): boolean {
    return this.#name === other.name();
  }

  toString(): string {
    return this.#name;
  }
}

/** Actual/360: actual days over 360. */
export class Actual360 extends DayCounter {
  /** @param includeLastDay Count the end date as a full day. */
  constructor(includeLastDay = false) {
    super("Actual360", undefined, includeLastDay);
  }
}

/** Actual/364: actual days over 364. */
export class Actual364 extends DayCounter {
  constructor() {
    super("Actual364");
  }
}

/** Actual/366: actual days over 366. */
export class Actual366 extends DayCounter {
  /** @param includeLastDay Count the end date as a full day. */
  constructor(includeLastDay = false) {
    super("Actual366", undefined, includeLastDay);
  }
}

/** Actual/365.25: actual days over 365.25. */
export class Actual36525 extends DayCounter {
  /** @param includeLastDay Count the end date as a full day. */
  constructor(includeLastDay = false) {
    super("Actual36525", undefined, includeLastDay);
  }
}

/** Actual/365 (Fixed) and its variants. */
export class Actual365Fixed extends DayCounter {
  /** Actual days over 365. */
  static readonly Standard = "Standard";
  /** Canadian bond convention; needs the coupon period as reference period. */
  static readonly Canadian = "Canadian";
  /** 29 February is not counted. */
  static readonly NoLeap = "NoLeap";

  constructor(convention: Actual365Fixed.Convention = Actual365Fixed.Standard) {
    super("Actual365Fixed", convention);
  }
}

export declare namespace Actual365Fixed {
  /** A convention of {@link Actual365Fixed}. */
  type Convention = "Standard" | "Canadian" | "NoLeap";
}

/** Actual/Actual and its variants. Aliases behave identically. */
export class ActualActual extends DayCounter {
  /** ICMA: actual days over the actual days of the reference period. */
  static readonly ISMA = "ISMA";
  /** Alias of {@link ActualActual.ISMA}. */
  static readonly Bond = "Bond";
  /** ISDA: days in leap years over 366, other days over 365. */
  static readonly ISDA = "ISDA";
  /** Alias of {@link ActualActual.ISDA}. */
  static readonly Historical = "Historical";
  /** Alias of {@link ActualActual.ISDA}. */
  static readonly Actual365 = "Actual365";
  /** AFB: whole years back from the end, then the stub over 365 or 366. */
  static readonly AFB = "AFB";
  /** Alias of {@link ActualActual.AFB}. */
  static readonly Euro = "Euro";

  constructor(convention: ActualActual.Convention) {
    super("ActualActual", convention);
  }
}

export declare namespace ActualActual {
  /** A convention of {@link ActualActual}. */
  type Convention = "ISMA" | "Bond" | "ISDA" | "Historical" | "Actual365" | "AFB" | "Euro";
}

/** 30/360 and its variants. Aliases behave identically. */
export class Thirty360 extends DayCounter {
  /** 30/360 US, with end-of-February rules. */
  static readonly USA = "USA";
  /** Bond basis. */
  static readonly BondBasis = "BondBasis";
  /** 30E/360. */
  static readonly European = "European";
  /** Alias of {@link Thirty360.European}. */
  static readonly EurobondBasis = "EurobondBasis";
  /** Italian. */
  static readonly Italian = "Italian";
  /** Alias of {@link Thirty360.ISDA}. */
  static readonly German = "German";
  /** Alias of {@link Thirty360.BondBasis}. */
  static readonly ISMA = "ISMA";
  /** 30E/360 ISDA; uses the termination date. */
  static readonly ISDA = "ISDA";
  /** NASD. */
  static readonly NASD = "NASD";

  /**
   * @param terminationDate Only used by `ISDA` and `German`: an end date on
   *   the last day of February keeps its day count if it is this date.
   */
  constructor(convention: Thirty360.Convention, terminationDate?: IsoDate) {
    super("Thirty360", convention, false, terminationDate);
  }
}

export declare namespace Thirty360 {
  /** A convention of {@link Thirty360}. */
  type Convention =
    | "USA"
    | "BondBasis"
    | "European"
    | "EurobondBasis"
    | "Italian"
    | "German"
    | "ISMA"
    | "ISDA"
    | "NASD";
}

/** 30/365. */
export class Thirty365 extends DayCounter {
  constructor() {
    super("Thirty365");
  }
}

/** 1/1: the year fraction is always 1 (−1 for reversed dates). */
export class OneDayCounter extends DayCounter {
  constructor() {
    super("OneDayCounter");
  }
}

/** Whole months as twelfths of a year, otherwise 30/360 bond basis. */
export class SimpleDayCounter extends DayCounter {
  constructor() {
    super("SimpleDayCounter");
  }
}
