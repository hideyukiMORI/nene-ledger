//! 画面を出せなかった理由。

use core::num::TryFromIntError;

use x11rb::errors::{ConnectError, ConnectionError, ReplyError, ReplyOrIdError};

/// 画面を出せなかった理由。閉じた集合（ARC-010）。
#[derive(Debug)]
pub enum UiFailure {
    /// X サーバにつなげなかった（`DISPLAY` が無い・サーバが居ない）。
    Connect(ConnectError),
    /// 接続が切れた。
    Connection(ConnectionError),
    /// 要求が拒まれた。
    Reply(ReplyError),
    /// 要求が拒まれたか、資源の番号が尽きた。
    ReplyOrId(ReplyOrIdError),
    /// cairo が描けなかった。
    Cairo(cairo::Error),
    /// 描いた面の中身を借りられなかった。
    Borrow(cairo::BorrowError),
    /// 大きさが X の要求の範囲（16 ビット）に収まらない。
    Size(TryFromIntError),
    /// 画面の形式が、描いた面（24 ビットの色・32 ビットの画素・リトルエンディアン）と違う。
    UnsupportedScreen,
}

impl From<ConnectError> for UiFailure {
    fn from(error: ConnectError) -> Self {
        Self::Connect(error)
    }
}

impl From<ConnectionError> for UiFailure {
    fn from(error: ConnectionError) -> Self {
        Self::Connection(error)
    }
}

impl From<ReplyError> for UiFailure {
    fn from(error: ReplyError) -> Self {
        Self::Reply(error)
    }
}

impl From<ReplyOrIdError> for UiFailure {
    fn from(error: ReplyOrIdError) -> Self {
        Self::ReplyOrId(error)
    }
}

impl From<cairo::Error> for UiFailure {
    fn from(error: cairo::Error) -> Self {
        Self::Cairo(error)
    }
}

impl From<cairo::BorrowError> for UiFailure {
    fn from(error: cairo::BorrowError) -> Self {
        Self::Borrow(error)
    }
}

impl From<TryFromIntError> for UiFailure {
    fn from(error: TryFromIntError) -> Self {
        Self::Size(error)
    }
}
