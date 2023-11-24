#[path = "gift/gift.rs"]
pub(super) mod gift;
pub(super) use gift::GiftComp;

pub(super) mod gifts;

#[path = "new_gift_popup/new_gift_popup.rs"]
pub(super) mod new_gift_popup;
pub(super) use new_gift_popup::NewGiftPopupComp;