// CaDiCaL C API Extension For External Propagators (Christoph Jabs)

#include "cipasirup.h"

#include <cassert>

#include "cadical.hpp"

namespace CaDiCaL {
class CExternalPropagator : public ExternalPropagator {
  void *data;
  CCaDiCaLExternalPropagatorCallbacks callbacks;

public:
  CExternalPropagator(void *data, CCaDiCaLExternalPropagatorCallbacks callbacks,
                      bool lazy)
      : callbacks(callbacks) {
    ExternalPropagator::is_lazy = lazy;
    assert(callbacks.notify_assignment);
    assert(callbacks.notify_new_decision_level);
    assert(callbacks.notify_backtrack);
    assert(callbacks.cb_check_found_model);
    assert(callbacks.cb_decide);
    assert(callbacks.cb_propagate);
    assert(callbacks.cb_add_reason_clause_lit);
    assert(callbacks.cb_has_external_clause);
    assert(callbacks.cb_add_external_clause_lit);
  }

  void notify_assignment(int lit, bool is_fixed) override {
    callbacks.notify_assignment(data, lit, is_fixed);
  }

  void notify_new_decision_level() override {
    callbacks.notify_new_decision_level(data);
  }

  void notify_backtrack(size_t new_level) override {
    callbacks.notify_backtrack(data, new_level);
  }

  bool cb_check_found_model(const std::vector<int> &model) override {
    return callbacks.cb_check_found_model(data, model.data(), model.size());
  }

  int cb_decide() override { return callbacks.cb_decide(data); }

  int cb_propagate() override { return callbacks.cb_propagate(data); }

  int cb_add_reason_clause_lit(int propagated_lit) override {
    return callbacks.cb_add_reason_clause_lit(data, propagated_lit);
  }

  bool cb_has_external_clause() override {
    return callbacks.cb_has_external_clause(data);
  }

  int cb_add_external_clause_lit() override {
    return callbacks.cb_add_external_clause_lit(data);
  }
};
} // namespace CaDiCaL
