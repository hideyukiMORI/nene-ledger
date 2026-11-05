//! 窓が描く相手の口。

use nene_ledger_core::frame::Frame;
use nene_ledger_core::pixel_point::PixelPoint;

use crate::clipboard_text::ClipboardText;
use crate::intent::Intent;
use crate::pointer_shape::PointerShape;
use crate::window_view::WindowView;
use crate::zoom_level::ZoomLevel;

/// 窓が描く相手の口（port）。合成ルートが組み、UI は描いて起こされたら取り込むだけ（ARC-011）。
pub trait Session {
    /// 今の窓 `view` の画面。
    fn frame(&self, view: WindowView) -> Frame;

    /// 起こされた。作業のスレッドの知らせと時計の刻みを状態に取り込む。`view` は今の窓。
    fn refresh(&mut self, view: WindowView);

    /// 今の表と詳細ペインの文字の大きさ。UI はこれが変わったら中身の字を測り直す（ADR 0020）。
    fn zoom(&self) -> ZoomLevel;

    /// 利用者の意図。`view` は今の窓（1 画面の大きさを決める）。コピーなら、クリップボードに置く文字列を返す。
    fn intent(&mut self, intent: Intent, view: WindowView) -> Option<ClipboardText>;

    /// 指がボタンを押さずに中身の `at` へ動いた（`None` は中身の外）。見た目が変わるなら `true`（描き直す）。
    fn hover(&mut self, at: Option<PixelPoint>, view: WindowView) -> bool;

    /// 中身の上で出すカーソルの形（最後の `hover` と、押している間の状態から・Issue #64）。
    fn pointer_shape(&self) -> PointerShape;

    /// 閉じるよう求められた（× のボタン・Alt+F4・窓の管理者）。すぐ閉じてよければ `true`。
    /// 未保存の書き換えがあれば尋ねる画面を出して `false`（UI は窓を閉じずに描き直す・ADR 0026 の決定 7）。
    fn close(&mut self) -> bool;

    /// 描く相手が閉じると決めた（尋ねる画面で選んだ・保存が終わった）。UI は窓を閉じる。
    fn finished(&self) -> bool;
}
