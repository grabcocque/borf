use crate::evaluator::{InteractionNet, ReductionRules};

pub struct InteractionEngine {
    net: InteractionNet,
    rules: ReductionRules,
}

impl InteractionEngine {
    pub fn new(net: InteractionNet, rules: ReductionRules) -> Self {
        Self { net, rules }
    }

    pub fn normalize(&mut self) -> Result<(), String> {
        // Run until no more reductions are possible
        loop {
            let redexes = self.net.find_redexes();
            if redexes.is_empty() {
                break;
            }

            for redex in redexes {
                self.net.apply_reduction(redex, &self.rules)?;
            }

            // After each reduction step, propagate types
            self.net.infer_types();
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<bool, String> {
        // Perform a single reduction step if possible
        let redexes = self.net.find_redexes();
        if let Some(redex) = redexes.first() {
            self.net.apply_reduction(*redex, &self.rules)?;
            self.net.infer_types();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
