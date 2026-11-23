// C ABI over QuantLib's day counters. Every entry point catches exceptions and
// reports them through a status code, since unwinding across `extern "C"` is
// undefined behaviour.

#include <ql/time/daycounters/all.hpp>

#include <cstdint>
#include <cstring>
#include <exception>
#include <string>

using namespace QuantLib;

extern "C" {

// Must match `ffi::Spec` in src/lib.rs.
struct ql_spec {
    int32_t kind;
    int32_t convention;
    int32_t include_last_day;
    int32_t has_termination_date;
    int32_t termination_year, termination_month, termination_day;
};

struct ql_date {
    int32_t year, month, day;
};

}  // extern "C"

namespace {

Date to_date(const ql_date& d) {
    return Date(static_cast<Day>(d.day), static_cast<Month>(d.month), static_cast<Year>(d.year));
}

DayCounter make(const ql_spec& s) {
    const bool inc = s.include_last_day != 0;
    switch (s.kind) {
      case 0: return Actual360(inc);
      case 1: return Actual364();
      case 2: return Actual366(inc);
      case 3: return Actual36525(inc);
      case 4: return Actual365Fixed(static_cast<Actual365Fixed::Convention>(s.convention));
      case 5: return ActualActual(static_cast<ActualActual::Convention>(s.convention));
      case 6: {
        Date termination;
        if (s.has_termination_date != 0) {
            termination = to_date({s.termination_year, s.termination_month, s.termination_day});
        }
        return Thirty360(static_cast<Thirty360::Convention>(s.convention), termination);
      }
      case 7: return Thirty365();
      case 8: return OneDayCounter();
      case 9: return SimpleDayCounter();
      default: QL_FAIL("unknown day counter kind " << s.kind);
    }
}

int fail(const char* message, char* err, size_t err_len) {
    if (err_len > 0) {
        std::strncpy(err, message, err_len - 1);
        err[err_len - 1] = '\0';
    }
    return 1;
}

}  // namespace

extern "C" {

int ql_name(const ql_spec* spec, char* out, size_t out_len) {
    try {
        std::string name = make(*spec).name();
        std::strncpy(out, name.c_str(), out_len - 1);
        out[out_len - 1] = '\0';
        return 0;
    } catch (const std::exception& e) {
        return fail(e.what(), out, out_len);
    }
}

int ql_day_count(const ql_spec* spec, const ql_date* d1, const ql_date* d2, int64_t* out,
                 char* err, size_t err_len) {
    try {
        *out = make(*spec).dayCount(to_date(*d1), to_date(*d2));
        return 0;
    } catch (const std::exception& e) {
        return fail(e.what(), err, err_len);
    }
}

// `ref_start`/`ref_end` may be null, meaning QuantLib's default `Date()`.
int ql_year_fraction(const ql_spec* spec, const ql_date* d1, const ql_date* d2,
                     const ql_date* ref_start, const ql_date* ref_end, double* out, char* err,
                     size_t err_len) {
    try {
        Date rs = ref_start != nullptr ? to_date(*ref_start) : Date();
        Date re = ref_end != nullptr ? to_date(*ref_end) : Date();
        *out = make(*spec).yearFraction(to_date(*d1), to_date(*d2), rs, re);
        return 0;
    } catch (const std::exception& e) {
        return fail(e.what(), err, err_len);
    }
}

}  // extern "C"
