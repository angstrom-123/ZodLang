use crate::represent::{IR, Instr, InstrKind, Operand};

pub struct Optimiser {
}
impl Default for Optimiser {
    fn default() -> Self {
        Self::new()
    }
}
impl Optimiser {
    pub fn new() -> Self {
        Optimiser {
        }
    }

    pub fn optimise(&mut self, ir: &mut IR) {
        let mut ph: bool = self.peephole(ir);
        while ph { ph = self.peephole(ir); }
    }

    fn peephole(&mut self, ir: &mut IR) -> bool {
        let mut did_change: bool = false;
        while ir.has_instr() {
           let instr: Instr = ir.peek_instr();
           // match instr.kind {
           //      InstrKind::Pop => {
           if instr.kind == InstrKind::Pop {
                    let prev_instr: &mut Instr = self.prev_instr(ir);
                    if prev_instr.kind == InstrKind::Push {
                        did_change = true;
                        let operb: Operand = prev_instr.opera.clone();
                        prev_instr.kind = InstrKind::Culled;
                        let replace: &mut Instr = ir.instrs.get_mut(ir.cur).unwrap();
                        if instr.opera != operb {
                            *replace = Instr {
                                kind: InstrKind::CopyToRegA,
                                opera: instr.opera.clone(),
                                operb
                            };
                        } else {
                            replace.kind = InstrKind::Culled;
                        }
                        ir.consume_instr();
                    }
           //      },
           //      _ => {}
           }
           ir.consume_instr();
        }
        
        ir.instrs.retain(|i| i.kind != InstrKind::Culled);
        did_change
    }

    fn prev_instr<'a> (&self, ir: &'a mut IR) -> &'a mut Instr {
        let mut ofst: usize = 1;
        let mut instr: &Instr = ir.instrs.get(ir.cur - ofst).unwrap();
        while matches!(instr.kind, InstrKind::Comment | InstrKind::Culled) {
            ofst += 1;
            instr = ir.instrs.get(ir.cur - ofst).unwrap();
        }
        ir.instrs.get_mut(ir.cur - ofst).unwrap()
    }
}
