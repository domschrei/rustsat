// CaDiCaL C API Extension For External Propagators (Christoph Jabs)

#include "cipasirup.h"
#include <cassert>

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

extern "C" {

void ccadical_connect_external_propagator(
    CCaDiCaL *wrapper, void *data,
    CCaDiCaLExternalPropagatorCallbacks callbacks, int lazy) {
  CExternalPropagator *prop = new CExternalPropagator(data, callbacks, lazy);
  ((Wrapper *)wrapper)->solver->connect_external_propagator(prop);
}

void ccadical_disconnect_external_propagator(CCaDiCaL *wrapper) {
  ((Wrapper *)wrapper)->solver->disconnect_external_propagator();
}

void ccadical_add_observed_var(CCaDiCaL *wrapper, int var) {
  ((Wrapper *)wrapper)->solver->add_observed_var(var);
}

void ccadical_remove_observed_var(CCaDiCaL *wrapper, int var) {
  ((Wrapper *)wrapper)->solver->remove_observed_var(var);
}

void ccadical_reset_observed_vars(CCaDiCaL *wrapper) {
  ((Wrapper *)wrapper)->solver->reset_observed_vars();
}

int ccadical_is_decision(CCaDiCaL *wrapper, int lit) {
  return ((Wrapper *)wrapper)->solver->is_decision(lit);
}
}
