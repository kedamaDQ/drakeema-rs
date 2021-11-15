use mastors::entities::{
	Account,
	Status,
	Visibility,
};

pub(crate) fn is_mine(status: &Status, me: &Account) -> bool {
	status.account().id() == me.id()
}

pub(crate) fn is_boosted(status: &Status) -> bool {
	status.reblog().is_some()
}

pub(crate) fn is_overlapped_at_local_and_home(status: &Status) -> bool {
    status.account().is_local() && status.visibility() == Visibility::Public
}

pub(crate) fn is_mention_to_myself(status: &Status, me: &Account) -> bool {
	status.mentions().iter().any(|m| m.acct() == me.acct())
}

pub(crate) fn has_spoiler_text(status: &Status) -> bool {
	!status.spoiler_text().is_empty()
}
