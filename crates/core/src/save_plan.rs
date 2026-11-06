//! 保存の計画。原本の区間を写すことと、文字列を書くことの列（ADR 0025 の決定 1）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::plan_sink::PlanSink;
use crate::save_step::SaveStep;

/// 保存の計画。続いた写す区間は 1 つにまとめ、空の歩は持たない。
///
/// 🔴 何も書き換えていなければ、原本の全体を写す 1 歩になる（KEEP-02。BOM と最後の改行を含む）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavePlan {
    steps: Vec<SaveStep>,
}

impl SavePlan {
    /// 空の計画。
    #[must_use]
    pub fn empty() -> Self {
        Self { steps: Vec::new() }
    }

    /// 歩の列。
    #[must_use]
    pub fn steps(&self) -> &[SaveStep] {
        &self.steps
    }

    /// 歩を順に書き先 `sink` に渡す。
    pub fn replay<S: PlanSink + ?Sized>(&self, sink: &mut S) {
        for step in &self.steps {
            match step {
                SaveStep::Copy { offset, length } => sink.copy(*offset, *length),
                SaveStep::Text(text) => sink.text(text),
            }
        }
    }
}

impl PlanSink for SavePlan {
    /// 原本の区間を写す歩を足す。前の歩が直前まで写す区間なら、それを伸ばす。長さ 0 なら何もしない。
    fn copy(&mut self, offset: u64, length: u64) {
        if length == 0 {
            return;
        }
        if let Some(SaveStep::Copy {
            offset: before,
            length: written,
        }) = self.steps.last_mut()
            && before.saturating_add(*written) == offset
        {
            *written = written.saturating_add(length);
            return;
        }
        self.steps.push(SaveStep::Copy { offset, length });
    }

    /// 文字列を書く歩を足す。前の歩も文字列なら、つなげる。空なら何もしない。
    fn text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if let Some(SaveStep::Text(before)) = self.steps.last_mut() {
            before.push_str(text);
            return;
        }
        self.steps.push(SaveStep::Text(String::from(text)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_copies_and_texts_are_merged_and_empty_steps_dropped() {
        let mut plan = SavePlan::empty();
        plan.copy(0, 3);
        plan.copy(3, 4);
        plan.copy(9, 0);
        plan.text("");
        plan.text("ab");
        plan.text("c");
        plan.copy(10, 2);
        plan.copy(13, 1);
        assert_eq!(
            plan.steps(),
            [
                SaveStep::Copy {
                    offset: 0,
                    length: 7
                },
                SaveStep::Text(String::from("abc")),
                SaveStep::Copy {
                    offset: 10,
                    length: 2
                },
                SaveStep::Copy {
                    offset: 13,
                    length: 1
                },
            ]
        );
    }
}
